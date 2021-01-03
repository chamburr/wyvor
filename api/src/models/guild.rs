use crate::db::schema::guild;
use crate::db::PgPool;
use crate::routes::ApiResult;

use actix_web::web::block;
use chrono::NaiveDateTime;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Queryable, Identifiable)]
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

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "guild"]
pub struct NewGuild {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub owner: i64,
    pub member_count: i32,
}

pub async fn batch_create(pool: &PgPool, guilds: Vec<NewGuild>) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::insert_into(guild::table)
            .values(guilds)
            .on_conflict(guild::id)
            .do_update()
            .set((
                guild::name.eq(excluded(guild::name)),
                guild::icon.eq(excluded(guild::icon)),
                guild::owner.eq(excluded(guild::owner)),
                guild::member_count.eq(excluded(guild::member_count)),
            ))
            .execute(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find_by_name(pool: &PgPool, name: String) -> ApiResult<Vec<Guild>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<Guild>> {
        let conn = pool.get()?;
        let res = guild::table
            .filter(guild::name.ilike(format!("%{}%", name)))
            .load(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find_by_owner(pool: &PgPool, owner: i64) -> ApiResult<Vec<Guild>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<Guild>> {
        let conn = pool.get()?;
        let res = guild::table.filter(guild::owner.eq(owner)).load(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find_by_member_count(pool: &PgPool, amount: i64) -> ApiResult<Vec<Guild>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<Guild>> {
        let conn = pool.get()?;
        let res = guild::table
            .order(guild::member_count.desc())
            .limit(amount)
            .load(&*conn)?;

        Ok(res)
    })
    .await?)
}
