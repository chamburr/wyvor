use crate::routes::{ApiResponse, ApiResult, OptionExt, ResultExt};
use crate::utils::auth::User;
use crate::utils::{html_unescape, player};

use actix_web::get;
use actix_web::web::Query;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use twilight_andesite::http::{LoadType, Track};

#[derive(Debug, Serialize)]
pub struct SimpleTrack {
    pub track: String,
    pub title: String,
    pub uri: String,
    pub length: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleLyrics {
    pub title: String,
    pub content: String,
}

impl From<Track> for SimpleTrack {
    fn from(track: Track) -> Self {
        Self {
            track: track.track,
            title: track.info.title,
            uri: track.info.uri,
            length: track.info.length as i32,
        }
    }
}

impl SimpleTrack {
    async fn from_id(id: String) -> ApiResult<Self> {
        let track = player::decode_track(&id).await.or_not_found()?;

        Ok(Self::from(track))
    }
}

#[get("")]
pub async fn get_tracks(
    _user: User,
    Query(mut query): Query<HashMap<String, String>>,
) -> ApiResult<ApiResponse> {
    let query = query.remove("query").or_bad_request()?;
    let tracks = player::get_track(query.as_str()).await?;

    if tracks.load_type == LoadType::LoadFailed {
        if let Some(cause) = tracks.cause {
            if let Some(message) = cause.message {
                return ApiResponse::bad_request()
                    .message(message.as_str())
                    .finish();
            }
        }

        return ApiResponse::internal_server_error().finish();
    }

    if let Some(tracks) = tracks.tracks {
        let tracks: Vec<SimpleTrack> = tracks
            .into_iter()
            .map(|track| SimpleTrack {
                track: track.track,
                title: track.info.title,
                uri: track.info.uri,
                length: track.info.length as i32,
            })
            .collect();

        ApiResponse::ok().data(tracks).finish()
    } else {
        ApiResponse::ok().data(Vec::<()>::new()).finish()
    }
}

#[get("/track")]
pub async fn get_track(
    _user: User,
    Query(mut query): Query<HashMap<String, String>>,
) -> ApiResult<ApiResponse> {
    let id = query.remove("id").or_bad_request()?;
    let track = SimpleTrack::from_id(id).await?;

    ApiResponse::ok().data(track).finish()
}

#[get("/lyrics")]
pub async fn get_track_lyrics(
    _user: User,
    Query(mut query): Query<HashMap<String, String>>,
) -> ApiResult<ApiResponse> {
    let id = query.remove("id").or_bad_request()?;
    let track = SimpleTrack::from_id(id).await?;

    let query = percent_encode(track.title.as_bytes(), NON_ALPHANUMERIC);
    let lyrics: Value = reqwest::Client::new()
        .get(format!("https://lyrics.tsu.sh/v1/?q={}", query).as_str())
        .send()
        .await?
        .json()
        .await?;

    let lyrics = SimpleLyrics {
        title: lyrics["song"]["title"].as_str().or_not_found()?.to_owned(),
        content: html_unescape(lyrics["content"].as_str().or_not_found()?),
    };

    ApiResponse::ok().data(lyrics).finish()
}
