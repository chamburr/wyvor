use crate::constants::PUBSUB_CHANNEL;
use crate::db::{get_redis_conn, RedisPool};
use crate::routes::ApiResult;
use crate::utils::player;

use actix_web::web::block;
use event_listener::Event;
use futures::StreamExt;
use lazy_static::lazy_static;
use nanoid::nanoid;
use redis::AsyncCommands;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::warn;
use twilight_andesite::model::Play;
use twilight_andesite::model::{SlimVoiceServerUpdate, VoiceUpdate};
use twilight_model::id::GuildId;

pub mod models;

lazy_static! {
    static ref EVENTS: Arc<RwLock<HashMap<String, Arc<Event>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref MESSAGES: Arc<RwLock<HashMap<String, Value>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    op: String,
    id: Option<String>,
    data: Value,
}

impl Message {
    pub fn new(op: &str, data: Value) -> Message {
        Message {
            op: op.to_owned(),
            id: Some(nanoid!()),
            data,
        }
    }

    pub async fn send(&self, pool: &RedisPool) -> ApiResult<()> {
        let payload = serde_json::to_string(self)?;

        let pool = pool.clone();
        let mut conn = block(move || pool.get()).await?;
        let _: () = conn.publish(PUBSUB_CHANNEL, payload).await?;

        Ok(())
    }

    pub async fn send_and_wait<T: DeserializeOwned>(
        &self,
        pool: &RedisPool,
    ) -> ApiResult<Option<T>> {
        let id = self.id.as_ref().unwrap();

        let events_arc = EVENTS.clone();
        let messages_arc = MESSAGES.clone();

        let event = Arc::new(Event::new());

        let mut events = events_arc.write()?;
        events.insert(id.clone(), event.clone());
        drop(events);

        let listener = event.listen();
        self.send(pool).await?;
        listener.await;

        let mut messages = messages_arc.write()?;
        let message = messages.remove(id);

        let mut events = events_arc.write()?;
        events.remove(id);

        if let Some(message) = message {
            Ok(serde_json::from_value(message)?)
        } else {
            Ok(None)
        }
    }

    pub async fn send_and_pause(&self, pool: &RedisPool) -> ApiResult<Option<()>> {
        let value: Option<Value> = self.send_and_wait(pool).await?;

        Ok(value.map(|_| ()))
    }

    pub fn get_user(user: u64) -> Message {
        Message::new("get_user", json!({ "user": user }))
    }

    pub fn get_member(guild: u64, member: u64) -> Message {
        Message::new(
            "get_member",
            json!({
                "guild": guild,
                "member": member
            }),
        )
    }

    pub fn get_permission(guild: u64, member: u64, channel: impl Into<Option<u64>>) -> Message {
        Message::new(
            "get_permission",
            json!({
                "guild": guild,
                "member": member,
                "channel": channel.into()
            }),
        )
    }

    pub fn get_guild(guild: u64) -> Message {
        Message::new("get_guild", json!({ "guild": guild }))
    }

    pub fn send_message(
        channel: u64,
        title: &str,
        content: &str,
        author: impl Into<Option<u64>>,
    ) -> Message {
        Message::new(
            "send_message",
            json!({
                "channel": channel,
                "title": title,
                "content": content,
                "author": author.into()
            }),
        )
    }

    pub fn get_connected(guild: u64, member: impl Into<Option<u64>>) -> Message {
        Message::new(
            "get_connected",
            json!({ "guild": guild, "member": member.into() }),
        )
    }

    pub fn set_connected(guild: u64, channel: impl Into<Option<u64>>) -> Message {
        Message::new(
            "set_connected",
            json!({ "guild": guild, "channel": channel.into() }),
        )
    }
}

pub fn init_pubsub() {
    actix_web::rt::spawn(async move {
        loop {
            let err = run_jobs().await;
            warn!("Pubsub jobs ended unexpectedly: {:?}", err);
        }
    });
}

async fn run_jobs() -> ApiResult<()> {
    let conn = get_redis_conn().await?;
    let mut pubsub = conn.into_pubsub();
    pubsub.subscribe(PUBSUB_CHANNEL).await?;

    while let Some(message) = pubsub.on_message().next().await {
        match message.get_payload::<String>() {
            Ok(payload) => match serde_json::from_str::<Message>(payload.as_str()) {
                Ok(payload) => {
                    if let Err(err) = handle_payload(payload).await {
                        warn!("Failed to handle pubsub payload: {:?}", err)
                    }
                },
                Err(err) => {
                    warn!("Failed to deserialize pubsub payload: {:?}", err);
                },
            },
            Err(err) => {
                warn!("Failed to get pubsub message: {:?}", err);
            },
        }
    }

    Ok(())
}

async fn handle_payload(payload: Message) -> ApiResult<()> {
    let events_arc = EVENTS.clone();
    let messages_arc = MESSAGES.clone();

    match payload.op.as_str() {
        "response" => {
            if let Some(id) = payload.id {
                if let Some(event) = events_arc.read()?.get(&id) {
                    let event = event.clone();

                    let mut messages = messages_arc.write()?;
                    messages.insert(id, payload.data);

                    event.notify(usize::MAX);
                }
            }
        },
        "voice_update" => {
            let update: models::VoiceUpdate = serde_json::from_value(payload.data)?;
            let guild = GuildId(update.guild as u64);

            player::send(VoiceUpdate::new(
                guild,
                update.session,
                SlimVoiceServerUpdate {
                    endpoint: Some(update.endpoint),
                    token: update.token,
                },
            ))
            .await?;

            player::send(Play::new(guild, "")).await?;
        },
        _ => {},
    }

    Ok(())
}
