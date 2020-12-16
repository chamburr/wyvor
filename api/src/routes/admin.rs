use crate::db::{cache, PgConn, RedisConn};
use crate::models::blacklist::{self, EditBlacklist, NewBlacklist};
use crate::models::guild::{self, Guild};
use crate::models::{account, Validate};
use crate::routes::{ApiResponse, OptionExt};
use crate::utils::auth::User;

use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleBlacklist {
    pub reason: String,
}

#[get("/guilds?<name>&<owner>")]
pub fn get_guilds(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    name: Option<String>,
    owner: Option<u64>,
) -> ApiResponse {
    user.has_bot_admin(&redis_conn)?;

    if name.is_none() && owner.is_none() {
        return ApiResponse::bad_request();
    }

    let mut guilds = vec![];

    if let Some(name) = name {
        guilds.push(guild::find_by_name(&*conn, name.as_str())?);
    }

    if let Some(owner) = owner {
        guilds.push(guild::find_by_owner(&*conn, owner as i64)?);
    }

    let all_guilds: HashSet<Guild> = guilds.iter().flat_map(|guild| guild.clone()).collect();
    let all_guilds: Vec<Guild> = all_guilds
        .iter()
        .filter(|guild| guilds.iter().all(|chunk| chunk.contains(*guild)))
        .cloned()
        .collect();

    ApiResponse::ok().data(all_guilds)
}

#[get("/top_guilds?<amount>")]
pub fn get_top_guilds(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    amount: Option<i64>,
) -> ApiResponse {
    user.has_bot_admin(&redis_conn)?;

    let amount = amount.into_bad_request()?;
    let guilds = guild::find_by_member_count(&*conn, amount)?;

    ApiResponse::ok().data(guilds)
}

#[get("/blacklist")]
pub fn get_blacklist(conn: PgConn, redis_conn: RedisConn, user: User) -> ApiResponse {
    user.has_bot_admin(&redis_conn)?;

    let blacklist = blacklist::all(&*conn)?;

    ApiResponse::ok().data(blacklist)
}

#[put("/blacklist/<item>", data = "<new_blacklist>")]
pub fn put_blacklist_item(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    item: u64,
    new_blacklist: Json<SimpleBlacklist>,
) -> ApiResponse {
    user.has_bot_admin(&redis_conn)?;

    account::find(&*conn, item as i64)?;

    if cache::get_blacklist_item(&redis_conn, item)?.is_some() {
        return ApiResponse::bad_request().message("The user is already blacklisted.");
    }

    let new_blacklist = NewBlacklist {
        id: item as i64,
        reason: new_blacklist.into_inner().reason,
        author: user.user.id,
    };

    new_blacklist.check()?;

    blacklist::create(&*conn, &new_blacklist)?;
    cache::invalidate_blacklist(&*conn, &redis_conn)?;

    ApiResponse::ok()
}

#[patch("/blacklist/<item>", data = "<new_blacklist>")]
pub fn patch_blacklist_item(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    item: u64,
    new_blacklist: Json<EditBlacklist>,
) -> ApiResponse {
    user.has_bot_admin(&redis_conn)?;

    cache::get_blacklist_item(&redis_conn, item)?.into_not_found()?;

    let new_blacklist = new_blacklist.into_inner();
    new_blacklist.check()?;

    blacklist::update(&*conn, item as i64, &new_blacklist)?;
    cache::invalidate_blacklist(&*conn, &redis_conn)?;

    ApiResponse::ok()
}

#[delete("/blacklist/<item>")]
pub fn delete_blacklist_item(
    conn: PgConn,
    redis_conn: RedisConn,
    user: User,
    item: u64,
) -> ApiResponse {
    user.has_bot_admin(&redis_conn)?;

    cache::get_blacklist_item(&redis_conn, item)?.into_not_found()?;

    blacklist::delete(&*conn, item as i64)?;
    cache::invalidate_blacklist(&*conn, &redis_conn)?;

    ApiResponse::ok()
}
