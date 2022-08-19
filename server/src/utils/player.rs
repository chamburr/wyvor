use crate::{
    db::{cache, RedisPool},
    error::ApiResult,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Player {
    pub space: i64,
    pub player: PlayerState,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PlayerState {
    pub playing: i64,
    pub paused: bool,
    pub position: i64,
    pub looping: Loop,
    pub time: i64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Loop {
    #[default]
    None,
    Track,
    Queue,
}

impl Player {
    pub async fn new(redis_pool: &RedisPool, space: i64) -> ApiResult<Self> {
        let player = cache::get(redis_pool, format!("player:{}", space))
            .await?
            .unwrap_or_default();

        Ok(Self { space, player })
    }

    pub async fn update(&self, redis_pool: &RedisPool) -> ApiResult<()> {
        cache::set(redis_pool, format!("player:{}", self.space), &self.player)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, redis_pool: &RedisPool) -> ApiResult<()> {
        cache::del(redis_pool, format!("player:{}", self.space)).await?;

        Ok(())
    }
}
impl Player {
    pub fn playing(&self) -> i64{
        self.player.playing
    }

    pub fn paused(&self) -> bool {
        self.player.paused 
    }

    pub fn position(&self) -> i64 {
        self.player.position 
    }

    pub fn looping(&self) -> Loop {
        self.player.looping
    }
    
    pub fn time(&self) -> i64 {
        self.player.time
    }

    pub fn set_playing(&mut self, playing: i64) {
        self.player.playing = playing;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.player.paused = paused;
    }

    pub fn set_position(&mut self, position: i64) {
        self.player.position = position;
    }

    pub fn set_looping(&mut self, looping: Loop) {
        self.player.looping = looping;
    }

    pub fn set_time(&mut self, time: i64) {
        self.player.time = time;
    }
}