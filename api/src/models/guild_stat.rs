use crate::constants::FETCH_STAT_DAYS;
use crate::db::schema::guild_stat;

use chrono::NaiveDateTime;
use diesel::dsl::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "guild_stat"]
pub struct GuildStat {
    pub id: i64,
    pub guild: i64,
    pub author: i64,
    pub title: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "guild_stat"]
pub struct NewGuildStat {
    pub guild: i64,
    pub author: i64,
    pub title: String,
}

pub fn create(conn: &PgConnection, new_guild_stat: &NewGuildStat) -> QueryResult<GuildStat> {
    diesel::insert_into(guild_stat::table)
        .values(new_guild_stat)
        .get_result(conn)
}

pub fn find_by_guild(conn: &PgConnection, guild: i64) -> QueryResult<Vec<GuildStat>> {
    guild_stat::table
        .filter(guild_stat::guild.eq(guild))
        .filter(guild_stat::created_at.gt(now - (FETCH_STAT_DAYS as i32).days()))
        .order(guild_stat::created_at.desc())
        .load(conn)
}

pub fn delete_by_guild(conn: &PgConnection, id: i64) -> QueryResult<usize> {
    diesel::delete(guild_stat::table.filter(guild_stat::guild.eq(id))).execute(conn)
}
