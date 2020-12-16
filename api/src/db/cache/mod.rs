use crate::constants::{
    BLACKLIST_KEY, CACHE_DUMP_INTERVAL, GUILD_CONFIG_KEY, GUILD_KEY, GUILD_PREFIX_KEY, USER_KEY,
};
use crate::db::{get_pg_conn, get_redis_conn, RedisConn};
use crate::models::account::{self, Account};
use crate::models::blacklist::{self, Blacklist};
use crate::models::config::{self, Config, EditConfig, NewConfig};
use crate::models::guild::{self, NewGuild};
use crate::routes::ApiResult;

use diesel::PgConnection;
use rocket::Rocket;
use rocket_contrib::databases::redis::Iter;
use rocket_contrib::databases::redis::{Commands, RedisResult};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::thread;
use std::time::Duration;

pub mod models;

struct Cache {
    config: Vec<Config>,
    blacklist: Vec<Blacklist>,
}

impl Cache {
    fn new(conn: &PgConnection) -> Result<Self, diesel::result::Error> {
        let config = config::all(conn)?;
        let blacklist = blacklist::all(conn)?;

        Ok(Self { config, blacklist })
    }

    fn load_config(&self, conn: &RedisConn) -> RedisResult<()> {
        for config in self.config.iter() {
            let config_key = format!("{}{}", GUILD_CONFIG_KEY, config.id);
            let prefix_key = format!("{}{}", GUILD_PREFIX_KEY, config.id);

            set(conn, &config_key, config)?;
            set(conn, &prefix_key, &config.prefix)?;
        }

        Ok(())
    }

    fn load_blacklist(&self, conn: &RedisConn) -> RedisResult<()> {
        del(conn, BLACKLIST_KEY)?;

        for blacklist in &self.blacklist {
            conn.sadd(BLACKLIST_KEY, blacklist.id)?;
        }

        Ok(())
    }

    fn load(&self, conn: &RedisConn) -> RedisResult<()> {
        self.load_config(conn)?;
        self.load_blacklist(conn)?;

        Ok(())
    }
}

pub fn init_cache(rocket: &Rocket) {
    let conn = get_pg_conn(rocket);
    let redis_conn = get_redis_conn(rocket);

    let cache = Cache::new(&*conn).expect("Failed to get initial cache");
    cache
        .load(&redis_conn)
        .expect("Failed to load initial cache");

    thread::spawn(move || loop {
        let user_keys: RedisResult<Iter<'_, String>> =
            redis_conn.scan_match(format!("{}{}", USER_KEY, "*"));

        if let Ok(user_keys) = user_keys {
            let mut users = vec![];

            for user_key in user_keys {
                let user: Option<Account> = get(&redis_conn, user_key.as_str()).unwrap_or(None);
                if let Some(user) = user {
                    users.push(user);
                }
            }

            let _ = account::batch_create(&*conn, users.as_slice());
        }

        let guild_keys: RedisResult<Iter<'_, String>> =
            redis_conn.scan_match(format!("{}{}", GUILD_KEY, "*"));

        if let Ok(guild_keys) = guild_keys {
            let mut guilds = vec![];

            for guild_key in guild_keys {
                let guild: Option<NewGuild> = get(&redis_conn, guild_key.as_str()).unwrap_or(None);
                if let Some(guild) = guild {
                    guilds.push(guild)
                }
            }

            let _ = guild::batch_create(&*conn, guilds.as_slice());
        }

        thread::sleep(Duration::from_millis(CACHE_DUMP_INTERVAL as u64));
    });
}

pub fn get<T: DeserializeOwned>(conn: &RedisConn, key: &str) -> ApiResult<Option<T>> {
    let res: Option<String> = conn.get(key)?;
    if let Some(res) = res {
        let res = serde_json::from_str(res.as_str())?;
        Ok(res)
    } else {
        Ok(None)
    }
}

pub fn set<T: Serialize>(conn: &RedisConn, key: &str, value: &T) -> RedisResult<()> {
    conn.set(key, serde_json::to_string(value).unwrap())
}

pub fn set_and_expire<T: Serialize>(
    conn: &RedisConn,
    key: &str,
    value: &T,
    expiry: usize,
) -> RedisResult<()> {
    set(conn, key, value)?;
    conn.expire(key, expiry / 1000)
}

pub fn del(conn: &RedisConn, key: &str) -> RedisResult<()> {
    conn.del(key)
}

pub fn get_config(conn: &PgConnection, redis_conn: &RedisConn, guild: u64) -> ApiResult<Config> {
    let config: Option<Config> = get(
        redis_conn,
        format!("{}{}", GUILD_CONFIG_KEY, guild).as_str(),
    )?;

    if let Some(config) = config {
        Ok(config)
    } else {
        config::create(conn, &NewConfig { id: guild as i64 })?;
        config::update(
            conn,
            guild as i64,
            &EditConfig {
                prefix: None,
                max_queue: None,
                no_duplicate: None,
                keep_alive: None,
                guild_roles: None,
                playlist_roles: None,
                player_roles: Some(vec![guild as i64]),
                queue_roles: Some(vec![guild as i64]),
                track_roles: Some(vec![guild as i64]),
                playing_log: None,
                player_log: None,
                queue_log: None,
            },
        )?;

        invalidate_config(conn, redis_conn, guild)?;

        let config = config::find(conn, guild as i64)?;
        Ok(config)
    }
}

pub fn get_blacklist_item(conn: &RedisConn, user: u64) -> ApiResult<Option<()>> {
    let blacklist = conn.sismember(BLACKLIST_KEY, user)?;

    if blacklist {
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

pub fn invalidate_config(conn: &PgConnection, redis_conn: &RedisConn, guild: u64) -> ApiResult<()> {
    let config = config::find(conn, guild as i64)?;

    set(
        redis_conn,
        &format!("{}{}", GUILD_CONFIG_KEY, guild),
        &config,
    )?;
    set(
        redis_conn,
        &format!("{}{}", GUILD_PREFIX_KEY, guild),
        &config.prefix,
    )?;

    Ok(())
}

pub fn invalidate_blacklist(conn: &PgConnection, redis_conn: &RedisConn) -> ApiResult<()> {
    let cache = Cache {
        config: vec![],
        blacklist: blacklist::all(conn)?,
    };

    cache.load_blacklist(redis_conn)?;

    Ok(())
}
