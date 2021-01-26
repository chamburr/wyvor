use crate::constants::{FETCH_STAT_TRACKS, FETCH_STAT_USERS};
use crate::db::pubsub::models::Guild;
use crate::db::pubsub::Message;
use crate::db::{cache, PgPool, RedisPool};
use crate::models::config::EditConfig;
use crate::models::{self, config, guild_log, guild_stat, playlist_item, Validate};
use crate::routes::{ApiResponse, ApiResult, OptionExt};
use crate::utils::auth::User;
use crate::utils::log::{self, LogInfo};
use crate::utils::player::get_player;
use crate::utils::{self, polling};

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, patch};
use serde::Serialize;
use std::collections::HashMap;
use twilight_andesite::model::Destroy;
use twilight_model::id::GuildId;

pub mod player;
pub mod playlist;
pub mod queue;

pub use player::*;
pub use playlist::*;
pub use queue::*;

#[derive(Debug, Serialize)]
pub struct GuildStats {
    pub top_tracks: Vec<String>,
    pub top_users: Vec<i64>,
}

#[get("/{id}")]
pub async fn get_guild(
    user: User,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    let guild: Guild = Message::get_guild(id)
        .send_and_wait(&redis_pool)
        .await?
        .or_not_found()?;

    ApiResponse::ok().data(guild).finish()
}

#[delete("/{id}")]
pub async fn delete_guild(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    let guild: Guild = Message::get_guild(id)
        .send_and_wait(&redis_pool)
        .await?
        .or_not_found()?;

    if guild.owner != user.user.id {
        return ApiResponse::forbidden().finish();
    }

    if get_player(&redis_pool, id).await.is_ok() {
        utils::player::send(Destroy::new(GuildId(id))).await?;
        utils::queue::delete(&redis_pool, id).await?;
    }

    let playlists = models::playlist::find_by_guild(&pool, id as i64).await?;
    for playlist in playlists {
        playlist_item::delete_by_playlist(&pool, playlist.id).await?;
    }

    models::playlist::delete_by_guild(&pool, id as i64).await?;
    guild_stat::delete_by_guild(&pool, id as i64).await?;
    guild_log::delete_by_guild(&pool, id as i64).await?;
    config::delete(&pool, id as i64).await?;

    ApiResponse::ok().finish()
}

#[get("/{id}/polling")]
pub async fn get_guild_polling(
    user: User,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    polling::listen(id).await?;

    ApiResponse::ok().finish()
}

#[get("/{id}/stats")]
pub async fn get_guild_stats(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    let stats = guild_stat::find_by_guild(&pool, id as i64).await?;
    let mut top_tracks = HashMap::new();
    let mut top_users = HashMap::new();

    for stat in stats {
        let track_counter = top_tracks.entry(stat.title).or_insert(0);
        *track_counter += 1;

        let user_counter = top_users.entry(stat.author).or_insert(0);
        *user_counter += 1;
    }

    let mut top_tracks = top_tracks.into_iter().collect::<Vec<(String, i32)>>();
    top_tracks.sort_by(|a, b| a.1.cmp(&b.1));

    let mut top_users = top_users.into_iter().collect::<Vec<(i64, i32)>>();
    top_users.sort_by(|a, b| a.1.cmp(&b.1));

    let guild_stats = GuildStats {
        top_tracks: top_tracks
            .into_iter()
            .take(FETCH_STAT_TRACKS)
            .map(|(k, _)| k)
            .collect(),
        top_users: top_users
            .into_iter()
            .take(FETCH_STAT_USERS)
            .map(|(k, _)| k)
            .collect(),
    };

    ApiResponse::ok().data(guild_stats).finish()
}

#[delete("/{id}/stats")]
pub async fn delete_guild_stats(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    let guild: Guild = Message::get_guild(id)
        .send_and_wait(&redis_pool)
        .await?
        .or_not_found()?;

    if guild.owner != user.user.id {
        return ApiResponse::forbidden().finish();
    }

    guild_stat::delete_by_guild(&pool, id as i64).await?;

    ApiResponse::ok().finish()
}

#[get("/{id}/settings")]
pub async fn get_guild_settings(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    user: User,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    let config = cache::get_config(&pool, &redis_pool, id).await?;

    ApiResponse::ok().data(config).finish()
}

#[patch("/{id}/settings")]
pub async fn patch_guild_settings(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
    Json(new_settings): Json<EditConfig>,
) -> ApiResult<ApiResponse> {
    user.has_manage_guild(&pool, &redis_pool, id).await?;

    new_settings.check()?;

    let guild: Guild = Message::get_guild(id)
        .send_and_wait(&redis_pool)
        .await?
        .or_not_found()?;

    let mut roles = vec![];
    roles.extend(new_settings.track_roles.as_deref());
    roles.extend(new_settings.player_roles.as_deref());
    roles.extend(new_settings.queue_roles.as_deref());
    roles.extend(new_settings.playlist_roles.as_deref());
    roles.extend(new_settings.guild_roles.as_deref());

    let invalid_roles: Vec<String> = roles
        .into_iter()
        .flatten()
        .filter(|role| guild.roles.iter().all(|guild_role| **role != guild_role.id))
        .map(|role| role.to_string())
        .collect();

    let channels = vec![
        new_settings.playing_log.unwrap_or_default(),
        new_settings.player_log.unwrap_or_default(),
        new_settings.queue_log.unwrap_or_default(),
    ];

    let invalid_channels: Vec<String> = channels
        .into_iter()
        .filter(|channel| {
            *channel != 0
                && guild
                    .channels
                    .iter()
                    .all(|guild_channel| *channel != guild_channel.id)
        })
        .map(|channel| channel.to_string())
        .collect();

    if !invalid_roles.is_empty() {
        return ApiResponse::bad_request()
            .message(format!("Invalid roles: {}", invalid_roles.join(", ")).as_str())
            .finish();
    }

    if !invalid_channels.is_empty() {
        return ApiResponse::bad_request()
            .message(format!("Invalid channels: {}", invalid_channels.join(", ")).as_str())
            .finish();
    }

    config::update(&pool, id as i64, new_settings.clone()).await?;
    cache::invalidate_config(&pool, &redis_pool, id).await?;

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::SettingsUpdate(new_settings),
    )
    .await?;

    ApiResponse::ok().finish()
}

#[get("/{id}/logs")]
pub async fn get_guild_logs(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_manage_guild(&pool, &redis_pool, id).await?;

    let logs = guild_log::find_by_guild(&pool, id as i64).await?;

    ApiResponse::ok().data(logs).finish()
}
