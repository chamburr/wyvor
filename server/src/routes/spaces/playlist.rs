use crate::{
    auth::User,
    db::PgPool,
    error::ApiResult,
    models::Playlist,
    routes::{ApiResponse, ResultExt},
};

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json},
};
use actix_web_lab::extract::Path;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct NewPlaylistData {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistData {
    pub name: Option<String>,
}

#[get("/{id}/playlists")]
pub async fn get_space_playlists(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_read_space(&pool, id as i64).await?;

    let playlists = Playlist::filter_by_space(&pool, id as i64)
        .await?
        .iter()
        .map(|x| x.to_json(&["space"]))
        .collect::<Vec<Value>>();

    ApiResponse::ok().data(playlists).finish()
}

#[post("/{id}/playlists")]
pub async fn post_space_playlists(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
    Json(_data): Json<NewPlaylistData>,
) -> ApiResult<ApiResponse> {
    user.can_write_space(&pool, id as i64).await?;

    // TODO: create space playlist

    ApiResponse::ok().finish()
}

#[get("/{id}/playlists/{item}")]
pub async fn get_space_playlist(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
    Path(item): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_read_space(&pool, id as i64).await?;

    let playlist = Playlist::find(&pool, item as i64).await?.or_not_found()?;

    ApiResponse::ok()
        .data(playlist.to_json(&["space"]))
        .finish()
}

#[patch("/{id}/playlists/{item}")]
pub async fn patch_space_playlist(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
    Path(item): Path<u64>,
    Json(data): Json<PlaylistData>,
) -> ApiResult<ApiResponse> {
    user.can_write_space(&pool, id as i64).await?;

    let mut playlist = Playlist::find(&pool, item as i64).await?.or_not_found()?;

    if let Some(name) = data.name {
        playlist.set_name(name);
    }

    playlist.validate()?;
    playlist.update(&pool).await?;

    ApiResponse::ok().finish()
}

#[delete("/{id}/playlists/{item}")]
pub async fn delete_space_playlist(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
    Path(item): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_write_space(&pool, id as i64).await?;

    let playlist = Playlist::find(&pool, item as i64).await?.or_not_found()?;

    playlist.delete(&pool).await?;

    ApiResponse::ok().finish()
}
