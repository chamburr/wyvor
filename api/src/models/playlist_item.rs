use crate::db::schema::playlist_item;

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "playlist_item"]
pub struct PlaylistItem {
    pub id: i64,
    pub playlist: i64,
    pub track: String,
    pub title: String,
    pub uri: String,
    pub length: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "playlist_item"]
pub struct NewPlaylistItem {
    pub playlist: i64,
    pub track: String,
    pub title: String,
    pub uri: String,
    pub length: i32,
}

pub fn create(
    conn: &PgConnection,
    new_playlist_item: &NewPlaylistItem,
) -> QueryResult<PlaylistItem> {
    diesel::insert_into(playlist_item::table)
        .values(new_playlist_item)
        .get_result(conn)
}

pub fn delete_by_playlist(conn: &PgConnection, playlist: i64) -> QueryResult<usize> {
    diesel::delete(playlist_item::table.filter(playlist_item::playlist.eq(playlist))).execute(conn)
}

pub fn find_by_playlist(conn: &PgConnection, playlist: i64) -> QueryResult<Vec<PlaylistItem>> {
    playlist_item::table
        .filter(playlist_item::playlist.eq(playlist))
        .load(conn)
}
