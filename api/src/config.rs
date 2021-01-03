use crate::routes::ApiResult;

use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use url::Url;

lazy_static! {
    pub static ref CONFIG: Config = Config {
        base_uri: get_env("BASE_URI"),
        environment: get_env_as("ENVIRONMENT"),
        sentry_dsn: get_env("SENTRY_DSN"),
        bot_client_id: get_env_as("BOT_CLIENT_ID"),
        bot_client_secret: get_env("BOT_CLIENT_SECRET"),
        api_host: get_env("API_HOST"),
        api_port: get_env_as("API_PORT"),
        api_workers: get_env_as("API_WORKERS"),
        api_secret: get_env("API_SECRET"),
        postgres_host: get_env("POSTGRES_HOST"),
        postgres_port: get_env_as("POSTGRES_PORT"),
        postgres_user: get_env("POSTGRES_USER"),
        postgres_password: get_env("POSTGRES_PASSWORD"),
        postgres_database: get_env("POSTGRES_DATABASE"),
        redis_host: get_env("REDIS_HOST"),
        redis_port: get_env_as("REDIS_PORT"),
        rabbit_host: get_env("RABBIT_HOST"),
        rabbit_port: get_env_as("RABBIT_PORT"),
        andesite_host: get_env("ANDESITE_HOST"),
        andesite_port: get_env_as("ANDESITE_PORT"),
        andesite_secret: get_env("ANDESITE_SECRET"),
    };
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl FromStr for Environment {
    type Err = serde_json::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(format!("\"{}\"", s).as_str())
    }
}

#[derive(Debug)]
pub struct Config {
    pub base_uri: String,
    pub environment: Environment,
    pub sentry_dsn: String,
    pub bot_client_id: u64,
    pub bot_client_secret: String,
    pub api_host: String,
    pub api_port: u16,
    pub api_workers: u64,
    pub api_secret: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_database: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub rabbit_host: String,
    pub rabbit_port: u16,
    pub andesite_host: String,
    pub andesite_port: u16,
    pub andesite_secret: String,
}

fn get_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Missing environmental variable: {}", name))
}

fn get_env_as<T>(name: &str) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    get_env(name)
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid environmental variable: {}", name))
}

pub fn get_api_address() -> ApiResult<SocketAddr> {
    let addr = SocketAddr::new(IpAddr::from_str(CONFIG.api_host.as_str())?, CONFIG.api_port);

    Ok(addr)
}

pub fn get_andesite_address() -> ApiResult<SocketAddr> {
    let addr = SocketAddr::new(
        IpAddr::from_str(CONFIG.andesite_host.as_str())?,
        CONFIG.andesite_port,
    );

    Ok(addr)
}

pub fn get_postgres_uri() -> ApiResult<String> {
    let mut uri = Url::parse("postgres://")?;

    uri.set_host(Some(CONFIG.postgres_host.as_str()))?;
    uri.set_port(Some(CONFIG.postgres_port))?;
    uri.set_username(CONFIG.postgres_user.as_str())?;
    uri.set_password(Some(CONFIG.postgres_password.as_str()))?;
    uri.set_path(format!("/{}", CONFIG.postgres_database).as_str());

    Ok(uri.into_string())
}

pub fn get_redis_uri() -> ApiResult<String> {
    let mut uri = Url::parse("redis://")?;

    uri.set_host(Some(CONFIG.redis_host.as_str()))?;
    uri.set_port(Some(CONFIG.redis_port))?;

    Ok(uri.into_string())
}

pub fn get_rabbit_uri() -> ApiResult<String> {
    let mut uri = Url::parse("amqp://")?;

    uri.set_host(Some(CONFIG.rabbit_host.as_str()))?;
    uri.set_port(Some(CONFIG.rabbit_port))?;

    Ok(uri.into_string())
}
