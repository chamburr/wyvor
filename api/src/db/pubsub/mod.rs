use crate::constants::{PUBSUB_CHANNEL, PUBSUB_MESSAGE_TIMEOUT};
use crate::db::{get_redis_conn, RedisConn};
use crate::routes::{ApiResponse, ApiResult};
use crate::utils::player::get_node;

use event_listener::Event;
use rocket::Rocket;
use rocket_contrib::databases::redis::{Commands, ErrorKind, RedisError};
use rocket_contrib::json::JsonValue;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use twilight_andesite::model::Play;
use twilight_andesite::model::{SlimVoiceServerUpdate, VoiceUpdate};
use twilight_model::id::GuildId;
use uuid::Uuid;

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
    pub fn new(op: &str, data: JsonValue) -> Message {
        Message {
            op: op.to_owned(),
            id: Some(Uuid::new_v4().to_string()),
            data: data.into(),
        }
    }

    pub fn send(&self, conn: &RedisConn) -> Result<(), RedisError> {
        let payload = serde_json::to_string(self).unwrap();
        let _: () = conn.publish(PUBSUB_CHANNEL, payload)?;
        Ok(())
    }

    pub fn send_and_wait<T: DeserializeOwned>(&self, conn: &RedisConn) -> ApiResult<Option<T>> {
        let id = self.id.clone().unwrap();

        let events_arc = EVENTS.clone();
        let messages_arc = MESSAGES.clone();

        let event = Arc::new(Event::new());
        let mut events = events_arc.write().unwrap();
        events.insert(id.clone(), event.clone());
        drop(events);

        let listener = event.listen();
        self.send(conn)?;

        listener.wait_timeout(Duration::from_millis(PUBSUB_MESSAGE_TIMEOUT as u64));

        let mut messages = messages_arc.write().unwrap();
        let message = if let Some(value) = messages.get(&id) {
            Some(value.clone())
        } else {
            None
        };

        messages.remove(&id);
        drop(messages);

        let mut events = events_arc.write().unwrap();
        events.remove(&id);
        drop(events);

        if let Some(value) = message {
            Ok(serde_json::from_value(value)?)
        } else {
            Err(ApiResponse::from(RedisError::from((
                ErrorKind::IoError,
                "Timed out waiting for message",
            ))))
        }
    }

    pub fn send_and_pause(&self, conn: &RedisConn) -> ApiResult<Option<()>> {
        let value: Option<Value> = self.send_and_wait(conn)?;
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

pub fn init_pubsub(rocket: &Rocket) {
    let mut conn = get_redis_conn(rocket);

    let events_arc = EVENTS.clone();
    let messages_arc = MESSAGES.clone();

    thread::spawn(move || {
        let mut pubsub = conn.as_pubsub();
        pubsub.subscribe(PUBSUB_CHANNEL).unwrap();

        loop {
            let payload: Message = {
                let payload_str: String = pubsub.get_message().unwrap().get_payload().unwrap();
                serde_json::from_str(&payload_str).unwrap()
            };

            let _ = || -> ApiResult<()> {
                match payload.op.as_str() {
                    "response" => {
                        let events = events_arc.read().unwrap();
                        if let Some(id) = payload.id {
                            if let Some(event) = events.get(&id) {
                                let event = event.clone();
                                drop(events);

                                let mut messages = messages_arc.write().unwrap();
                                messages.insert(id, payload.data);

                                event.notify(usize::MAX);
                            }
                        }
                    },
                    "voice_update" => {
                        let update: models::VoiceUpdate = serde_json::from_value(payload.data)?;
                        let guild = GuildId(update.guild as u64);

                        get_node().send(VoiceUpdate::new(
                            guild,
                            update.session,
                            SlimVoiceServerUpdate {
                                endpoint: Some(update.endpoint),
                                token: update.token,
                            },
                        ))?;

                        get_node().send(Play::new(guild, ""))?;
                    },
                    _ => (),
                }

                Ok(())
            }();
        }
    });
}
