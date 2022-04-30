use crate::{
    auth::User,
    routes::{ApiResponse, ApiResult, ResultExt},
    utils::music::Client,
};

use actix_web::get;
use actix_web::web::Query;
use actix_web_lab::extract::Path;
use serde_json::json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchData {
    pub query: String,
}

#[get("/search")]
pub async fn get_tracks(
    _user: User,
    Query(mut query): Query<SearchData>,
) -> ApiResult<ApiResponse> {
    let tracks = Client::new().search(query.query.as_str()).await?;

    ApiResponse::ok().data(json!({ "tracks": tracks })).finish()
}

#[get("/{id}")]
pub async fn get_track(
    _user: User,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    let track = Client::new().track(id as i64).await?.or_not_found()?;

    ApiResponse::ok().data(track).finish()
}

#[get("/{id}/lyrics")]
pub async fn get_track_lyrics(
    _user: User,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    let lyrics = Client::new().lyrics(id as i64).await?.or_not_found()?;

    ApiResponse::ok().data(json!({ "lyrics": lyrics })).finish()
}
