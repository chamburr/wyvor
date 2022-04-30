use crate::{
    auth::User,
    db::PgPool,
    error::ApiResult,
    mail::Client,
    models::{Account, Member, MemberRole, Space, Playlist},
    routes::{ApiResponse, ResultExt},
    spaces::NewPlaylistData,
};

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json},
};
use actix_web_lab::extract::Path;
use handlebars::JsonValue;

#[get("/{id}/playlists")]
pub async fn get_playlists(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_read_space(&pool, id as i64).await?; 
    let playlists = Playlist::filter_by_space(&pool, id as i64)
    .await?
    .iter()
    .map(|x| x.to_json(&[]))
    .collect::<ApiResult<Vec<JsonValue>>>()?;
    ApiResponse::ok().data(playlists).finish()
}

#[get("/{id}/playlists/{item}")]
pub async fn get_item(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
    Path(item): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_read_space(&pool, id as i64).await?;
    let playlist = Playlist::find(&pool, item as i64).await?.or_not_found()?;
    ApiResponse::ok().data(playlist.to_json(&["space"])?).finish()
}

#[patch("/{id}/playlists/{item}")]
pub async fn patch_playlist(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
    Path(item): Path<u64>,
    Json(data): Json<NewPlaylistData>,
) -> ApiResult<ApiResponse> {
    user.can_write_space(&pool, id as i64).await?;
    let mut playlist = Playlist::find(&pool, item as i64).await?.or_not_found()?;
    playlist.set_name(data.name);
    playlist.validate()?;
    playlist.update(&pool).await?;
    

    ApiResponse::ok().finish()
}
#[delete("/{id}/playlists/{item}")]
pub async fn delete_playlist(
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
