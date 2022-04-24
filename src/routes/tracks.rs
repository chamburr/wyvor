use crate::{
    auth::User,
    music::Client,
    routes::{ApiResponse, ApiResult, ResultExt},
};

use actix_web::{get, web::Query};
use actix_web_lab::extract::Path;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[get("")]
pub async fn get_tracks(
    _user: User,
    Query(mut query): Query<HashMap<String, String>>,
) -> ApiResult<ApiResponse> {
    ApiResponse::ok().finish()
}

#[get("/{id}")]
pub async fn get_track(_user: User, Path(id): Path<u64>) -> ApiResult<ApiResponse> {
    let track = Client::new().track(id as i64).await?.or_not_found()?;

    ApiResponse::ok().data(track).finish()
}

#[get("/{id}/lyrics")]
pub async fn get_track_lyrics(_user: User, Path(id): Path<u64>) -> ApiResult<ApiResponse> {
    let lyrics = Client::new().lyrics(id as i64).await?.or_not_found()?;

    ApiResponse::ok().data(json!({ "lyrics": lyrics })).finish()
}
