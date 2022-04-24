use crate::{
    constants::PLAYLIST_MAX,
    db::{cache, PgPool, RedisPool},
    models::{
        playlist::{self, EditPlaylist, NewPlaylist},
        playlist_item::{self, NewPlaylistItem, PlaylistItem},
        Validate,
    },
    routes::{ApiResponse, ApiResult, OptionExt},
    utils::{
        auth_old::User,
        log::{self, LogInfo},
        polling,
        queue::{self, QueueItem},
    },
};

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SimplePlaylist {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct FullPlaylist {
    pub id: i64,
    pub guild: i64,
    pub name: String,
    pub author: i64,
    pub created_at: NaiveDateTime,
    pub items: Vec<PlaylistItem>,
}

#[get("/{id}/playlists")]
pub async fn get_space_playlists(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    let mut playlists = vec![];
    for playlist in playlist::find_by_guild(&pool, id as i64).await? {
        let playlist = FullPlaylist {
            id: playlist.id,
            guild: playlist.guild,
            name: playlist.name,
            author: playlist.author,
            created_at: playlist.created_at,
            items: playlist_item::find_by_playlist(&pool, playlist.id).await?,
        };
        playlists.push(playlist);
    }

    ApiResponse::ok().data(playlists).finish()
}

#[post("/{id}/playlists")]
pub async fn post_guild_playlists(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
    Json(new_playlist): Json<SimplePlaylist>,
) -> ApiResult<ApiResponse> {
    user.has_manage_playlist(&pool, &redis_pool, id).await?;

    let playlists = playlist::find_by_guild(&pool, id as i64).await?;
    let tracks = queue::get(&redis_pool, id).await?;
    let new_playlist = NewPlaylist {
        guild: id as i64,
        author: user.user.id,
        name: new_playlist.name,
    };

    new_playlist.check()?;

    if playlists.len() >= PLAYLIST_MAX {
        return ApiResponse::bad_request()
            .message("This server has reached the maximum number of playlists.")
            .finish();
    }

    if playlists
        .iter()
        .any(|playlist| playlist.name.to_lowercase() == new_playlist.name.to_lowercase())
    {
        return ApiResponse::bad_request()
            .message("A playlist with the same name already exists.")
            .finish();
    }

    if tracks.is_empty() {
        return ApiResponse::bad_request()
            .message("There are no tracks in the queue currently.")
            .finish();
    }

    let playlist = playlist::create(&pool, new_playlist.clone()).await?;

    for track in tracks {
        let item = NewPlaylistItem {
            playlist: playlist.id,
            track: track.track,
            title: track.title,
            uri: track.uri,
            length: track.length,
        };
        playlist_item::create(&pool, item).await?;
    }

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::PlaylistAdd(new_playlist),
    )
    .await?;

    ApiResponse::ok().finish()
}

#[patch("/{id}/playlists/{item}")]
pub async fn patch_guild_playlist(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path((id, item)): Path<(u64, u64)>,
    Json(new_playlist): Json<EditPlaylist>,
) -> ApiResult<ApiResponse> {
    user.has_manage_playlist(&pool, &redis_pool, id).await?;

    playlist::find(&pool, item as i64).await?;
    new_playlist.check()?;

    if let Some(name) = &new_playlist.name {
        if playlist::find_by_guild(&pool, id as i64)
            .await?
            .iter()
            .any(|playlist| playlist.name.to_lowercase().as_str() == name)
        {
            return ApiResponse::bad_request()
                .message("A playlist with the same name already exists.")
                .finish();
        }
    }

    playlist::update(&pool, item as i64, new_playlist.clone()).await?;

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::PlaylistUpdate(new_playlist),
    )
    .await?;

    ApiResponse::ok().finish()
}

#[delete("/{id}/playlists/{item}")]
pub async fn delete_guild_playlist(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path((id, item)): Path<(u64, u64)>,
) -> ApiResult<ApiResponse> {
    user.has_manage_playlist(&pool, &redis_pool, id).await?;

    let playlist = playlist::find(&pool, item as i64).await?.or_not_found()?;
    playlist_item::delete_by_playlist(&pool, item as i64).await?;
    playlist::delete(&pool, item as i64).await?;

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::PlaylistRemove(playlist),
    )
    .await?;

    ApiResponse::ok().finish()
}

#[post("/{id}/playlists/{item}/load")]
pub async fn post_guild_playlist_load(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path((id, item)): Path<(u64, u64)>,
) -> ApiResult<ApiResponse> {
    user.has_manage_track(&pool, &redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    let playlist = playlist::find(&pool, item as i64).await?.or_not_found()?;
    let tracks = playlist_item::find_by_playlist(&pool, item as i64).await?;
    let config = cache::get_config(&pool, &redis_pool, id).await?;
    let amount = tracks.len();

    if config.max_queue as usize - queue::len(&redis_pool, id).await? < amount {
        return ApiResponse::bad_request()
            .message("The queue is already at maximum length.")
            .finish();
    }

    for track in tracks {
        if config.no_duplicate
            && queue::get(&redis_pool, id)
                .await?
                .iter()
                .any(|item| item.track == track.track)
        {
            continue;
        }

        queue::add(
            &redis_pool,
            id,
            QueueItem {
                track: track.track,
                title: track.title,
                uri: track.uri,
                length: track.length,
                author: user.user.id,
                username: user.user.username.clone(),
                discriminator: user.user.discriminator,
            },
        )
        .await?;
    }

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::PlaylistLoad(playlist, amount as u64),
    )
    .await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}
