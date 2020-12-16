use crate::constants::{PLAYLIST_NAME_MAX, PLAYLIST_NAME_MIN};
use crate::db::schema::playlist;
use crate::models::{UpdateExt, Validate, ValidateExt};
use crate::routes::ApiResult;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
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

pub fn create(conn: &PgConnection, new_playlist: &NewPlaylist) -> QueryResult<Playlist> {
    diesel::insert_into(playlist::table)
        .values(new_playlist)
        .get_result(conn)
}

pub fn update(conn: &PgConnection, id: i64, edit_playlist: &EditPlaylist) -> QueryResult<usize> {
    diesel::update(playlist::table.find(id))
        .set(edit_playlist)
        .execute(conn)
        .safely()
}

pub fn delete(conn: &PgConnection, id: i64) -> QueryResult<usize> {
    diesel::delete(playlist::table.find(id)).execute(conn)
}

pub fn delete_by_guild(conn: &PgConnection, id: i64) -> QueryResult<usize> {
    diesel::delete(playlist::table.filter(playlist::guild.eq(id))).execute(conn)
}

pub fn find(conn: &PgConnection, id: i64) -> QueryResult<Playlist> {
    playlist::table.find(id).first(conn)
}

pub fn find_by_guild(conn: &PgConnection, guild: i64) -> QueryResult<Vec<Playlist>> {
    playlist::table.filter(playlist::guild.eq(guild)).load(conn)
}
