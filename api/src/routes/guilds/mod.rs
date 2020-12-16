use crate::constants::{FETCH_STAT_TRACKS, FETCH_STAT_USERS, POLLING_TIMEOUT};
use crate::db::pubsub::models::Guild;
use crate::db::pubsub::Message;
use crate::db::{cache, PgConn, RedisConn};
use crate::models::config::EditConfig;
use crate::models::{self, config, guild_log, guild_stat, playlist_item, Validate};
use crate::routes::{ApiResponse, OptionExt};
use crate::utils::auth::User;
use crate::utils::log::{self, LogInfo};
use crate::utils::player::{get_client, ClientExt};
use crate::utils::polling;
use crate::utils::queue::Queue;

use rocket_contrib::json::Json;
use serde::Serialize;
use std::collections::HashMap;
use twilight_andesite::model::Destroy;
use twilight_model::id::GuildId;

mod player;
mod playlist;
mod queue;

pub use player::*;
pub use playlist::*;
pub use queue::*;

#[derive(Debug, Clone, Serialize)]
pub struct GuildStats {
    pub top_tracks: Vec<String>,
    pub top_users: Vec<i64>,
}

#[get("/<id>")]
pub fn get_guild(redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;

    let guild: Guild = Message::get_guild(id)
        .send_and_wait(&redis_conn)?
        .into_not_found()?;

    ApiResponse::ok().data(guild)
}

#[delete("/<id>")]
pub fn delete_guild(conn: PgConn, redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;

    let guild: Guild = Message::get_guild(id)
        .send_and_wait(&redis_conn)?
        .ok_or_else(ApiResponse::not_found)?;
    if guild.owner != user.user.id {
        return ApiResponse::forbidden();
    }

    let client = get_client();
    if let Ok(player) = client.get_player(&redis_conn, id) {
        player.send(Destroy::new(GuildId(id)))?;
        Queue::from(&redis_conn, id).delete()?;
    }

    let playlists = models::playlist::find_by_guild(&*conn, id as i64)?;
    for playlist in playlists {
        playlist_item::delete_by_playlist(&*conn, playlist.id)?;
    }

    models::playlist::delete_by_guild(&*conn, id as i64)?;
    guild_stat::delete_by_guild(&*conn, id as i64)?;
    guild_log::delete_by_guild(&*conn, id as i64)?;
    config::delete(&*conn, id as i64)?;

    ApiResponse::ok()
}

#[get("/<id>/polling")]
pub fn get_guild_polling(redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;

    polling::listen(id, POLLING_TIMEOUT as u64);

    ApiResponse::ok()
}

#[get("/<id>/stats")]
pub fn get_guild_stats(conn: PgConn, redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;

    let stats = guild_stat::find_by_guild(&*conn, id as i64)?;
    let mut top_tracks = HashMap::new();
    let mut top_users = HashMap::new();

    for stat in stats {
        let track_counter = top_tracks.entry(stat.title).or_insert(0);
        *track_counter += 1;

        let user_counter = top_users.entry(stat.author).or_insert(0);
        *user_counter += 1;
    }

    let mut top_tracks = top_tracks
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect::<Vec<(String, i32)>>();
    top_tracks.sort_by(|a, b| a.1.cmp(&b.1));

    let mut top_users = top_users
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect::<Vec<(i64, i32)>>();
    top_users.sort_by(|a, b| a.1.cmp(&b.1));

    let guild_stats = GuildStats {
        top_tracks: top_tracks
            .iter()
            .take(FETCH_STAT_TRACKS)
            .map(|(k, _)| k.clone())
            .collect(),
        top_users: top_users
            .iter()
            .take(FETCH_STAT_USERS)
            .map(|(k, _)| *k)
            .collect(),
    };

    ApiResponse::ok().data(guild_stats)
}

#[delete("/<id>/stats")]
pub fn delete_guild_stats(conn: PgConn, redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;

    let guild: Guild = Message::get_guild(id)
        .send_and_wait(&redis_conn)?
        .ok_or_else(ApiResponse::not_found)?;
    if guild.owner != user.user.id {
        return ApiResponse::forbidden();
    }

    guild_stat::delete_by_guild(&*conn, id as i64)?;

    ApiResponse::ok()
}

#[get("/<id>/settings")]
pub fn get_guild_settings(conn: PgConn, redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;

    let config = cache::get_config(&*conn, &redis_conn, id)?;

    ApiResponse::ok().data(config)
}

#[patch("/<id>/settings", data = "<new_settings>")]
pub fn patch_guild_settings(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
    new_settings: Json<EditConfig>,
) -> ApiResponse {
    user.has_manage_guild(&*conn, &redis_conn, id)?;

    let new_settings = new_settings.into_inner();
    new_settings.check()?;

    let guild: Guild = Message::get_guild(id)
        .send_and_wait(&redis_conn)?
        .into_not_found()?;

    let invalid_roles: Vec<String> = [
        new_settings.track_roles.clone(),
        new_settings.player_roles.clone(),
        new_settings.queue_roles.clone(),
        new_settings.playlist_roles.clone(),
        new_settings.guild_roles.clone(),
    ]
    .iter()
    .filter(|roles| roles.is_some())
    .flat_map(|roles| roles.clone().unwrap())
    .filter(|role| guild.roles.iter().all(|guild_role| *role != guild_role.id))
    .map(|role| role.to_string())
    .collect();

    let invalid_channels: Vec<String> = [
        new_settings.playing_log,
        new_settings.player_log,
        new_settings.queue_log,
    ]
    .iter()
    .filter(|channel| channel.is_some() && channel.unwrap() != 0)
    .map(|channel| channel.unwrap())
    .filter(|channel| {
        guild
            .channels
            .iter()
            .all(|guild_channel| *channel != guild_channel.id)
    })
    .map(|channel| channel.to_string())
    .collect();

    if !invalid_roles.is_empty() {
        return ApiResponse::bad_request()
            .message(format!("Invalid roles: {}", invalid_roles.join(", ")).as_str());
    }

    if !invalid_channels.is_empty() {
        return ApiResponse::bad_request()
            .message(format!("Invalid channels: {}", invalid_channels.join(", ")).as_str());
    }

    config::update(&*conn, id as i64, &new_settings)?;
    cache::invalidate_config(&*conn, &redis_conn, id)?;

    log::register(
        &*conn,
        &redis_conn,
        id,
        user,
        LogInfo::SettingsUpdate(new_settings),
    )?;

    ApiResponse::ok()
}

#[get("/<id>/logs")]
pub fn get_guild_logs(conn: PgConn, redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_manage_guild(&*conn, &redis_conn, id)?;

    let logs = guild_log::find_by_guild(&*conn, id as i64)?;

    ApiResponse::ok().data(logs)
}
