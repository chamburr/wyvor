use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Guild {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub region: String,
    pub owner: i64,
    pub member_count: i32,
    pub roles: Vec<Role>,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub kind: i32,
    pub position: i32,
    pub parent: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub color: i32,
    pub position: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Member {
    pub id: i64,
    pub username: String,
    pub discriminator: i32,
    pub avatar: String,
    pub nickname: String,
    pub roles: Vec<i64>,
    pub joined_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct Permission {
    pub permission: i32,
}

#[derive(Debug, Deserialize)]
pub struct Connected {
    pub channel: i64,
    pub members: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct VoiceUpdate {
    pub session: String,
    pub guild: i64,
    pub endpoint: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct VoiceDestroy {
    pub guild: i64,
}
