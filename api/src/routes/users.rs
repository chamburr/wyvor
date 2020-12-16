use crate::constants::{FETCH_USERS_MAX, USER_KEY, USER_KEY_TTL};
use crate::db::pubsub::Message;
use crate::db::{cache, PgConn, RedisConn};
use crate::models::account::{self, Account};
use crate::routes::{ApiResponse, OptionExt};
use crate::utils::auth::{get_remove_token_cookie, User};

use rocket::http::Cookies;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SimplePermissions {
    pub manage_guild: bool,
    pub manage_playlist: bool,
    pub manage_player: bool,
    pub manage_queue: bool,
    pub manage_track: bool,
}

#[get("/?<ids>")]
pub fn get_users(
    conn: PgConn,
    _redis_conn: RedisConn,
    _user: User,
    ids: Option<String>,
) -> ApiResponse {
    let ids = ids.into_bad_request()?;
    let ids = ids
        .split(',')
        .map(|value| value.parse().map_err(|_| ApiResponse::bad_request()))
        .collect::<Result<Vec<u64>, ApiResponse>>()?;

    if ids.len() > FETCH_USERS_MAX {
        return ApiResponse::bad_request()
            .message("The request has exceeded the limit for the maximum number of users.");
    }

    let mut users = vec![];
    for id in ids {
        if let Ok(user) = account::find(&*conn, id as i64) {
            users.push(user);
        }
    }

    ApiResponse::ok().data(users)
}

#[get("/<id>")]
pub fn get_user(conn: PgConn, redis_conn: RedisConn, _user: User, id: u64) -> ApiResponse {
    let user = account::find(&*conn, id as i64);

    if let Ok(user) = user {
        return ApiResponse::ok().data(user);
    }

    let user: Account = Message::get_user(id)
        .send_and_wait(&redis_conn)?
        .into_not_found()?;

    cache::set_and_expire(
        &redis_conn,
        &format!("{}{}", USER_KEY, id),
        &user,
        USER_KEY_TTL,
    )?;

    ApiResponse::ok().data(user)
}

#[get("/@me")]
pub fn get_user_me(_redis_conn: RedisConn, user: User) -> ApiResponse {
    user.is_not_bot()?;

    ApiResponse::ok().data(user.user)
}

#[get("/@me/guilds")]
pub fn get_user_me_guilds(redis_conn: RedisConn, user: User) -> ApiResponse {
    user.is_not_bot()?;

    let guilds = user.get_guilds(&redis_conn)?;
    ApiResponse::ok().data(guilds)
}

#[get("/@me/guilds/<id>")]
pub fn get_users_me_guild(conn: PgConn, redis_conn: RedisConn, user: User, id: u64) -> ApiResponse {
    user.has_read_guild(&redis_conn, id)?;

    let permissions = SimplePermissions {
        manage_guild: user.has_manage_guild(&*conn, &redis_conn, id).is_ok(),
        manage_playlist: user.has_manage_playlist(&*conn, &redis_conn, id).is_ok(),
        manage_player: user.has_manage_player(&*conn, &redis_conn, id).is_ok(),
        manage_queue: user.has_manage_queue(&*conn, &redis_conn, id).is_ok(),
        manage_track: user.has_manage_track(&*conn, &redis_conn, id).is_ok(),
    };

    ApiResponse::ok().data(permissions)
}

#[post("/@me/logout")]
pub fn post_user_me_logout(
    _redis_conn: RedisConn,
    user: User,
    mut cookies: Cookies<'_>,
) -> ApiResponse {
    user.is_not_bot()?;

    user.revoke_token()?;
    cookies.remove_private(get_remove_token_cookie());

    ApiResponse::ok()
}
