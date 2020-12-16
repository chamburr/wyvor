use crate::constants::PLAYLIST_MAX;
use crate::db::{cache, PgConn, RedisConn};
use crate::models::playlist::{self, EditPlaylist, NewPlaylist};
use crate::models::playlist_item::{self, NewPlaylistItem, PlaylistItem};
use crate::models::Validate;
use crate::routes::{ApiResponse, ApiResult};
use crate::utils::auth::User;
use crate::utils::log::{self, LogInfo};
use crate::utils::polling;
use crate::utils::queue::{Queue, QueueItem};

use chrono::NaiveDateTime;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SimplePlaylist {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FullPlaylist {
    pub id: i64,
    pub guild: i64,
    pub name: String,
    pub author: i64,
    pub created_at: NaiveDateTime,
    pub items: Vec<PlaylistItem>,
}

#[get("/<id>/playlists")]
pub fn get_guild_playlists(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;

    let playlists = playlist::find_by_guild(&*conn, id as i64)?
        .iter()
        .map(|playlist| {
            Ok(FullPlaylist {
                id: playlist.id,
                guild: playlist.guild,
                name: playlist.name.clone(),
                author: playlist.author,
                created_at: playlist.created_at,
                items: playlist_item::find_by_playlist(&*conn, playlist.id)?,
            })
        })
        .collect::<ApiResult<Vec<FullPlaylist>>>()?;

    ApiResponse::ok().data(playlists)
}

#[post("/<id>/playlists", data = "<new_playlist>")]
pub fn post_guild_playlists(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
    new_playlist: Json<SimplePlaylist>,
) -> ApiResponse {
    user.has_manage_playlist(&*conn, &redis_conn, id)?;

    let playlists = playlist::find_by_guild(&*conn, id as i64)?;
    let queue = Queue::from(&redis_conn, id).get()?;
    let new_playlist = NewPlaylist {
        guild: id as i64,
        author: user.user.id,
        name: new_playlist.into_inner().name,
    };

    new_playlist.check()?;

    if playlists.len() >= PLAYLIST_MAX {
        return ApiResponse::bad_request()
            .message("This server has reached the maximum number of playlists.");
    }

    if playlists
        .iter()
        .any(|playlist| playlist.name.to_lowercase() == new_playlist.name.to_lowercase())
    {
        return ApiResponse::bad_request().message("A playlist with the same name already exists.");
    }

    if queue.is_empty() {
        return ApiResponse::bad_request().message("There are no tracks in the queue currently.");
    }

    let playlist = playlist::create(&*conn, &new_playlist)?;
    let tracks: Vec<NewPlaylistItem> = queue
        .iter()
        .map(|item| NewPlaylistItem {
            playlist: playlist.id,
            track: item.track.clone(),
            title: item.title.clone(),
            uri: item.uri.clone(),
            length: item.length,
        })
        .collect();

    for track in tracks {
        playlist_item::create(&*conn, &track)?;
    }

    log::register(
        &*conn,
        &redis_conn,
        id,
        user,
        LogInfo::PlaylistAdd(new_playlist),
    )?;

    ApiResponse::ok()
}

#[patch("/<id>/playlists/<item>", data = "<new_playlist>")]
pub fn patch_guild_playlist(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
    item: u64,
    new_playlist: Json<EditPlaylist>,
) -> ApiResponse {
    user.has_manage_playlist(&*conn, &redis_conn, id)?;

    let new_playlist = new_playlist.into_inner();
    playlist::find(&*conn, item as i64)?;
    new_playlist.check()?;

    if let Some(name) = new_playlist.name.clone() {
        if playlist::find_by_guild(&*conn, id as i64)?
            .iter()
            .any(|playlist| playlist.name.to_lowercase() == name)
        {
            return ApiResponse::bad_request()
                .message("A playlist with the same name already exists.");
        }
    }

    playlist::update(&*conn, item as i64, &new_playlist)?;

    log::register(
        &*conn,
        &redis_conn,
        id,
        user,
        LogInfo::PlaylistUpdate(new_playlist),
    )?;

    ApiResponse::ok()
}

#[delete("/<id>/playlists/<item>")]
pub fn delete_guild_playlist(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
    item: u64,
) -> ApiResponse {
    user.has_manage_playlist(&*conn, &redis_conn, id)?;

    let playlist = playlist::find(&*conn, item as i64)?;
    playlist_item::delete_by_playlist(&*conn, item as i64)?;
    playlist::delete(&*conn, item as i64)?;

    log::register(
        &*conn,
        &redis_conn,
        id,
        user,
        LogInfo::PlaylistRemove(playlist),
    )?;

    ApiResponse::ok()
}

#[post("/<id>/playlists/<item>/load")]
pub fn post_guild_playlist_load(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
    item: u64,
) -> ApiResponse {
    user.has_manage_track(&*conn, &redis_conn, id)?;
    user.is_connected(&redis_conn, id, true)?;

    let playlist = playlist::find(&*conn, item as i64)?;
    let tracks = playlist_item::find_by_playlist(&*conn, item as i64)?;
    let config = cache::get_config(&*conn, &redis_conn, id)?;
    let queue = Queue::from(&redis_conn, id);

    if config.max_queue as usize - queue.len()? < tracks.len() {
        return ApiResponse::bad_request().message("The queue is already at maximum length.");
    }

    for track in tracks.clone() {
        if config.no_duplicate && queue.get()?.iter().any(|item| item.track == track.track) {
            continue;
        }

        queue.add(&QueueItem::from((track, user.user.clone())))?;
    }

    log::register(
        &*conn,
        &redis_conn,
        id,
        user,
        LogInfo::PlaylistLoad(playlist, tracks),
    )?;

    polling::notify(id);

    ApiResponse::ok()
}
