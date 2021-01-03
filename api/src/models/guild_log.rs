use crate::constants::FETCH_LOGS_MAX;
use crate::db::schema::guild_log;
use crate::db::PgPool;
use crate::routes::ApiResult;

use actix_web::web::block;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "guild_log"]
pub struct GuildLog {
    pub id: i64,
    pub guild: i64,
    pub action: String,
    pub author: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "guild_log"]
pub struct NewGuildLog {
    pub guild: i64,
    pub action: String,
    pub author: i64,
}

pub async fn create(pool: &PgPool, new_guild_log: NewGuildLog) -> ApiResult<GuildLog> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<GuildLog> {
        let conn = pool.get()?;
        let res = diesel::insert_into(guild_log::table)
            .values(new_guild_log)
            .get_result(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find_by_guild(pool: &PgPool, guild: i64) -> ApiResult<Vec<GuildLog>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<GuildLog>> {
        let conn = pool.get()?;
        let res = guild_log::table
            .filter(guild_log::guild.eq(guild))
            .order(guild_log::created_at.desc())
            .limit(FETCH_LOGS_MAX as i64)
            .load(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn delete_by_guild(pool: &PgPool, id: i64) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res =
            diesel::delete(guild_log::table.filter(guild_log::guild.eq(id))).execute(&*conn)?;

        Ok(res)
    })
    .await?)
}
