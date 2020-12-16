use crate::db::pubsub::Message;
use crate::db::{cache, RedisConn};
use crate::models::config::EditConfig;
use crate::models::guild_log::{self, NewGuildLog};
use crate::models::playlist::{EditPlaylist, NewPlaylist, Playlist};
use crate::models::playlist_item::PlaylistItem;
use crate::routes::guilds::{SimplePlayer, SimplePosition};
use crate::routes::ApiResult;
use crate::utils::auth::User;
use crate::utils::queue::{Queue, QueueItem};
use crate::utils::{format_duration, format_track};

use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use twilight_andesite::model::{TrackException, TrackStuck, WebsocketClose};

#[derive(Debug, Clone)]
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
    PlaylistLoad(Playlist, Vec<PlaylistItem>),
    SettingsUpdate(EditConfig),
}

impl LogInfo {
    fn title(&self) -> &str {
        match self {
            LogInfo::NowPlaying(_) => "Now Playing",
            LogInfo::TrackStuck(_) => "Track Stuck",
            LogInfo::TrackException(_) => "Track Exception",
            LogInfo::WebsocketClose(_) => "Connection Closed",
            LogInfo::PlayerAdd(_) => "Player Connected",
            LogInfo::PlayerRemove(_) => "Player Disconnected",
            LogInfo::PlayerUpdate(_) => "Player Updated",
            LogInfo::QueueAdd(_) => "Track Added",
            LogInfo::QueueRemove(_) => "Track Removed",
            LogInfo::QueueShift(_, _) => "Track Moved",
            LogInfo::QueueClear(_) => "Queue Cleared",
            LogInfo::QueueShuffle => "Queue Shuffled",
            LogInfo::PlaylistLoad(_, _) => "Playlist Loaded",
            _ => "",
        }
    }

    fn message(&self) -> &str {
        match self {
            LogInfo::NowPlaying(_) => "{} - <@{}>",
            LogInfo::TrackStuck(_) => "The track got stuck for {}ms while playing.",
            LogInfo::TrackException(_) => "The track could not be played: {}.",
            LogInfo::WebsocketClose(_) => "The bot was disconnected: {}{}.",
            LogInfo::PlayerAdd(_) => "Connected to the channel <#{}>.",
            LogInfo::PlayerRemove(_) => "Disconnected from the channel <#{}>.",
            LogInfo::QueueAdd(_) => "Added {} to the queue.",
            LogInfo::QueueRemove(_) => "Removed {} from the queue.",
            LogInfo::QueueShift(_, _) => "Moved {} to position {}",
            LogInfo::QueueClear(_) => "Removed {} tracks from the queue",
            LogInfo::QueueShuffle => "The queue has been shuffled.",
            LogInfo::PlaylistAdd(_) => "Created a playlist ({}).",
            LogInfo::PlaylistRemove(_) => "Deleted a playlist ({}).",
            LogInfo::PlaylistUpdate(_) => "Updated a playlist ({}).",
            LogInfo::PlaylistLoad(_, _) => "Loaded {} tracks from the playlist '{}'.",
            LogInfo::SettingsUpdate(_) => "Updated the settings ({}).",
            _ => "",
        }
    }

    fn format(&self, content: impl ToString) -> String {
        self.message().replace("{}", content.to_string().as_str())
    }

    fn format_n<T: ToString>(&self, content: &[T]) -> String {
        self.message()
            .split("{}")
            .enumerate()
            .map(|(index, value)| {
                if let Some(argument) = content.get(index) {
                    value.to_owned() + argument.to_string().as_str()
                } else {
                    value.to_owned()
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PlayerUpdateInfo {
    Looping,
    Playing,
    Position,
    Paused,
    Volume,
    Filters,
}

impl PlayerUpdateInfo {
    fn message(&self) -> &str {
        match self {
            PlayerUpdateInfo::Looping => "Set the player loop to {}.",
            PlayerUpdateInfo::Playing => "Set the playing track to {}.",
            PlayerUpdateInfo::Position => "Seeked the player position to {}.",
            PlayerUpdateInfo::Paused => "{} the player",
            PlayerUpdateInfo::Volume => "Set the player volume to {}%.",
            PlayerUpdateInfo::Filters => "Updated the player filters or the equalizer.",
        }
    }

    fn format(&self, content: impl ToString) -> String {
        let mut content = content.to_string();
        if content.len() > 2 && content.starts_with('"') && content.ends_with('"') {
            content = content[1..content.len() - 1].to_string()
        }

        self.message().replace("{}", content.as_str())
    }
}

fn get_updates(value: impl Serialize) -> Vec<(String, Value)> {
    let value = serde_json::to_value(value);
    if let Ok(value) = value {
        value
            .as_object()
            .unwrap_or(&Map::new())
            .iter()
            .filter(|(_, v)| !v.is_null())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        Vec::new()
    }
}

fn has_update(value: impl Serialize) -> bool {
    !get_updates(value).is_empty()
}

pub fn register(
    conn: &PgConnection,
    redis_conn: &RedisConn,
    guild: u64,
    user: User,
    info: LogInfo,
) -> ApiResult<()> {
    let queue = Queue::from(redis_conn, guild);
    let title = info.title();

    let player_message = match info.clone() {
        LogInfo::PlayerAdd(id) => info.format(id),
        LogInfo::PlayerRemove(id) => info.format(id),
        LogInfo::PlayerUpdate(player) if has_update(&player) => {
            let (key, value): (PlayerUpdateInfo, Value) = get_updates(&player)
                .get(0)
                .map(|(k, v)| {
                    let k = format!("\"{}\"", k.as_str());
                    (serde_json::from_str(k.as_str()).unwrap(), v.clone())
                })
                .unwrap();

            match key {
                PlayerUpdateInfo::Looping => key.format(value),
                PlayerUpdateInfo::Playing => key.format(format_track(
                    &queue.get()?[value.as_i64().unwrap() as usize],
                )),
                PlayerUpdateInfo::Position => key.format(format_duration(value.as_u64().unwrap())),
                PlayerUpdateInfo::Paused if value.as_bool().unwrap() => key.format("Paused"),
                PlayerUpdateInfo::Paused => key.format("Resumed"),
                PlayerUpdateInfo::Volume => key.format(value),
                PlayerUpdateInfo::Filters => key.message().to_owned(),
            }
        },
        _ => "".to_owned(),
    };

    let queue_message = match info.clone() {
        LogInfo::QueueAdd(track) => info.format(format_track(&track)),
        LogInfo::QueueRemove(track) => info.format(format_track(&track)),
        LogInfo::QueueShift(track, position) => {
            info.format_n(&[format_track(&track), position.position.to_string()])
        },
        LogInfo::QueueClear(tracks) => info.format(tracks.len()),
        LogInfo::QueueShuffle => info.message().to_owned(),
        LogInfo::PlaylistLoad(playlist, tracks) => {
            info.format_n(&[tracks.len().to_string(), playlist.name])
        },
        _ => "".to_owned(),
    };

    let action = match info.clone() {
        LogInfo::PlaylistAdd(playlist) => info.format(playlist.name),
        LogInfo::PlaylistRemove(playlist) => info.format(playlist.name),
        LogInfo::PlaylistUpdate(playlist) if has_update(&playlist) => {
            info.format(playlist.name.unwrap())
        },
        LogInfo::SettingsUpdate(settings) if has_update(&settings) => info.format(
            get_updates(&settings)
                .iter()
                .map(|(k, _)| k.clone())
                .collect::<Vec<String>>()
                .join(", "),
        ),
        _ => "".to_owned(),
    };

    let config = cache::get_config(conn, redis_conn, guild)?;

    if !user.is_bot && config.player_log > 0 && !player_message.is_empty() {
        Message::send_message(
            config.player_log as u64,
            title,
            player_message.as_str(),
            user.user.id as u64,
        )
        .send(redis_conn)?;
    } else if !user.is_bot && config.queue_log > 0 && !queue_message.is_empty() {
        Message::send_message(
            config.queue_log as u64,
            title,
            queue_message.as_str(),
            user.user.id as u64,
        )
        .send(redis_conn)?;
    }

    if !action.is_empty() {
        guild_log::create(
            conn,
            &NewGuildLog {
                guild: guild as i64,
                action,
                author: user.user.id as i64,
            },
        )?;
    }

    Ok(())
}

pub fn register_playing(
    conn: &PgConnection,
    redis_conn: &RedisConn,
    guild: u64,
    info: LogInfo,
) -> ApiResult<()> {
    let message = match info.clone() {
        LogInfo::NowPlaying(track) => {
            info.format_n(&[format_track(&track), track.author.to_string()])
        },
        LogInfo::TrackStuck(error) => info.format(error.threshold_ms),
        LogInfo::TrackException(error) => info.format(error.error),
        LogInfo::WebsocketClose(error) => info.format_n(&[
            error.code.to_string(),
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
                .unwrap_or_default(),
        ]),
        _ => "".to_owned(),
    };

    let config = cache::get_config(conn, redis_conn, guild)?;

    if config.playing_log > 0 && !message.is_empty() {
        Message::send_message(
            config.playing_log as u64,
            info.title(),
            message.as_str(),
            None,
        )
        .send(redis_conn)?;
    }

    Ok(())
}
