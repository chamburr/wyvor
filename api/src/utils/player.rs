use crate::config::{get_andesite_address, CONFIG};
use crate::constants::{player_key, PLAYER_QUEUE, PLAYER_RECONNECT_WAIT, PLAYER_SEND_QUEUE};
use crate::db::pubsub::models::Connected;
use crate::db::pubsub::Message;
use crate::db::{cache, PgPool, RedisPool};
use crate::models::guild_stat::{self, NewGuildStat};
use crate::routes::ApiResult;
use crate::utils::log::{self, LogInfo};
use crate::utils::{polling, queue, sleep};

use event_listener::Event;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::warn;
use twilight_andesite::http::{self, LoadedTracks, Track};
use twilight_andesite::model::{
    Destroy, Filters, GetPlayer, IncomingEvent, OutgoingEvent, PlayerUpdateState,
};
use twilight_andesite::node::NodeConfig;
use twilight_model::id::{GuildId, UserId};

lazy_static! {
    static ref CHANNEL: Arc<RwLock<Option<Channel>>> = Arc::new(RwLock::new(None));
    static ref RECONNECTS: Arc<RwLock<HashMap<u64, Arc<Event>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub fn init_player(pool: PgPool, redis_pool: RedisPool, channel: Channel) {
    actix_web::rt::spawn(async move {
        loop {
            let err = run_jobs(&pool, &redis_pool, &channel).await;
            warn!("Player jobs ended unexpectedly: {:?}", err);
        }
    });
}

async fn run_jobs(pool: &PgPool, redis_pool: &RedisPool, channel: &Channel) -> ApiResult<()> {
    CHANNEL.clone().write()?.replace(channel.clone());

    let mut consumer = channel
        .basic_consume(
            PLAYER_QUEUE,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(message) = consumer.next().await {
        match message {
            Ok((channel, delivery)) => {
                if let Err(err) = channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await
                {
                    warn!("Failed to ack amqp delivery: {:?}", err);
                }

                match serde_json::from_slice::<IncomingEvent>(delivery.data.as_slice()) {
                    Ok(payload) => {
                        if let Err(err) = handle_payload(pool, redis_pool, payload).await {
                            warn!("Failed to handle amqp payload: {:?}", err)
                        }
                    },
                    Err(err) => {
                        warn!("Failed to deserialize amqp payload: {:?}", err);
                    },
                }
            },
            Err(err) => {
                warn!("Failed to consume amqp delivery: {:?}", err);
            },
        }
    }

    Ok(())
}

async fn handle_payload(
    pool: &PgPool,
    redis_pool: &RedisPool,
    payload: IncomingEvent,
) -> ApiResult<()> {
    let guild = payload.guild_id().0;

    match payload {
        IncomingEvent::TrackStart(event) => {
            send(GetPlayer::new(event.guild_id)).await?;

            let playing = queue::get_playing_track(redis_pool, guild)
                .await?
                .unwrap_or_default();

            let stat = NewGuildStat {
                guild: guild as i64,
                author: playing.author,
                title: playing.title.clone(),
            };

            guild_stat::create(pool, stat).await?;
            log::register_playing(pool, redis_pool, guild, LogInfo::NowPlaying(playing)).await?;

            polling::notify(guild)?;
        },
        IncomingEvent::TrackEnd(event) => {
            if event.reason == "FINISHED" {
                queue::play_next(redis_pool, guild).await?;
                polling::notify(guild)?;
            }
        },
        IncomingEvent::TrackStuck(event) => {
            log::register_playing(pool, redis_pool, guild, LogInfo::TrackStuck(event)).await?;
            queue::play_next(redis_pool, guild).await?;
            polling::notify(guild)?;
        },
        IncomingEvent::TrackException(event) => {
            log::register_playing(pool, redis_pool, guild, LogInfo::TrackException(event)).await?;
            queue::play_next(redis_pool, guild).await?;
            polling::notify(guild)?;
        },
        IncomingEvent::WebsocketClose(event) => {
            if reconnect(redis_pool, guild).await? {
                log::register_playing(pool, redis_pool, guild, LogInfo::WebsocketClose(event))
                    .await?;
            } else {
                send(Destroy::new(GuildId(guild))).await?;
                queue::delete(redis_pool, guild).await?;
            }
        },
        IncomingEvent::PlayerDestroy(event) => {
            if !event.cleanup {
                queue::delete(redis_pool, guild).await?;
            } else {
                reconnect(redis_pool, guild).await?;
            }
        },
        _ => {},
    }

    Ok(())
}

async fn reconnect(pool: &RedisPool, guild: u64) -> ApiResult<bool> {
    let connected: Option<Connected> = Message::get_connected(guild, None)
        .send_and_wait(pool)
        .await?;

    if let Some(connected) = connected {
        Message::set_connected(guild, None)
            .send_and_pause(pool)
            .await?;
        Message::set_connected(guild, connected.channel as u64)
            .send_and_pause(pool)
            .await?;

        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn send(event: impl Into<OutgoingEvent>) -> ApiResult<()> {
    let channel_arc = CHANNEL.clone();
    let channel_guard = channel_arc.read()?;
    let channel = channel_guard.as_ref().unwrap().clone();
    drop(channel_guard);

    channel
        .basic_publish(
            "",
            PLAYER_SEND_QUEUE,
            BasicPublishOptions::default(),
            serde_json::to_vec(&event.into())?,
            BasicProperties::default(),
        )
        .await?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    pub guild_id: GuildId,
    pub time: i64,
    pub position: Option<i64>,
    pub paused: bool,
    pub volume: i64,
    pub filters: Filters,
}

impl Player {
    pub fn from(guild: u64, state: PlayerUpdateState) -> Self {
        Self {
            guild_id: GuildId(guild),
            time: state.time,
            position: state.position,
            paused: state.paused,
            volume: state.volume,
            filters: state.filters,
        }
    }
}

pub async fn get_player(pool: &RedisPool, guild: u64) -> ApiResult<Player> {
    let state: Option<PlayerUpdateState> = cache::get(pool, player_key(guild)).await?;

    if let Some(state) = state {
        let player = Player::from(guild, state);
        return Ok(player);
    }

    let connected = Message::get_connected(guild, None)
        .send_and_pause(pool)
        .await?;

    if connected.is_some() {
        if let Ok(state) = fetch_player(guild).await {
            cache::set(pool, player_key(guild), &state).await?;
            return Ok(Player::from(guild, state));
        }

        let reconnects_arc = RECONNECTS.clone();
        let reconnects = reconnects_arc.read()?;

        if let Some(event) = reconnects.get(&guild) {
            let listener = event.listen();
            drop(reconnects);
            listener.await;
        } else {
            drop(reconnects);

            let event = Arc::new(Event::new());
            let mut reconnects = reconnects_arc.write()?;
            reconnects.insert(guild, event.clone());
            drop(reconnects);

            reconnect(pool, guild).await?;
            sleep(Duration::from_millis(PLAYER_RECONNECT_WAIT as u64)).await?;
            event.notify(usize::MAX);

            let mut reconnects = reconnects_arc.write()?;
            reconnects.remove(&guild);
        }

        let state: Option<PlayerUpdateState> = cache::get(pool, player_key(guild)).await?;
        if let Some(state) = state {
            return Ok(Player::from(guild, state));
        }
    }

    Err(().into())
}

fn get_config() -> ApiResult<NodeConfig> {
    let config = NodeConfig {
        user_id: UserId(CONFIG.bot_client_id),
        address: get_andesite_address()?,
        authorization: CONFIG.andesite_secret.clone(),
        resume: None,
    };

    Ok(config)
}

pub async fn get_track(identifier: &str) -> ApiResult<LoadedTracks> {
    let request = http::load_track(get_config()?, identifier)?;

    let tracks = reqwest::Client::new()
        .execute(request.try_into()?)
        .await?
        .json()
        .await?;

    Ok(tracks)
}

pub async fn decode_track(track: &str) -> ApiResult<Track> {
    let request = http::decode_track(get_config()?, track)?;

    let track = reqwest::Client::new()
        .execute(request.try_into()?)
        .await?
        .json()
        .await?;

    Ok(track)
}

pub async fn fetch_player(guild: u64) -> ApiResult<PlayerUpdateState> {
    let request = http::get_player(get_config()?, GuildId(guild))?;

    let track = reqwest::Client::new()
        .execute(request.try_into()?)
        .await?
        .json()
        .await?;

    Ok(track)
}
