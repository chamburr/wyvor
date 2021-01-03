use crate::constants::{queue_key, queue_loop_key, queue_playing_key};
use crate::db::{cache, RedisPool};
use crate::models::account::Account;
use crate::models::playlist_item::PlaylistItem;
use crate::routes::ApiResult;
use crate::utils::player;

use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use twilight_andesite::http::Track;
use twilight_andesite::model::Play;
use twilight_model::id::GuildId;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Loop {
    None,
    Track,
    Queue,
}

pub async fn get(pool: &RedisPool, guild: u64) -> ApiResult<Vec<QueueItem>> {
    let queue = cache::get(pool, queue_key(guild))
        .await?
        .unwrap_or_default();

    Ok(queue)
}

pub async fn get_track(pool: &RedisPool, guild: u64, index: i32) -> ApiResult<Option<QueueItem>> {
    if index < 0 {
        return Ok(None);
    }

    let mut tracks = get(pool, guild).await?;
    let track = if (index as usize) < tracks.len() {
        Some(tracks.remove(index as usize))
    } else {
        None
    };

    Ok(track)
}

pub async fn set(pool: &RedisPool, guild: u64, queue: Vec<QueueItem>) -> ApiResult<()> {
    cache::set(pool, queue_key(guild), &queue).await?;

    Ok(())
}

pub async fn len(pool: &RedisPool, guild: u64) -> ApiResult<usize> {
    let len = get(pool, guild).await?.len();

    Ok(len)
}

pub async fn get_playing(pool: &RedisPool, guild: u64) -> ApiResult<i32> {
    let playing = cache::get(pool, queue_playing_key(guild))
        .await?
        .unwrap_or(-1);

    Ok(playing)
}

pub async fn get_playing_track(pool: &RedisPool, guild: u64) -> ApiResult<Option<QueueItem>> {
    let playing = get_playing(pool, guild).await?;
    let track = get_track(pool, guild, playing).await?;

    Ok(track)
}

pub async fn set_playing(pool: &RedisPool, guild: u64, playing: i32) -> ApiResult<()> {
    cache::set(pool, queue_playing_key(guild), &playing).await?;

    Ok(())
}

pub async fn get_loop(pool: &RedisPool, guild: u64) -> ApiResult<Loop> {
    let looping = cache::get(pool, queue_loop_key(guild))
        .await?
        .unwrap_or(Loop::None);

    Ok(looping)
}

pub async fn set_loop(pool: &RedisPool, guild: u64, kind: &Loop) -> ApiResult<()> {
    cache::set(pool, queue_loop_key(guild), kind).await?;

    Ok(())
}

pub async fn add(pool: &RedisPool, guild: u64, item: QueueItem) -> ApiResult<()> {
    let mut queue = get(pool, guild).await?;
    queue.push(item);
    set(pool, guild, queue).await?;

    Ok(())
}

pub async fn remove(pool: &RedisPool, guild: u64, index: u32) -> ApiResult<QueueItem> {
    let mut queue = get(pool, guild).await?;
    let playing = get_playing(pool, guild).await?;

    let removed = queue.remove(index as usize);
    set(pool, guild, queue).await?;

    if (index as i32) < playing {
        set_playing(pool, guild, playing - 1).await?;
    }

    Ok(removed)
}

pub async fn shift(pool: &RedisPool, guild: u64, index: u32, position: u32) -> ApiResult<()> {
    let mut queue = get(pool, guild).await?;
    let playing = get_playing(pool, guild).await?;

    let item = queue.remove(index as usize);
    queue.insert(position as usize, item);
    set(pool, guild, queue).await?;

    let new_playing = if (index as i32) < playing && (position as i32) > playing {
        playing - 1
    } else if (index as i32) > playing && (position as i32) < playing {
        playing + 1
    } else {
        playing
    };

    if playing != new_playing {
        set_playing(pool, guild, new_playing).await?;
    }

    Ok(())
}

pub async fn next(pool: &RedisPool, guild: u64) -> ApiResult<()> {
    let tracks = get(pool, guild).await?;
    let playing = get_playing(pool, guild).await?;
    let looping = get_loop(pool, guild).await?;

    let new_playing = match looping {
        Loop::None if tracks.len() <= (playing + 1) as usize => -1,
        Loop::Queue if tracks.len() <= (playing + 1) as usize => 0,
        Loop::None | Loop::Queue => playing + 1,
        Loop::Track => playing,
    };

    if playing != new_playing {
        set_playing(pool, guild, new_playing).await?;
    }

    Ok(())
}

pub async fn shuffle(pool: &RedisPool, guild: u64) -> ApiResult<()> {
    let mut queue = get(pool, guild).await?;
    let playing = get_playing(pool, guild).await?;

    let mut new_queue = vec![];
    let mut rng = thread_rng();

    if playing >= 0 {
        let playing = playing as usize;
        let mut queue_before: Vec<QueueItem> = queue.drain(..playing).collect();
        let playing_item = queue.remove(0);
        let mut queue_after = queue;

        queue_before.shuffle(&mut rng);
        queue_after.shuffle(&mut rng);
        new_queue.extend(queue_before);
        new_queue.push(playing_item);
        new_queue.extend(queue_after);
    } else {
        queue.shuffle(&mut rng);
        new_queue.append(&mut queue);
    }

    set(pool, guild, new_queue).await?;

    Ok(())
}

pub async fn delete(pool: &RedisPool, guild: u64) -> ApiResult<Vec<QueueItem>> {
    let tracks = get(pool, guild).await?;

    cache::del(pool, queue_key(guild)).await?;
    cache::del(pool, queue_playing_key(guild)).await?;
    cache::del(pool, queue_loop_key(guild)).await?;

    Ok(tracks)
}

pub async fn play(pool: &RedisPool, guild: u64) -> ApiResult<()> {
    if let Some(track) = get_playing_track(pool, guild).await? {
        player::send(Play::new(GuildId(guild), track.track)).await?;
    }

    Ok(())
}

pub async fn play_next(pool: &RedisPool, guild: u64) -> ApiResult<()> {
    next(pool, guild).await?;
    play(pool, guild).await?;

    Ok(())
}
