use crate::constants::{
    guild_config_key, guild_prefix_key, BLACKLIST_KEY, CACHE_DUMP_INTERVAL, GUILD_KEY, USER_KEY,
};
use crate::db::{PgPool, RedisPool};
use crate::models::account::{self, Account};
use crate::models::blacklist;
use crate::models::config::{self, Config, EditConfig, NewConfig};
use crate::models::guild::{self, NewGuild};
use crate::routes::{ApiResult, OptionExt};
use crate::utils::sleep;

use actix_web::web::block;
use redis::{AsyncCommands, AsyncIter};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use tracing::warn;

pub mod models;

async fn load_config(pool: &PgPool, redis_pool: &RedisPool) -> ApiResult<()> {
    let configs = config::all(pool).await?;

    for item in configs {
        set(redis_pool, guild_config_key(item.id as u64), &item).await?;
        set(redis_pool, guild_prefix_key(item.id as u64), &item.prefix).await?;
    }

    Ok(())
}

async fn load_blacklist(pool: &PgPool, redis_pool: &RedisPool) -> ApiResult<()> {
    let blacklists = blacklist::all(pool).await?;

    del(redis_pool, BLACKLIST_KEY).await?;

    for item in blacklists {
        sadd(redis_pool, BLACKLIST_KEY, &item.id).await?;
    }

    Ok(())
}

pub fn init_cache(pool: PgPool, redis_pool: RedisPool) {
    actix_web::rt::spawn(async move {
        loop {
            let err = run_jobs(&pool, &redis_pool).await;
            warn!("Cache jobs ended unexpectedly: {:?}", err);
        }
    });
}

async fn run_jobs(pool: &PgPool, redis_pool: &RedisPool) -> ApiResult<()> {
    load_config(pool, redis_pool).await?;
    load_blacklist(pool, redis_pool).await?;

    loop {
        let redis_pool_clone = redis_pool.clone();
        let mut conn = block(move || redis_pool_clone.get()).await?;
        let mut user_keys: AsyncIter<'_, String> =
            conn.scan_match(format!("{}:{}", USER_KEY, "*")).await?;

        let mut users = vec![];

        while let Some(user_key) = user_keys.next_item().await {
            let user: Option<Account> = get(redis_pool, user_key).await?;
            if let Some(user) = user {
                users.push(user);
            }
        }

        account::batch_create(pool, users).await?;

        let redis_pool_clone = redis_pool.clone();
        let mut conn = block(move || redis_pool_clone.get()).await?;
        let mut guild_keys: AsyncIter<'_, String> =
            conn.scan_match(format!("{}:{}", GUILD_KEY, "*")).await?;

        let mut guilds = vec![];

        while let Some(guild_key) = guild_keys.next_item().await {
            let guild: Option<NewGuild> = get(redis_pool, guild_key.as_str()).await?;
            if let Some(guild) = guild {
                guilds.push(guild)
            }
        }

        guild::batch_create(pool, guilds).await?;

        sleep(Duration::from_millis(CACHE_DUMP_INTERVAL as u64)).await?;
    }
}

pub async fn get<T: DeserializeOwned>(
    pool: &RedisPool,
    key: impl ToString,
) -> ApiResult<Option<T>> {
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await?;
    let res: Option<String> = conn.get(key.to_string()).await?;

    let res: Option<T> = res
        .map(|value| serde_json::from_str(value.as_str()))
        .transpose()?;

    Ok(res)
}

pub async fn set<T: Serialize>(pool: &RedisPool, key: impl ToString, value: &T) -> ApiResult<()> {
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await?;
    conn.set(key.to_string(), serde_json::to_string(value)?)
        .await?;

    Ok(())
}

pub async fn set_and_expire<T: Serialize>(
    pool: &RedisPool,
    key: impl ToString,
    value: &T,
    expiry: usize,
) -> ApiResult<()> {
    set(pool, key.to_string(), value).await?;

    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await?;
    conn.expire(key.to_string(), expiry / 1000).await?;

    Ok(())
}

pub async fn del(pool: &RedisPool, key: impl ToString) -> ApiResult<()> {
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await?;
    conn.del(key.to_string()).await?;

    Ok(())
}

pub async fn sadd(pool: &RedisPool, key: impl ToString, value: impl ToString) -> ApiResult<()> {
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await?;
    conn.sadd(key.to_string(), value.to_string()).await?;

    Ok(())
}

pub async fn sismember(
    pool: &RedisPool,
    key: impl ToString,
    value: impl ToString,
) -> ApiResult<bool> {
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await?;
    let res = conn.sismember(key.to_string(), value.to_string()).await?;

    Ok(res)
}

pub async fn get_config(pool: &PgPool, redis_pool: &RedisPool, guild: u64) -> ApiResult<Config> {
    let config: Option<Config> = get(redis_pool, guild_config_key(guild)).await?;

    if let Some(config) = config {
        Ok(config)
    } else {
        config::create(pool, NewConfig { id: guild as i64 }).await?;
        config::update(
            pool,
            guild as i64,
            EditConfig {
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
        )
        .await?;

        invalidate_config(pool, redis_pool, guild).await?;

        let config = config::find(pool, guild as i64).await?.or_not_found()?;

        Ok(config)
    }
}

pub async fn get_blacklist_item(pool: &RedisPool, user: u64) -> ApiResult<Option<()>> {
    let blacklist = sismember(pool, BLACKLIST_KEY, &user).await?;

    if blacklist {
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

pub async fn invalidate_config(pool: &PgPool, redis_pool: &RedisPool, guild: u64) -> ApiResult<()> {
    let config = config::find(pool, guild as i64).await?.or_not_found()?;

    set(redis_pool, guild_config_key(guild), &config).await?;
    set(redis_pool, guild_prefix_key(guild), &config.prefix).await?;

    Ok(())
}

pub async fn invalidate_blacklist(pool: &PgPool, redis_pool: &RedisPool) -> ApiResult<()> {
    load_blacklist(pool, redis_pool).await?;

    Ok(())
}
