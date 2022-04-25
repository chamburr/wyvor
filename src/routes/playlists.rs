use crate::{
    auth::User,
    db::PgPool,
    error::ApiResult,
    mail::Client,
    models::{Account, Member, MemberRole, Space, Playlist},
    routes::{ApiResponse, ResultExt},
};
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json},
};


#[get("/{id}/playlists")]
pub async fn get_playlist(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_read_space(&pool, id as i64).await?;
    let playlists = Playlist::filter_by_space(&pool, id as i64).await?;

    ApiResponse::ok().data(playlists.to_json(&["space"])?).finish()
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
    new_name: String,
) -> ApiResult<ApiResponse> {
    user.can_write_space(&pool, id as i64).await?;
    let mut playlist = Playlist::find(&pool, item as i64).await?.or_internal_server_error()?;
    playlist.set_name(new_name);
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
    user.can_delete_space(&pool, id as i64).await?;
    let mut playlist = Playlist::find(&pool, item as i64).await?.or_internal_server_error()?;
    playlist.delete(&pool).await?;
    

    ApiResponse::ok().finish()
}