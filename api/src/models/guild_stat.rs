use crate::constants::FETCH_STAT_DAYS;
use crate::db::schema::guild_stat;
use crate::db::PgPool;
use crate::routes::ApiResult;

use actix_web::web::block;
use chrono::NaiveDateTime;
use diesel::dsl::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "guild_stat"]
pub struct GuildStat {
    pub id: i64,
    pub guild: i64,
    pub author: i64,
    pub title: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "guild_stat"]
pub struct NewGuildStat {
    pub guild: i64,
    pub author: i64,
    pub title: String,
}

pub async fn create(pool: &PgPool, new_guild_stat: NewGuildStat) -> ApiResult<GuildStat> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<GuildStat> {
        let conn = pool.get()?;
        let res = diesel::insert_into(guild_stat::table)
            .values(new_guild_stat)
            .get_result(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find_by_guild(pool: &PgPool, guild: i64) -> ApiResult<Vec<GuildStat>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<GuildStat>> {
        let conn = pool.get()?;
        let res = guild_stat::table
            .filter(guild_stat::guild.eq(guild))
            .filter(guild_stat::created_at.gt(now - (FETCH_STAT_DAYS as i32).days()))
            .order(guild_stat::created_at.desc())
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
            diesel::delete(guild_stat::table.filter(guild_stat::guild.eq(id))).execute(&*conn)?;

        Ok(res)
    })
    .await?)
}
