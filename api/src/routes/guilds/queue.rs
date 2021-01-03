use crate::db::{cache, PgPool, RedisPool};
use crate::routes::{ApiResponse, ApiResult, OptionExt};
use crate::utils::auth::User;
use crate::utils::log::{self, LogInfo};
use crate::utils::player::{decode_track, get_player};
use crate::utils::queue::{self, QueueItem};
use crate::utils::{player, polling};

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::Deserialize;
use twilight_andesite::model::Stop;
use twilight_model::id::GuildId;

#[derive(Debug, Deserialize)]
pub struct SimpleQueueItem {
    pub track: String,
}

#[derive(Debug, Deserialize)]
pub struct SimplePosition {
    pub position: u32,
}

#[get("/{id}/queue")]
pub async fn get_guild_queue(
    user: User,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;
    user.is_connected(&redis_pool, id, false).await?;

    let tracks = queue::get(&redis_pool, id).await?;

    ApiResponse::ok().data(tracks).finish()
}

#[post("/{id}/queue")]
pub async fn post_guild_queue(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
    Json(item): Json<SimpleQueueItem>,
) -> ApiResult<ApiResponse> {
    user.has_manage_track(&pool, &redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    let config = cache::get_config(&pool, &redis_pool, id).await?;
    let tracks = queue::get(&redis_pool, id).await?;

    if tracks.len() >= config.max_queue as usize {
        return ApiResponse::bad_request()
            .message("The queue is already at maximum length.")
            .finish();
    }

    let track = {
        let decoded_track = decode_track(
            percent_encode(item.track.as_bytes(), NON_ALPHANUMERIC)
                .to_string()
                .as_str(),
        )
        .await
        .map_err(|_| {
            ApiResponse::bad_request().message("The requested track could not be found.")
        })?;

        QueueItem {
            track: decoded_track.track,
            title: decoded_track.info.title,
            uri: decoded_track.info.uri,
            length: decoded_track.info.length as i32,
            author: user.user.id,
            username: user.user.username.clone(),
            discriminator: user.user.discriminator,
        }
    };

    if config.no_duplicate && tracks.iter().any(|item| item.track == track.track) {
        return ApiResponse::bad_request()
            .message("Duplicated tracks are not allowed in this server.")
            .finish();
    }

    queue::add(&redis_pool, id, track.clone()).await?;

    let player = get_player(&redis_pool, id).await?;
    if player.position.is_none() {
        queue::set_playing(&redis_pool, id, tracks.len() as i32).await?;
        queue::play(&redis_pool, id).await?;
    }

    log::register(&pool, &redis_pool, id, user, LogInfo::QueueAdd(track)).await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}

#[delete("/{id}/queue")]
pub async fn delete_guild_queue(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_manage_queue(&pool, &redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    if get_player(&redis_pool, id).await.is_ok() {
        player::send(Stop::new(GuildId(id))).await?;
    }

    let tracks = queue::delete(&redis_pool, id).await?;

    log::register(&pool, &redis_pool, id, user, LogInfo::QueueClear(tracks)).await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}

#[post("/{id}/queue/shuffle")]
pub async fn post_guild_queue_shuffle(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_manage_queue(&pool, &redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    queue::shuffle(&redis_pool, id).await?;

    log::register(&pool, &redis_pool, id, user, LogInfo::QueueShuffle).await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}

#[put("/{id}/queue/{item}/position")]
pub async fn put_guild_queue_item_position(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path((id, item)): Path<(u64, u32)>,
    Json(new_position): Json<SimplePosition>,
) -> ApiResult<ApiResponse> {
    user.has_manage_queue(&pool, &redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    let track = queue::get_track(&redis_pool, id, item as i32)
        .await?
        .or_not_found()?;
    let playing = queue::get_playing(&redis_pool, id).await?;

    if playing == item as i32 || playing == new_position.position as i32 {
        return ApiResponse::bad_request()
            .message("The position of the currently playing track cannot be changed.")
            .finish();
    }

    if queue::len(&redis_pool, id).await? <= new_position.position as usize {
        return ApiResponse::bad_request()
            .message("The position to move the track to is invalid.")
            .finish();
    }

    queue::shift(&redis_pool, id, item, new_position.position).await?;

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::QueueShift(track, new_position),
    )
    .await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}

#[delete("/{id}/queue/{item}")]
pub async fn delete_guild_queue_item(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path((id, item)): Path<(u64, u32)>,
) -> ApiResult<ApiResponse> {
    user.has_manage_queue(&pool, &redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    queue::get_track(&redis_pool, id, item as i32)
        .await?
        .or_not_found()?;

    if queue::get_playing(&redis_pool, id).await? == item as i32 {
        return ApiResponse::bad_request()
            .message("The currently playing track cannot be removed.")
            .finish();
    }

    let removed_track = queue::remove(&redis_pool, id, item).await?;

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::QueueRemove(removed_track),
    )
    .await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}
