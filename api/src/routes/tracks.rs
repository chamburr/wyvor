use crate::db::RedisConn;
use crate::routes::{ApiResponse, ApiResult, OptionExt};
use crate::utils::auth::User;
use crate::utils::{html_unescape, player};

use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use twilight_andesite::http::{LoadType, Track};

#[derive(Debug, Serialize)]
pub struct SimpleTrack {
    pub track: String,
    pub title: String,
    pub uri: String,
    pub length: i32,
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
    fn from_id(id: String) -> ApiResult<Self> {
        let track = player::decode_track(&id);
        if let Ok(track) = track {
            Ok(Self::from(track))
        } else {
            Err(ApiResponse::not_found())
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Lyrics {
    title: String,
    content: String,
}

#[get("/?<query>")]
pub fn get_tracks(_redis_conn: RedisConn, _user: User, query: Option<String>) -> ApiResponse {
    let query = query.into_bad_request()?;
    let tracks = player::get_track(query.as_str())?;

    if tracks.load_type == LoadType::LoadFailed {
        if let Some(cause) = tracks.cause {
            if let Some(message) = cause.message {
                return ApiResponse::bad_request().message(message.as_str());
            }
        }

        return ApiResponse::internal_server_error();
    }

    if tracks.tracks.is_none() {
        return ApiResponse::ok().data(Vec::<()>::new());
    }

    let tracks: Vec<SimpleTrack> = tracks
        .tracks
        .unwrap()
        .iter()
        .map(|track| SimpleTrack::from(track.clone()))
        .collect();

    ApiResponse::ok().data(tracks)
}

#[get("/track?<id>")]
pub fn get_track(_redis_conn: RedisConn, _user: User, id: Option<String>) -> ApiResponse {
    let id = id.into_bad_request()?;
    let track = SimpleTrack::from_id(id)?;

    ApiResponse::ok().data(track)
}

#[get("/lyrics?<id>")]
pub fn get_track_lyrics(_redis_conn: RedisConn, _user: User, id: Option<String>) -> ApiResponse {
    let id = id.into_bad_request()?;
    let track = SimpleTrack::from_id(id)?;

    let query = percent_encode(track.title.as_bytes(), NON_ALPHANUMERIC);
    let lyrics: Value = Client::new()
        .get(format!("https://lyrics.tsu.sh/v1/?q={}", query).as_str())
        .send()?
        .json()?;

    let lyrics = Lyrics {
        title: lyrics["song"]["title"]
            .as_str()
            .ok_or_else(ApiResponse::not_found)?
            .to_owned(),
        content: html_unescape(
            lyrics["content"]
                .as_str()
                .ok_or_else(ApiResponse::not_found)?,
        ),
    };

    ApiResponse::ok().data(lyrics)
}
