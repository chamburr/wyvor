use crate::{
    auth::User,
    db::{
        PgPool, RedisPool,
    },
    routes::{ApiResponse, ApiResult},
    utils::{
        player::Player,
        player::PlayerState,
        queue::Queue,
    },
};
use actix_web_lab::extract::Path;
use actix_web::{
    delete, get, patch,
    web::{Data, Json},
};
use chrono::Utc;
use crate::websockets::server;
use crate::websockets::server::{ChatServer, General};
use crate::websockets::server::Kind::{UpdatePlayer, UpdateQueue};
use crate::websockets::session;
use actix::*;

/* EVENTUALLYDO: optimise this
#[derive(Debug, Deserialize, Serialize)]
pub struct SimplePlayer {
    pub looping: Option<Loop>,
    pub playing: Option<i32>,
    pub position: Option<u64>,
    pub paused: Option<bool>,
    pub volume: Option<u64>,
    pub filters: Option<Filters>,
}
*/

#[get("/{id}/player")]
pub async fn get_guild_player(
    user: User,
    redis_pool: Data<RedisPool>,
    pool: Data<PgPool>,
    addr: Data<Addr<ChatServer>>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_read_space(&pool, id as i64).await?;

    let mut player = Player::new(&redis_pool, id as i64).await?;
    let position = player.position();
    if position!=-1 && !player.paused() {
        let difference = Utc::now().timestamp_millis() - player.time();

        player.set_position(player.position() + difference);
    }
    player.set_position(position);
    player.set_time(Utc::now().timestamp_millis());

    ApiResponse::ok().data(player).finish()
}
/*
#[post("/{id}/player")]
pub async fn post_guild_player(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;
    
    let connected: Option<Connected> = Message::get_connected(id, None)
        .send_and_wait(&redis_pool)
        .await?;

    if let Some(connected) = connected {
        if connected.members.len() > 1 {
            return ApiResponse::bad_request()
                .message("The bot is already in another channel.")
                .finish();
        }
    }

    let user_connected: models::Connected = Message::get_connected(id, user.user.id as u64)
        .send_and_wait(&redis_pool)
        .await?
        .ok_or_else(|| {
            ApiResponse::bad_request().message("You need to be connected to a channel.")
        })?;

    Message::set_connected(id, user_connected.channel as u64)
        .send_and_pause(&redis_pool)
        .await?;

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::PlayerAdd(user_connected.channel as u64),
    )
    .await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}
*/
#[patch("/{id}/player")]
pub async fn patch_guild_player(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    addr: Data<Addr<ChatServer>>,
    Path(id): Path<u64>,
    Json(mut new_player): Json<PlayerState>,

) -> ApiResult<ApiResponse> {
    user.can_read_space(&pool, id as i64).await?;
    // TODO: Implement checks in the playerstate
    let mut player = Player::new(&redis_pool, id as i64).await?;
    let queue = Queue::new(&redis_pool, id as i64).await?;
    
    player.set_paused(new_player.paused);
    user.can_manage_space(&pool, id as i64).await?;

    player.set_playing(new_player.playing);
    player.set_looping(new_player.looping);
    player.set_position(new_player.position);
    player.set_time(new_player.time);

    if player.paused() == new_player.paused {
        if !player.paused() && player.position() == -1 { // HELP: When it's unpaused, it needs to start??
            player.set_position(0);
        }
    }

    if new_player.playing >= queue.get_length() as i64 || player.playing() < -1 {
        return ApiResponse::bad_request()
            .message("The requested track to play does not exist.")
            .finish();
    }

    
    /*
    if new_player.playing == -1 {
        player::send(Stop::new(GuildId(id))).await?;
        return ApiResponse::ok().finish();
    } */
    
    player.update(&redis_pool).await?;
    let update = General {
        kind: server::Kind::UpdatePlayer,
        data: server::UpdateData::UpdatePlayer(player.clone())
    };
    addr.do_send(update);
    ApiResponse::ok().finish()
}

#[delete("/{id}/player")]
pub async fn delete_guild_player(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {

    user.can_manage_space(&pool, id as i64).await?;
    // TOOD: this whole function
    ApiResponse::ok().finish()

}
