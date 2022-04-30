use crate::{db::RedisPool, ApiResult};

use actix_web::web::block;
use redis::{AsyncCommands, ToRedisArgs};
use serde::{de::DeserializeOwned, Serialize};

pub async fn get<K, T>(pool: &RedisPool, key: K) -> ApiResult<Option<T>>
where
    K: ToRedisArgs + Send + Sync,
    T: DeserializeOwned,
{
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await??;
    let res: Option<String> = conn.get(key).await?;

    let res: Option<T> = res
        .map(|x| {
            serde_json::from_str(x.as_str())
                .or_else(|_| serde_json::from_str(format!("\"{}\"", x).as_str()))
        })
        .transpose()?;

    Ok(res)
}

pub async fn set<K, T>(pool: &RedisPool, key: K, value: &T) -> ApiResult<()>
where
    K: ToRedisArgs + Send + Sync,
    T: Serialize,
{
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await??;
    conn.set(key, serde_json::to_string(value)?.trim_matches('"'))
        .await?;

    Ok(())
}

pub async fn set_ex<K, T>(pool: &RedisPool, key: K, value: &T, expiry: usize) -> ApiResult<()>
where
    K: ToRedisArgs + Send + Sync,
    T: Serialize,
{
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await??;
    conn.set_ex(
        key,
        serde_json::to_string(value)?.trim_matches('"'),
        expiry / 1000,
    )
    .await?;

    Ok(())
}

pub async fn del<K>(pool: &RedisPool, key: K) -> ApiResult<()>
where
    K: ToRedisArgs + Send + Sync,
{
    let pool = pool.clone();
    let mut conn = block(move || pool.get()).await??;
    conn.del(key).await?;

    Ok(())
}
