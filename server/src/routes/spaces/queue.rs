use crate::{
    db::{cache, PgPool, RedisPool},
    routes::{ApiResponse, ApiResult, OptionExt},
    utils::{
        auth_old::User,
        log::{self, LogInfo},
        player,
        player::{decode_track, get_player},
        polling,
        queue::{self, QueueItem},
    },
};
use crate::utils::Queue 
#[derive(Debug, Deserialize)]
pub struct SimpleQueueItem {
    pub track: String,
}

#[derive(Debug, Deserialize)]
pub struct SimplePosition {
    pub position: u32,
}

use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path},
};

use serde::Deserialize;



#[get("/{id}/queue")]
pub async fn get_guild_queue(
    user: User,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;
    user.is_connected(&redis_pool, id, false).await?;

    let tracks = Queue::new(&redis_pool, id).await?;

    ApiResponse::ok().data(tracks).finish()
}

#[post("/{id}/queue")]
pub async fn post_guild_queue( // append something into the queue
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
    Json(item): Json<SimpleQueueItem>,
) -> ApiResult<ApiResponse> {
    user.has_manage_track(&pool, &redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    //let config = cache::get_config(&pool, &redis_pool, id).await?;
    let tracks = Queue::new(&redis_pool, id).await?;
    
    // TODO: this whole part
    if tracks.len() >= 200 {
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
    if player.position().is_none() {
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

    let tracks = Queue::new(&redis_pool, id).await?;
    let mut player = Player::new(&redis_pool, id).await?;
    player.set_playing(-1);
    tracks.delete().await?;
    player.update(&redis_pool).await?;

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

    let mut tracks = Queue::new(&redis_pool, id).await?;
    let player = Player::new(&redis_pool, id).await?;
    let current_track = tracks.remove(player.playing());
    tracks.shuffle();
    tracks.insert(player.playing(), current_track);
    tracks.update(&redis_pool).await?;
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
    let mut tracks = Queue::new(&redis_pool, id).await?;
    let mut player = Player::new(&redis_pool, id).await?;
    if item == player.playing() {
        return ApiResponse::bad_request()
            .message("You cannot move the currently playing track.")
            .finish();
    }

    let current_track = tracks.remove(item).await?;
    tracks.insert(new_position.position, current_track).await?;
    
    if item < player.playing() && new_position.position > player.playing() {
        player.set_playing(player.playing() - 1);
    } else if item > player.playing() and new_position.position <= player.playing() {
        player.set_playing(player.playing() + 1);
    }
    tracks.update(&redis_pool).await?;
    player.update(&redis_pool).await?;
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

    let mut tracks = Queue::new(&redis_pool, id).await?;
    let mut player = Player::new(&redis_pool, id).await?;

    if player.playing() == item as i32 {
        return ApiResponse::bad_request()
            .message("The currently playing track cannot be removed.")
            .finish();
    }
    tracks.remove(item).await?;
    tracks.update(&redis_pool).await?;
    if item < player.playing {
        player.set_playing(player.playing() - 1);
    }
    player.update(&redis_pool).await?;

    ApiResponse::ok().finish()
}
