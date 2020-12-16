use crate::db::{cache, PgConn, RedisConn};
use crate::routes::{ApiResponse, OptionExt};
use crate::utils::auth::User;
use crate::utils::log::{self, LogInfo};
use crate::utils::player::{decode_track, get_client, ClientExt};
use crate::utils::polling;
use crate::utils::queue::{Queue, QueueItem};

use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use rocket_contrib::json::Json;
use serde::Deserialize;
use twilight_andesite::model::Stop;
use twilight_model::id::GuildId;

#[derive(Debug, Clone, Deserialize)]
pub struct SimpleQueueItem {
    pub track: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimplePosition {
    pub position: u32,
}

#[get("/<id>/queue")]
pub fn get_guild_queue(redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;
    user.is_connected(&redis_conn, id, false)?;

    let queue = Queue::from(&redis_conn, id).get()?;

    ApiResponse::ok().data(queue)
}

#[post("/<id>/queue", data = "<item>")]
pub fn post_guild_queue(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
    item: Json<SimpleQueueItem>,
) -> ApiResponse {
    user.has_manage_track(&*conn, &redis_conn, id)?;
    user.is_connected(&redis_conn, id, true)?;

    let queue = Queue::from(&redis_conn, id);
    let config = cache::get_config(&*conn, &redis_conn, id)?;

    if queue.len()? >= config.max_queue as usize {
        return ApiResponse::bad_request().message("The queue is already at maximum length.");
    }

    let track = {
        let decoded_track = decode_track(
            percent_encode(item.track.as_bytes(), NON_ALPHANUMERIC)
                .to_string()
                .as_str(),
        )
        .map_err(|_| {
            ApiResponse::bad_request().message("The requested track could not be found.")
        })?;

        QueueItem::from((decoded_track, user.user.clone()))
    };

    if config.no_duplicate && queue.get()?.iter().any(|item| item.track == track.track) {
        return ApiResponse::bad_request()
            .message("Duplicated tracks are not allowed in this server.");
    }

    queue.add(&track)?;

    let client = get_client();
    let player = client.get_player(&redis_conn, id)?;
    if player.position().is_none() {
        queue.set_playing(queue.get()?.len() as i32 - 1)?;
        queue.play_with(&player)?;
    }

    log::register(&*conn, &redis_conn, id, user, LogInfo::QueueAdd(track))?;

    polling::notify(id);

    ApiResponse::ok()
}

#[delete("/<id>/queue")]
pub fn delete_guild_queue(conn: PgConn, redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_manage_queue(&*conn, &redis_conn, id)?;
    user.is_connected(&redis_conn, id, true)?;

    let client = get_client();
    if let Ok(player) = client.get_player(&redis_conn, id) {
        player.send(Stop::new(GuildId(id)))?;
    }

    let queue = Queue::from(&redis_conn, id).delete()?;

    log::register(&*conn, &redis_conn, id, user, LogInfo::QueueClear(queue))?;

    polling::notify(id);

    ApiResponse::ok()
}

#[post("/<id>/queue/shuffle")]
pub fn post_guild_queue_shuffle(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
) -> ApiResponse {
    user.has_manage_queue(&*conn, &redis_conn, id)?;
    user.is_connected(&redis_conn, id, true)?;

    Queue::from(&redis_conn, id).shuffle()?;

    log::register(&*conn, &redis_conn, id, user, LogInfo::QueueShuffle)?;

    polling::notify(id);

    ApiResponse::ok()
}

#[put("/<id>/queue/<item>/position", data = "<new_position>")]
pub fn put_guild_queue_item_position(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
    item: u32,
    new_position: Json<SimplePosition>,
) -> ApiResponse {
    user.has_manage_queue(&*conn, &redis_conn, id)?;
    user.is_connected(&redis_conn, id, true)?;

    let new_position = new_position.into_inner();

    let queue = Queue::from(&redis_conn, id);
    let track = queue.get_track(item)?.into_not_found()?;
    let playing = queue.get_playing()?;

    if playing == item as i32 || playing == new_position.position as i32 {
        return ApiResponse::bad_request()
            .message("The position of the currently playing track cannot be changed.");
    }

    if queue.len()? <= new_position.position as usize {
        return ApiResponse::bad_request().message("The position to move the track to is invalid.");
    }

    Queue::from(&redis_conn, id).shift(item, new_position.position)?;

    log::register(
        &*conn,
        &redis_conn,
        id,
        user,
        LogInfo::QueueShift(track, new_position),
    )?;

    polling::notify(id);

    ApiResponse::ok()
}

#[delete("/<id>/queue/<item>")]
pub fn delete_guild_queue_item(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    id: u64,
    item: u32,
) -> ApiResponse {
    user.has_manage_queue(&*conn, &redis_conn, id)?;
    user.is_connected(&redis_conn, id, true)?;

    let queue = Queue::from(&redis_conn, id);
    queue.get_track(item)?.into_not_found()?;

    if queue.get_playing()? == item as i32 {
        return ApiResponse::bad_request()
            .message("The currently playing track cannot be removed.");
    }

    let removed_track = queue.remove(item)?;

    log::register(
        &*conn,
        &redis_conn,
        id,
        user,
        LogInfo::QueueRemove(removed_track),
    )?;

    polling::notify(id);

    ApiResponse::ok()
}
