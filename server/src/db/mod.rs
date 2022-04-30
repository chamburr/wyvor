use crate::{config::CONFIG, error::ApiResult};

use diesel::{r2d2::ConnectionManager, PgConnection};
use lazy_static::lazy_static;
use r2d2::{ManageConnection, Pool};
use redis::{Client, IntoConnectionInfo, RedisError};
use tokio::runtime::Runtime;

pub mod cache;
pub mod schema;

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

pub struct PgConnectionManager {
    pub inner: ConnectionManager<PgConnection>,
}

impl PgConnectionManager {
    pub fn new<T: Into<String>>(database_url: T) -> Self {
        Self {
            inner: ConnectionManager::new(database_url),
        }
    }
}

impl ManageConnection for PgConnectionManager {
    type Connection = PgConnection;
    type Error = diesel::r2d2::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        self.inner.connect()
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        self.inner.is_valid(conn)
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub struct RedisConnectionManager {
    pub inner: Client,
}

impl RedisConnectionManager {
    pub fn new(info: String) -> Result<RedisConnectionManager, RedisError> {
        Ok(RedisConnectionManager {
            inner: Client::open(info.into_connection_info()?)?,
        })
    }
}

impl ManageConnection for RedisConnectionManager {
    type Connection = redis::aio::Connection;
    type Error = RedisError;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        RUNTIME.block_on(async move { self.inner.get_async_connection().await })
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        RUNTIME.block_on(async move { redis::cmd("PING").query_async(conn).await })
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub type PgPool = Pool<PgConnectionManager>;
pub type RedisPool = Pool<RedisConnectionManager>;

pub fn get_pg_pool() -> ApiResult<PgPool> {
    let uri = CONFIG.postgres_uri();
    let pool = Pool::builder().build(PgConnectionManager::new(uri))?;

    Ok(pool)
}

pub fn get_redis_pool() -> ApiResult<RedisPool> {
    let uri = CONFIG.redis_uri();
    let pool = Pool::builder().build(RedisConnectionManager::new(uri)?)?;

    Ok(pool)
}

embed_migrations!();

pub async fn run_migrations(pool: &PgPool) -> ApiResult<()> {
    let conn = pool.get()?;

    embedded_migrations::run(&*conn)?;

    Ok(())
}
