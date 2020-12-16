use crate::config::{get_config, get_value};
use crate::constants::{
    PLAYER_BUFFER, PLAYER_ID_KEY, PLAYER_RECONNECT_WAIT, PLAYER_STATS_KEY, PLAYER_STATS_KEY_TTL,
    PLAYER_TIMEOUT,
};
use crate::db::pubsub::models::Connected;
use crate::db::pubsub::Message;
use crate::db::{cache, get_pg_conn, get_redis_conn, RedisConn};
use crate::models::guild_stat::{self, NewGuildStat};
use crate::routes::{ApiResponse, ApiResult};
use crate::utils::log::{self, LogInfo};
use crate::utils::metrics::{ANDESITE_EVENTS, PLAYED_TRACKS, VOICE_CLOSES};
use crate::utils::polling;
use crate::utils::queue::Queue;
use crate::utils::{block_on, to_screaming_snake_case};

use chrono::Utc;
use dashmap::mapref::one::Ref;
use event_listener::Event;
use futures::StreamExt;
use reqwest::blocking;
use rocket::config::Value;
use rocket::Rocket;
use std::collections::HashMap;
use std::convert::TryInto;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use twilight_andesite::client::Lavalink;
use twilight_andesite::http::{load_track, LoadedTracks, Track};
use twilight_andesite::model::{
    Destroy, GetPlayer, IncomingEvent, Opcode, PlayerUpdate, PlayerUpdateState, Stats,
};
use twilight_andesite::node::{Node, Resume};
use twilight_andesite::player::Player;
use twilight_model::id::{GuildId, UserId};
use twilight_model::voice::CloseCode;

lazy_static! {
    static ref PLAYER_CLIENT: RwLock<Option<Lavalink>> = RwLock::new(None);
    static ref PLAYER_NODE: RwLock<Option<Node>> = RwLock::new(None);
    static ref RECONNECTS: Arc<RwLock<HashMap<u64, Arc<Event>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub fn init_player(rocket: &Rocket) {
    let config = get_config(rocket);
    let andesite_config: HashMap<String, Value> = get_value(&config, "andesite");
    let discord_config: HashMap<String, Value> = get_value(&config, "discord");

    let uri: String = get_value(&andesite_config, "uri");
    let secret: String = get_value(&andesite_config, "secret");
    let id: u64 = get_value(&discord_config, "id");

    let conn = get_pg_conn(rocket);
    let redis_conn = get_redis_conn(rocket);
    let connection_id: Option<u64> = cache::get(&redis_conn, PLAYER_ID_KEY).unwrap_or(None);

    let client = Lavalink::new(UserId(id));
    let (node, mut receiver) = block_on(client.add_with_resume(
        SocketAddr::from_str(uri.as_str()).unwrap(),
        secret,
        Resume::new_with_id(PLAYER_BUFFER as u64, connection_id),
    ))
    .expect("Failed to connect to andesite node");

    PLAYER_CLIENT.write().unwrap().replace(client);
    PLAYER_NODE.write().unwrap().replace(node);

    cache::set(&redis_conn, PLAYER_ID_KEY, &get_node().connection_id())
        .expect("Failed to set andesite player id");

    thread::spawn(move || loop {
        let message = block_on(receiver.next()).unwrap();
        let op = to_screaming_snake_case(
            serde_json::to_value(&message)
                .unwrap()
                .get("op")
                .unwrap()
                .as_str()
                .unwrap(),
        );

        ANDESITE_EVENTS.with_label_values(&[op.as_str()]).inc();

        let _ = || -> ApiResult<()> {
            match message {
                IncomingEvent::TrackStart(event) => {
                    let guild = event.guild_id.0;
                    let queue = Queue::from(&redis_conn, guild);
                    let playing = queue.get_playing_track()?;

                    get_node().send(GetPlayer::new(GuildId(guild)))?;

                    if let Some(playing) = playing {
                        log::register_playing(
                            &*conn,
                            &redis_conn,
                            guild,
                            LogInfo::NowPlaying(playing.clone()),
                        )?;

                        guild_stat::create(
                            &*conn,
                            &NewGuildStat {
                                guild: guild as i64,
                                author: playing.author,
                                title: playing.title.clone(),
                            },
                        )?;

                        PLAYED_TRACKS
                            .with_label_values(&[
                                playing.title.as_str(),
                                playing.length.to_string().as_str(),
                            ])
                            .inc();
                    }

                    polling::notify(guild);
                },
                IncomingEvent::TrackEnd(event) => {
                    let guild = event.guild_id.0;

                    if event.reason == "FINISHED" {
                        play_next(&redis_conn, guild)?;
                        polling::notify(guild);
                    }
                },
                IncomingEvent::TrackStuck(event) => {
                    let guild = event.guild_id.0;

                    log::register_playing(&*conn, &redis_conn, guild, LogInfo::TrackStuck(event))?;
                    play_next(&redis_conn, guild)?;
                    polling::notify(guild);
                },
                IncomingEvent::TrackException(event) => {
                    let guild = event.guild_id.0;

                    log::register_playing(
                        &*conn,
                        &redis_conn,
                        guild,
                        LogInfo::TrackException(event),
                    )?;
                    play_next(&redis_conn, guild)?;
                    polling::notify(guild);
                },
                IncomingEvent::WebsocketClose(event) => {
                    VOICE_CLOSES
                        .with_label_values(&[event.code.to_string().as_str()])
                        .inc();

                    let guild = event.guild_id.0;

                    if event.code == CloseCode::Disconnected as i64 || !event.by_remote {
                        let client = get_client();
                        let player = client.get_player(&redis_conn, guild)?;

                        Queue::from(&redis_conn, guild).delete()?;
                        player.send(Destroy::new(GuildId(guild)))?;
                    } else if event.code != CloseCode::SessionNoLongerValid as i64 {
                        log::register_playing(
                            &*conn,
                            &redis_conn,
                            guild,
                            LogInfo::WebsocketClose(event),
                        )?;

                        let connected: Option<Connected> =
                            Message::get_connected(guild, None).send_and_wait(&redis_conn)?;
                        if let Some(connected) = connected {
                            Message::set_connected(guild, None).send_and_pause(&redis_conn)?;
                            Message::set_connected(guild, connected.channel as u64)
                                .send_and_pause(&redis_conn)?;
                        }
                    } else {
                        log::register_playing(
                            &*conn,
                            &redis_conn,
                            guild,
                            LogInfo::WebsocketClose(event),
                        )?;
                    }
                },
                _ => {},
            }

            Ok(())
        }();
    });
}

fn play_next(conn: &RedisConn, guild: u64) -> ApiResult<()> {
    let queue = Queue::from(conn, guild);
    let client = get_client();
    let player = client.get_player(conn, guild)?;

    queue.next()?;
    queue.play_with(&player)?;

    Ok(())
}

pub fn get_client() -> Lavalink {
    PLAYER_CLIENT.read().unwrap().clone().unwrap()
}

pub fn get_node() -> Node {
    PLAYER_NODE.read().unwrap().clone().unwrap()
}

pub trait NodeExt {
    fn get_stats(&self, conn: &RedisConn) -> ApiResult<Stats>;
}

impl NodeExt for Node {
    fn get_stats(&self, conn: &RedisConn) -> ApiResult<Stats> {
        let stats: Option<Stats> = cache::get(&conn, PLAYER_STATS_KEY).unwrap_or(None);
        if let Some(stats) = stats {
            return Ok(stats);
        }

        let stats = block_on(self.stats());
        cache::set_and_expire(conn, PLAYER_STATS_KEY, &stats, PLAYER_STATS_KEY_TTL)?;

        Ok(stats)
    }
}

pub trait ClientExt {
    fn get_player(&self, conn: &RedisConn, guild: u64) -> ApiResult<Ref<'_, GuildId, Player>>;
}

impl ClientExt for Lavalink {
    fn get_player(&self, conn: &RedisConn, guild: u64) -> ApiResult<Ref<'_, GuildId, Player>> {
        if let Some(player) = self.players().get(&GuildId(guild)) {
            if player.position().is_none() || !player.is_timed_out() {
                return Ok(player);
            }
        }

        let connected: Option<Connected> =
            Message::get_connected(guild, None).send_and_wait(conn)?;
        if let Some(connected) = connected {
            if let Ok(update) = fetch_player(GuildId(guild)) {
                let client = get_client();
                get_node().provide_player_update(
                    client.players(),
                    &PlayerUpdate {
                        op: Opcode::PlayerUpdate,
                        guild_id: GuildId(guild),
                        user_id: None,
                        state: update,
                    },
                )?;

                if let Some(player) = self.players().get(&GuildId(guild)) {
                    return Ok(player);
                }

                return Err(ApiResponse::internal_server_error());
            }

            let reconnects_arc = RECONNECTS.clone();
            let reconnects = reconnects_arc.read().unwrap();

            if let Some(event) = reconnects.get(&guild) {
                let listener = event.listen();

                drop(reconnects);

                listener.wait_timeout(Duration::from_millis(PLAYER_RECONNECT_WAIT as u64));

                if let Some(player) = self.players().get(&GuildId(guild)) {
                    return Ok(player);
                }
            } else {
                let reconnects_arc = RECONNECTS.clone();
                let mut reconnects = reconnects_arc.write().unwrap();
                let event = Arc::new(Event::new());
                reconnects.insert(guild, event);

                drop(reconnects);

                Message::set_connected(guild, None).send_and_pause(conn)?;
                thread::sleep(Duration::from_millis((PLAYER_RECONNECT_WAIT / 2) as u64));

                Message::set_connected(guild, connected.channel as u64).send_and_pause(conn)?;
                thread::sleep(Duration::from_millis((PLAYER_RECONNECT_WAIT / 2) as u64));

                let mut reconnects = reconnects_arc.write().unwrap();
                reconnects.remove(&guild);

                if let Some(player) = self.players().get(&GuildId(guild)) {
                    return Ok(player);
                }
            }
        }

        Err(ApiResponse::internal_server_error())
    }
}

pub trait PlayerExt {
    fn is_timed_out(&self) -> bool;
}

impl PlayerExt for Player {
    fn is_timed_out(&self) -> bool {
        let difference = Utc::now().timestamp_millis() - self.time();
        difference > 0 && (difference as usize) > PLAYER_TIMEOUT
    }
}

pub fn get_track(identifier: &str) -> Result<LoadedTracks, reqwest::Error> {
    let config = get_node().config().clone();
    let request = load_track(config, identifier).unwrap();

    blocking::Client::new()
        .execute(request.try_into().unwrap())?
        .json()
}

pub fn decode_track(track: &str) -> Result<Track, reqwest::Error> {
    let config = get_node().config().clone();
    let request = twilight_andesite::http::decode_track(config, track).unwrap();

    blocking::Client::new()
        .execute(request.try_into().unwrap())?
        .json()
}

pub fn fetch_player(guild: GuildId) -> Result<PlayerUpdateState, reqwest::Error> {
    let config = get_node().config().clone();
    let request = twilight_andesite::http::get_player(config, guild).unwrap();

    blocking::Client::new()
        .execute(request.try_into().unwrap())?
        .json()
}
