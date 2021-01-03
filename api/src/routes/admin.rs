use crate::db::{cache, PgPool, RedisPool};
use crate::models::blacklist::{self, EditBlacklist, NewBlacklist};
use crate::models::guild::{self, Guild};
use crate::models::{account, Validate};
use crate::routes::{ApiResponse, ApiResult, OptionExt, ResultExt};
use crate::utils::auth::User;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, put};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleBlacklist {
    pub reason: String,
}

#[get("/guilds")]
pub async fn get_guilds(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Query(mut query): Query<HashMap<String, String>>,
) -> ApiResult<ApiResponse> {
    user.has_bot_admin(&redis_pool).await?;

    let name = query.remove("name");
    let owner = query
        .remove("owner")
        .map(|user| user.parse::<u64>().or_bad_request())
        .transpose()?;

    if name.is_none() && owner.is_none() {
        return ApiResponse::bad_request().finish();
    }

    let all_guilds: Vec<Guild> = if let Some(name) = name {
        guild::find_by_name(&pool, name)
            .await?
            .into_iter()
            .filter(|guild| owner == None || Some(guild.owner as u64) == owner)
            .collect()
    } else {
        guild::find_by_owner(&pool, owner.unwrap_or_default() as i64).await?
    };

    ApiResponse::ok().data(all_guilds).finish()
}

#[get("/top_guilds")]
pub async fn get_top_guilds(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    user: User,
    Query(mut query): Query<HashMap<String, String>>,
) -> ApiResult<ApiResponse> {
    user.has_bot_admin(&redis_pool).await?;

    let amount = query
        .remove("amount")
        .map(|user| user.parse::<u64>().or_bad_request())
        .transpose()?
        .or_bad_request()?;

    let guilds = guild::find_by_member_count(&pool, amount as i64).await?;

    ApiResponse::ok().data(guilds).finish()
}

#[get("/blacklist")]
pub async fn get_blacklist(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    user: User,
) -> ApiResult<ApiResponse> {
    user.has_bot_admin(&redis_pool).await?;

    let blacklist = blacklist::all(&pool).await?;

    ApiResponse::ok().data(blacklist).finish()
}

#[put("/blacklist/{item}")]
pub async fn put_blacklist_item(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    user: User,
    Path(item): Path<u64>,
    Json(new_blacklist): Json<SimpleBlacklist>,
) -> ApiResult<ApiResponse> {
    user.has_bot_admin(&redis_pool).await?;

    let new_blacklist = NewBlacklist {
        id: item as i64,
        reason: new_blacklist.reason,
        author: user.user.id,
    };

    account::find(&pool, item as i64).await?;
    new_blacklist.check()?;

    if cache::get_blacklist_item(&redis_pool, item)
        .await?
        .is_some()
    {
        return ApiResponse::bad_request()
            .message("The user is already blacklisted.")
            .finish();
    }

    blacklist::create(&pool, new_blacklist).await?;
    cache::invalidate_blacklist(&pool, &redis_pool).await?;

    ApiResponse::ok().finish()
}

#[patch("/blacklist/{item}")]
pub async fn patch_blacklist_item(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    user: User,
    Path(item): Path<u64>,
    Json(new_blacklist): Json<EditBlacklist>,
) -> ApiResult<ApiResponse> {
    user.has_bot_admin(&redis_pool).await?;

    cache::get_blacklist_item(&redis_pool, item)
        .await?
        .or_not_found()?;
    new_blacklist.check()?;

    blacklist::update(&pool, item as i64, new_blacklist).await?;
    cache::invalidate_blacklist(&pool, &redis_pool).await?;

    ApiResponse::ok().finish()
}

#[delete("/blacklist/{item}")]
pub async fn delete_blacklist_item(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    user: User,
    Path(item): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_bot_admin(&redis_pool).await?;

    cache::get_blacklist_item(&redis_pool, item)
        .await?
        .or_not_found()?;

    blacklist::delete(&pool, item as i64).await?;
    cache::invalidate_blacklist(&pool, &redis_pool).await?;

    ApiResponse::ok().finish()
}
