use crate::constants::{PLAYLIST_NAME_MAX, PLAYLIST_NAME_MIN};
use crate::db::schema::playlist;
use crate::db::PgPool;
use crate::models::{Validate, ValidateExt};
use crate::routes::ApiResult;

use actix_web::web::block;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error::QueryBuilderError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "playlist"]
pub struct Playlist {
    pub id: i64,
    pub guild: i64,
    pub name: String,
    pub author: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "playlist"]
pub struct NewPlaylist {
    pub guild: i64,
    pub name: String,
    pub author: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, AsChangeset)]
#[table_name = "playlist"]
pub struct EditPlaylist {
    pub name: Option<String>,
}

impl Validate for NewPlaylist {
    fn check(&self) -> ApiResult<()> {
        self.name
            .len()
            .check_btw(PLAYLIST_NAME_MIN, PLAYLIST_NAME_MAX, "length of name")?;

        Ok(())
    }
}

impl Validate for EditPlaylist {
    fn check(&self) -> ApiResult<()> {
        if let Some(name) = &self.name {
            name.len()
                .check_btw(PLAYLIST_NAME_MIN, PLAYLIST_NAME_MAX, "length of name")?;
        }

        Ok(())
    }
}

pub async fn create(pool: &PgPool, new_playlist: NewPlaylist) -> ApiResult<Playlist> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Playlist> {
        let conn = pool.get()?;
        let res = diesel::insert_into(playlist::table)
            .values(new_playlist)
            .get_result(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn update(pool: &PgPool, id: i64, edit_playlist: EditPlaylist) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::update(playlist::table.find(id))
            .set(edit_playlist)
            .execute(&*conn)
            .or_else(|err| {
                if let QueryBuilderError(_) = err {
                    Ok(0)
                } else {
                    Err(err)
                }
            })?;

        Ok(res)
    })
    .await?)
}

pub async fn delete(pool: &PgPool, id: i64) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::delete(playlist::table.find(id)).execute(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn delete_by_guild(pool: &PgPool, id: i64) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::delete(playlist::table.filter(playlist::guild.eq(id))).execute(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find(pool: &PgPool, id: i64) -> ApiResult<Option<Playlist>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Option<Playlist>> {
        let conn = pool.get()?;
        let res = playlist::table.find(id).first(&*conn).optional()?;

        Ok(res)
    })
    .await?)
}

pub async fn find_by_guild(pool: &PgPool, guild: i64) -> ApiResult<Vec<Playlist>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<Playlist>> {
        let conn = pool.get()?;
        let res = playlist::table
            .filter(playlist::guild.eq(guild))
            .load(&*conn)?;

        Ok(res)
    })
    .await?)
}
