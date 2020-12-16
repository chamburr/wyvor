use crate::constants::FETCH_LOGS_MAX;
use crate::db::schema::guild_log;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "guild_log"]
pub struct GuildLog {
    pub id: i64,
    pub guild: i64,
    pub action: String,
    pub author: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "guild_log"]
pub struct NewGuildLog {
    pub guild: i64,
    pub action: String,
    pub author: i64,
}

pub fn create(conn: &PgConnection, new_guild_log: &NewGuildLog) -> QueryResult<GuildLog> {
    diesel::insert_into(guild_log::table)
        .values(new_guild_log)
        .get_result(conn)
}

pub fn find_by_guild(conn: &PgConnection, guild: i64) -> QueryResult<Vec<GuildLog>> {
    guild_log::table
        .filter(guild_log::guild.eq(guild))
        .order(guild_log::created_at.desc())
        .limit(FETCH_LOGS_MAX as i64)
        .load(conn)
}

pub fn delete_by_guild(conn: &PgConnection, id: i64) -> QueryResult<usize> {
    diesel::delete(guild_log::table.filter(guild_log::guild.eq(id))).execute(conn)
}
