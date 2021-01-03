use crate::db::pubsub::Message;
use crate::db::{cache, PgPool, RedisPool};
use crate::models::config::EditConfig;
use crate::models::guild_log::{self, NewGuildLog};
use crate::models::playlist::{EditPlaylist, NewPlaylist, Playlist};
use crate::routes::guilds::{SimplePlayer, SimplePosition};
use crate::routes::ApiResult;
use crate::utils::auth::User;
use crate::utils::queue::{self, QueueItem};
use crate::utils::{format_duration, format_track};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use twilight_andesite::model::{TrackException, TrackStuck, WebsocketClose};

#[derive(Debug)]
pub enum LogInfo {
    NowPlaying(QueueItem),
    TrackStuck(TrackStuck),
    TrackException(TrackException),
    WebsocketClose(WebsocketClose),
    PlayerAdd(u64),
    PlayerRemove(u64),
    PlayerUpdate(SimplePlayer),
    QueueAdd(QueueItem),
    QueueRemove(QueueItem),
    QueueShift(QueueItem, SimplePosition),
    QueueClear(Vec<QueueItem>),
    QueueShuffle,
    PlaylistAdd(NewPlaylist),
    PlaylistRemove(Playlist),
    PlaylistUpdate(EditPlaylist),
    PlaylistLoad(Playlist, u64),
    SettingsUpdate(EditConfig),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PlayerUpdateInfo {
    Looping,
    Playing,
    Position,
    Paused,
    Volume,
    Filters,
}

fn get_updates(value: impl Serialize) -> Vec<(String, Value)> {
    let value = serde_json::to_value(value);

    if let Ok(value) = value {
        let mut updates = vec![];

        if let Value::Object(object) = value {
            for (key, value) in object {
                if !value.is_null() {
                    updates.push((key, value));
                }
            }
        }

        updates
    } else {
        Vec::new()
    }
}

fn has_update(value: impl Serialize) -> bool {
    !get_updates(value).is_empty()
}

pub async fn register(
    pool: &PgPool,
    redis_pool: &RedisPool,
    guild: u64,
    user: User,
    info: LogInfo,
) -> ApiResult<()> {
    let mut title = "";

    let player_message = match &info {
        LogInfo::PlayerAdd(id) => {
            title = "Player Connected";
            format!("Connected to the channel <#{}>.", id)
        },
        LogInfo::PlayerRemove(id) => {
            title = "Player Disconnected";
            format!("Disconnected from the channel <#{}>.", id)
        },
        LogInfo::PlayerUpdate(player) if has_update(&player) => {
            title = "Player Updated";

            let (key, value) = get_updates(&player).remove(0);

            match serde_json::from_str(format!("\"{}\"", key).as_str())? {
                PlayerUpdateInfo::Looping => {
                    format!("Set the player loop to {}.", value)
                },
                PlayerUpdateInfo::Playing => {
                    format!(
                        "Set the playing track to {}.",
                        format_track(
                            &queue::get_track(
                                redis_pool,
                                guild,
                                value.as_i64().unwrap_or_default() as i32,
                            )
                            .await?
                            .unwrap_or_default()
                        )
                    )
                },
                PlayerUpdateInfo::Position => {
                    format!(
                        "Set the player position to {}.",
                        format_duration(value.as_u64().unwrap_or_default())
                    )
                },
                PlayerUpdateInfo::Paused if value.as_bool().unwrap_or_default() => {
                    "Paused the player.".to_owned()
                },
                PlayerUpdateInfo::Paused => "Resumed the player.".to_owned(),
                PlayerUpdateInfo::Volume => {
                    format!("Set the player volume to {}%.", value)
                },
                PlayerUpdateInfo::Filters => {
                    "Updated the player filters or the equalizer.".to_owned()
                },
            }
        },
        _ => "".to_owned(),
    };

    let queue_message = match &info {
        LogInfo::QueueAdd(track) => {
            title = "Track Added";
            format!("Added {} to the queue.", format_track(&track))
        },
        LogInfo::QueueRemove(track) => {
            title = "Track Removed";
            format!("Removed {} from the queue.", format_track(&track))
        },
        LogInfo::QueueShift(track, position) => {
            title = "Track Moved";
            format!(
                "Moved {} to position {}.",
                format_track(&track),
                position.position
            )
        },
        LogInfo::QueueClear(tracks) => {
            title = "Queue Cleared";
            format!("Removed {} tracks from the queue.", tracks.len())
        },
        LogInfo::QueueShuffle => {
            title = "Queue Shuffled";
            "The queue has been shuffled.".to_owned()
        },
        LogInfo::PlaylistLoad(playlist, amount) => {
            title = "Playlist Loaded";
            format!(
                "Loaded {} tracks from playlist '{}'.",
                amount, playlist.name
            )
        },
        _ => "".to_owned(),
    };

    let action = match &info {
        LogInfo::PlaylistAdd(playlist) => {
            format!("Created a playlist ({}).", playlist.name)
        },
        LogInfo::PlaylistRemove(playlist) => {
            format!("Deleted a playlist ({}).", playlist.name)
        },
        LogInfo::PlaylistUpdate(playlist) if has_update(&playlist) => {
            format!(
                "Updated a playlist ({}).",
                playlist.name.as_deref().unwrap_or_default()
            )
        },
        LogInfo::SettingsUpdate(settings) if has_update(&settings) => {
            format!(
                "Updated the settings ({}).",
                get_updates(&settings)
                    .into_iter()
                    .map(|(k, _)| k)
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        },
        _ => "".to_owned(),
    };

    let config = cache::get_config(pool, redis_pool, guild).await?;

    if !user.is_bot && config.player_log > 0 && !player_message.is_empty() {
        Message::send_message(
            config.player_log as u64,
            title,
            player_message.as_str(),
            user.user.id as u64,
        )
        .send(redis_pool)
        .await?;
    } else if !user.is_bot && config.queue_log > 0 && !queue_message.is_empty() {
        Message::send_message(
            config.queue_log as u64,
            title,
            queue_message.as_str(),
            user.user.id as u64,
        )
        .send(redis_pool)
        .await?;
    }

    if !action.is_empty() {
        guild_log::create(
            pool,
            NewGuildLog {
                guild: guild as i64,
                action,
                author: user.user.id as i64,
            },
        )
        .await?;
    }

    Ok(())
}

pub async fn register_playing(
    pool: &PgPool,
    redis_pool: &RedisPool,
    guild: u64,
    info: LogInfo,
) -> ApiResult<()> {
    let mut title = "";

    let message = match info {
        LogInfo::NowPlaying(track) => {
            title = "Now Playing";
            format!("{} - <@{}>", format_track(&track), track.author)
        },
        LogInfo::TrackStuck(error) => {
            title = "Track Stuck";
            format!("The track got stuck for {}ms.", error.threshold_ms)
        },
        LogInfo::TrackException(error) => {
            title = "Track Exception";
            format!("The track could not be played: {}.", error.error)
        },
        LogInfo::WebsocketClose(error) => {
            title = "Websocket Closed";
            format!(
                "The bot was disconnected: {}{}.",
                error.code,
                error
                    .reason
                    .map(|mut reason| {
                        if reason.ends_with('.') {
                            reason.pop();
                        }
                        if !reason.is_empty() {
                            reason = format!(" {}", reason);
                        }
                        reason
                    })
                    .unwrap_or_default()
            )
        },
        _ => "".to_owned(),
    };

    let config = cache::get_config(pool, redis_pool, guild).await?;

    if config.playing_log > 0 && !message.is_empty() {
        Message::send_message(config.playing_log as u64, title, message.as_str(), None)
            .send(redis_pool)
            .await?;
    }

    Ok(())
}
