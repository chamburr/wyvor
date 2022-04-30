use crate::{config::CONFIG, ApiResult};

use chrono::Utc;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};

lazy_static! {
    static ref MUSIC_URI: String = CONFIG.music_uri();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Track {
    pub id: i64,
    pub name: String,
    pub artists: Vec<String>,
    pub album: String,
    pub image: String,
    pub length: i64,
}

impl Track {
    fn from_raw(data: &Value) -> Option<Self> {
        let fee = data["fee"].as_i64();

        if fee != Some(0) && fee != Some(8) {
            return None;
        }

        let id = data["id"].as_i64();
        let name = data["name"].as_str();
        let artists = data["ar"].as_array().map(|x| {
            x.iter()
                .filter_map(|y| y["name"].as_str())
                .collect::<Vec<_>>()
        });
        let album = data["al"]["name"].as_str();
        let image = data["al"]["picUrl"].as_str();
        let length = data["dt"].as_i64();

        if let (Some(id), Some(name), Some(artists), Some(album), Some(image), Some(length)) =
            (id, name, artists, album, image, length)
        {
            Some(Self {
                id,
                name: name.to_string(),
                artists: artists.into_iter().map(|x| x.to_string()).collect(),
                album: album.to_string(),
                image: image.to_string(),
                length,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Music API error")
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Client(reqwest::Client);

impl Client {
    async fn request(&self, path: &str, query: &[(&str, &str)]) -> ApiResult<Value> {
        let timestamp = Utc::now().timestamp_millis().to_string();

        let response = self
            .0
            .get(format!("{}{}", MUSIC_URI.as_str(), path))
            .query(query)
            .query(&[("realIP", "118.88.88.88")])
            .query(&[("timestamp", timestamp.as_str())])
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}

impl Client {
    pub fn new() -> Self {
        Self(reqwest::Client::new())
    }

    pub async fn search(&self, query: &str) -> ApiResult<Vec<Track>> {
        let query = query.to_lowercase();

        let response = self
            .request(
                "/cloudsearch",
                &[("keywords", query.as_str()), ("limit", "10")],
            )
            .await?;

        response["result"]["songs"]
            .as_array()
            .map(|x| x.iter().filter_map(Track::from_raw).collect())
            .ok_or_else(|| Error.into())
    }

    pub async fn track(&self, id: i64) -> ApiResult<Option<Track>> {
        Ok(self.tracks(&[id]).await?.pop())
    }

    pub async fn tracks(&self, ids: &[i64]) -> ApiResult<Vec<Track>> {
        let ids = ids
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let response = self
            .request("/song/detail", &[("ids", ids.as_str())])
            .await?;

        response["songs"]
            .as_array()
            .map(|x| x.iter().filter_map(Track::from_raw).collect())
            .ok_or_else(|| Error.into())
    }

    pub async fn url(&self, id: i64) -> ApiResult<Option<String>> {
        let response = self
            .request("/song/url", &[("id", id.to_string().as_str())])
            .await?;

        Ok(response["data"][0]["url"].as_str().map(|x| x.to_string()))
    }

    pub async fn lyrics(&self, id: i64) -> ApiResult<Option<String>> {
        let response = self
            .request("/lyric", &[("id", id.to_string().as_str())])
            .await?;

        Ok(response["lrc"]["lyric"].as_str().map(|x| {
            x.split('\n')
                .skip_while(|&y| y.contains(" : "))
                .map(|y| {
                    y.chars()
                        .into_iter()
                        .skip_while(|&z| z != ']')
                        .skip(1)
                        .collect::<String>()
                        + "\n"
                })
                .collect::<String>()
                .trim()
                .to_string()
        }))
    }
}
