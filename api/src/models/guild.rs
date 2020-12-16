use crate::db::schema::guild;

use chrono::NaiveDateTime;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "guild"]
pub struct Guild {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub owner: i64,
    pub member_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "guild"]
pub struct NewGuild {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub owner: i64,
    pub member_count: i32,
}

pub fn batch_create(conn: &PgConnection, guilds: &[NewGuild]) -> QueryResult<usize> {
    diesel::insert_into(guild::table)
        .values(guilds)
        .on_conflict(guild::id)
        .do_update()
        .set((
            guild::name.eq(excluded(guild::name)),
            guild::icon.eq(excluded(guild::icon)),
            guild::owner.eq(excluded(guild::owner)),
            guild::member_count.eq(excluded(guild::member_count)),
        ))
        .execute(conn)
}

pub fn find_by_name(conn: &PgConnection, name: &str) -> QueryResult<Vec<Guild>> {
    guild::table
        .filter(guild::name.ilike(format!("%{}%", name)))
        .load(conn)
}

pub fn find_by_owner(conn: &PgConnection, owner: i64) -> QueryResult<Vec<Guild>> {
    guild::table.filter(guild::owner.eq(owner)).load(conn)
}

pub fn find_by_member_count(conn: &PgConnection, amount: i64) -> QueryResult<Vec<Guild>> {
    guild::table
        .order(guild::member_count.desc())
        .limit(amount)
        .load(conn)
}
