use crate::db::schema::playlist_item;
use crate::db::PgPool;
use crate::routes::ApiResult;

use actix_web::web::block;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "playlist_item"]
pub struct PlaylistItem {
    pub id: i64,
    pub playlist: i64,
    pub track: String,
    pub title: String,
    pub uri: String,
    pub length: i32,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "playlist_item"]
pub struct NewPlaylistItem {
    pub playlist: i64,
    pub track: String,
    pub title: String,
    pub uri: String,
    pub length: i32,
}

pub async fn create(pool: &PgPool, new_playlist_item: NewPlaylistItem) -> ApiResult<PlaylistItem> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<PlaylistItem> {
        let conn = pool.get()?;
        let res = diesel::insert_into(playlist_item::table)
            .values(new_playlist_item)
            .get_result(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn delete_by_playlist(pool: &PgPool, playlist: i64) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::delete(playlist_item::table.filter(playlist_item::playlist.eq(playlist)))
            .execute(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find_by_playlist(pool: &PgPool, playlist: i64) -> ApiResult<Vec<PlaylistItem>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<PlaylistItem>> {
        let conn = pool.get()?;
        let res = playlist_item::table
            .filter(playlist_item::playlist.eq(playlist))
            .load(&*conn)?;

        Ok(res)
    })
    .await?)
}
