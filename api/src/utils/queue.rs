use crate::constants::{QUEUE_KEY, QUEUE_LOOP_KEY, QUEUE_PLAYING_KEY};
use crate::db::cache;
use crate::db::RedisConn;
use crate::models::account::Account;
use crate::models::playlist_item::PlaylistItem;
use crate::routes::ApiResult;

use dashmap::mapref::one::Ref;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rocket_contrib::databases::redis::RedisResult;
use serde::{Deserialize, Serialize};
use twilight_andesite::http::Track;
use twilight_andesite::model::Play;
use twilight_andesite::player::Player;
use twilight_model::id::GuildId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueItem {
    pub track: String,
    pub title: String,
    pub uri: String,
    pub length: i32,
    pub author: i64,
    pub username: String,
    pub discriminator: i32,
}

impl From<(Track, Account)> for QueueItem {
    fn from((track, author): (Track, Account)) -> Self {
        Self {
            track: track.track,
            title: track.info.title,
            uri: track.info.uri,
            length: track.info.length as i32,
            author: author.id,
            username: author.username,
            discriminator: author.discriminator,
        }
    }
}

impl From<(PlaylistItem, Account)> for QueueItem {
    fn from((track, author): (PlaylistItem, Account)) -> Self {
        Self {
            track: track.track,
            title: track.title,
            uri: track.uri,
            length: track.length,
            author: author.id,
            username: author.username,
            discriminator: author.discriminator,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Loop {
    None,
    Track,
    Queue,
}

pub struct Queue<'a> {
    pub conn: &'a RedisConn,
    pub guild: u64,
}

impl<'a> Queue<'a> {
    pub fn from(conn: &'a RedisConn, guild: u64) -> Self {
        Self { conn, guild }
    }

    fn get_queue_key(&self) -> String {
        format!("{}{}", QUEUE_KEY, self.guild)
    }

    fn get_playing_key(&self) -> String {
        format!("{}{}", QUEUE_PLAYING_KEY, self.guild)
    }

    fn get_loop_key(&self) -> String {
        format!("{}{}", QUEUE_LOOP_KEY, self.guild)
    }

    pub fn get(&self) -> ApiResult<Vec<QueueItem>> {
        let queue = cache::get(self.conn, self.get_queue_key().as_str())?.unwrap_or_default();
        Ok(queue)
    }

    pub fn get_track(&self, index: u32) -> ApiResult<Option<QueueItem>> {
        let track = self.get()?.get(index as usize).cloned();
        Ok(track)
    }

    pub fn set(&self, queue: Vec<QueueItem>) -> RedisResult<()> {
        cache::set(self.conn, self.get_queue_key().as_str(), &queue)
    }

    pub fn len(&self) -> ApiResult<usize> {
        Ok(self.get()?.len())
    }

    pub fn get_playing(&self) -> ApiResult<i32> {
        let playing = cache::get(self.conn, self.get_playing_key().as_str())?.unwrap_or(-1);
        Ok(playing)
    }

    pub fn get_playing_track(&self) -> ApiResult<Option<QueueItem>> {
        Ok(self.get_track(self.get_playing()? as u32)?)
    }

    pub fn set_playing(&self, playing: i32) -> RedisResult<()> {
        cache::set(self.conn, self.get_playing_key().as_str(), &playing)
    }

    pub fn get_loop(&self) -> ApiResult<Loop> {
        let looping = cache::get(self.conn, self.get_loop_key().as_str())?.unwrap_or(Loop::None);
        Ok(looping)
    }

    pub fn set_loop(&self, kind: Loop) -> RedisResult<()> {
        cache::set(self.conn, self.get_loop_key().as_str(), &kind)
    }

    pub fn add(&self, item: &QueueItem) -> ApiResult<()> {
        let mut queue = self.get()?;
        queue.push(item.clone());
        self.set(queue)?;

        Ok(())
    }

    pub fn remove(&self, index: u32) -> ApiResult<QueueItem> {
        let mut queue = self.get()?;
        let playing = self.get_playing()?;

        let removed = queue.remove(index as usize);
        self.set(queue)?;

        if (index as i32) < playing {
            self.set_playing(playing - 1)?;
        }

        Ok(removed)
    }

    pub fn shift(&self, index: u32, position: u32) -> ApiResult<()> {
        let mut queue = self.get()?;
        let playing = self.get_playing()?;

        let item = queue.remove(index as usize);
        queue.insert(position as usize, item);
        self.set(queue)?;

        if (index as i32) < playing && (position as i32) > playing {
            self.set_playing(playing - 1)?;
        } else if (index as i32) > playing && (position as i32) < playing {
            self.set_playing(playing + 1)?;
        }

        Ok(())
    }

    pub fn next(&self) -> ApiResult<()> {
        let queue = self.get()?;
        let playing = self.get_playing()?;
        let looping = self.get_loop()?;

        let new_playing = match looping {
            Loop::None if queue.len() <= (playing + 1) as usize => -1,
            Loop::Queue if queue.len() <= (playing + 1) as usize => 0,
            Loop::None | Loop::Queue => playing + 1,
            Loop::Track => playing,
        };

        if playing != new_playing {
            self.set_playing(new_playing)?;
        }

        Ok(())
    }

    pub fn shuffle(&self) -> ApiResult<()> {
        let mut queue = self.get()?;
        let playing = self.get_playing()?;
        let mut new_queue = vec![];
        let mut rng = thread_rng();

        if playing >= 0 {
            let playing = playing as usize;
            let mut queue_before = queue[..playing].to_vec();
            let mut queue_after = queue[(playing + 1)..].to_vec();

            queue_before.shuffle(&mut rng);
            queue_after.shuffle(&mut rng);
            new_queue.append(&mut queue_before);
            new_queue.push(queue[playing].clone());
            new_queue.append(&mut queue_after);
        } else {
            queue.shuffle(&mut rng);
            new_queue.append(&mut queue);
        }

        self.set(new_queue)?;

        Ok(())
    }

    pub fn delete(&self) -> ApiResult<Vec<QueueItem>> {
        let tracks = self.get()?;

        cache::del(self.conn, self.get_playing_key().as_str())?;
        cache::del(self.conn, self.get_queue_key().as_str())?;
        cache::del(self.conn, self.get_loop_key().as_str())?;

        Ok(tracks)
    }

    pub fn play_with(&self, player: &Ref<'_, GuildId, Player>) -> ApiResult<()> {
        if let Some(track) = self.get_playing_track()? {
            let guild = player.guild_id();
            player.send(Play::new(guild, track.track))?;
        }

        Ok(())
    }
}
