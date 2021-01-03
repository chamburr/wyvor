use crate::constants::{user_key, COOKIE_NAME, FETCH_USERS_MAX, USER_KEY_TTL};
use crate::db::pubsub::Message;
use crate::db::{cache, PgPool, RedisPool};
use crate::models::account::{self, Account};
use crate::routes::{ApiResponse, ApiResult, OptionExt, ResultExt};
use crate::utils::auth::User;

use actix_web::web::{Data, Path, Query};
use actix_web::{get, post};
use serde_json::json;
use std::collections::HashMap;

#[get("")]
pub async fn get_users(
    _user: User,
    pool: Data<PgPool>,
    Query(query): Query<HashMap<String, String>>,
) -> ApiResult<ApiResponse> {
    let ids = query
        .get("ids")
        .or_bad_request()?
        .split(',')
        .map(|value| value.parse().or_bad_request())
        .collect::<ApiResult<Vec<u64>>>()?;

    if ids.len() > FETCH_USERS_MAX {
        return ApiResponse::bad_request()
            .message("The request has exceeded the limit for the maximum number of users.")
            .finish();
    }

    let mut users = vec![];
    for id in ids {
        if let Some(user) = account::find(&pool, id as i64).await? {
            users.push(user);
        }
    }

    ApiResponse::ok().data(users).finish()
}

#[get("/{id}")]
pub async fn get_user(
    _user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    if let Some(user) = account::find(&pool, id as i64).await? {
        return ApiResponse::ok().data(user).finish();
    }

    let user: Account = Message::get_user(id)
        .send_and_wait(&redis_pool)
        .await?
        .or_not_found()?;

    cache::set_and_expire(&redis_pool, user_key(id), &user, USER_KEY_TTL).await?;

    ApiResponse::ok().data(user).finish()
}

#[get("/@me")]
pub async fn get_user_me(user: User) -> ApiResult<ApiResponse> {
    user.is_not_bot()?;

    ApiResponse::ok().data(user.user).finish()
}

#[get("/@me/guilds")]
pub async fn get_user_me_guilds(user: User, redis_pool: Data<RedisPool>) -> ApiResult<ApiResponse> {
    user.is_not_bot()?;

    let guilds = user.get_guilds(&redis_pool).await?;

    ApiResponse::ok().data(guilds).finish()
}

#[get("/@me/guilds/{id}")]
pub async fn get_users_me_guild(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    let permissions = json!({
        "manage_guild": user.has_manage_guild(&pool, &redis_pool, id).await.is_ok(),
        "manage_playlist": user.has_manage_playlist(&pool, &redis_pool, id).await.is_ok(),
        "manage_player": user.has_manage_player(&pool, &redis_pool, id).await.is_ok(),
        "manage_queue": user.has_manage_queue(&pool, &redis_pool, id).await.is_ok(),
        "manage_track": user.has_manage_track(&pool, &redis_pool, id).await.is_ok(),
    });

    ApiResponse::ok().data(permissions).finish()
}

#[post("/@me/logout")]
pub async fn post_user_me_logout(user: User) -> ApiResult<ApiResponse> {
    user.is_not_bot()?;

    user.revoke_token().await?;

    ApiResponse::ok().del_cookie(COOKIE_NAME).finish()
}
