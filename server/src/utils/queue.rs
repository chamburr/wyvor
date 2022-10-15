use crate::{
    db::{cache, RedisPool},
    error::ApiResult,
    utils::music::Track,
};

use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};


use actix::prelude::*;
#[derive(Message)]
#[rtype(result = "()")]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Queue {
    pub space: i64,
    pub tracks: Vec<Track>,
}

impl Queue {
    pub async fn new(redis_pool: &RedisPool, space: i64) -> ApiResult<Self> {
        let tracks = cache::get(redis_pool, format!("queue:{}", space))
            .await?
            .unwrap_or_default();

        Ok(Self { space, tracks })
    }

    pub async fn update(&self, redis_pool: &RedisPool) -> ApiResult<()> {
        cache::set(redis_pool, format!("queue:{}", self.space), &self.tracks)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, redis_pool: &RedisPool) -> ApiResult<()> {
        cache::del(redis_pool, format!("queue:{}", self.space)).await?;

        Ok(())
    }
}

impl Queue {
    pub fn get(&self) -> &[Track] {
        self.tracks.as_slice()
    }

    pub fn get_index(&self, index: u32) -> Option<&Track> {
        self.tracks.get(index as usize)
    }

    pub fn get_length(&self) -> usize {
        self.tracks.len()
    }

    pub fn add(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn insert(&mut self, index: usize, track: Track) {
        self.tracks.insert(index, track);
    }

    pub fn remove(&mut self, index: u32) -> Track {
        self.tracks.remove(index as usize)
    }

    pub fn shift(&mut self, index: u32, position: u32) {
        let item = self.tracks.remove(index as usize);
        self.tracks.insert(position as usize, item);
    }

    pub fn shuffle(&mut self) {
        self.tracks.shuffle(&mut thread_rng());
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }
}
