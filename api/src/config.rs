use crate::constants::HTTP_KEEP_ALIVE;

use rocket::config::{Config, Environment, Value};
use rocket::Rocket;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use url::Url;

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

pub fn get_config(rocket: &Rocket) -> HashMap<String, Value> {
    rocket
        .config()
        .extras
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

pub fn get_value<'a, T: Deserialize<'a>>(map: &HashMap<String, Value>, key: &str) -> T {
    map.get(key).unwrap().clone().try_into().unwrap()
}

pub fn get_api_secret() -> String {
    get_env("API_SECRET")
}

pub fn get_sentry_dsn() -> String {
    get_env("SENTRY_DSN")
}

pub fn get_environment() -> Environment {
    match get_env("ENVIRONMENT").as_str() {
        "production" => Environment::Production,
        "development" => Environment::Development,
        _ => panic!("Invalid environment variable: ENVIRONMENT"),
    }
}

pub fn get_rocket_config() -> Config {
    let environment = get_environment();
    let address = get_env("API_HOST");
    let port = get_env_as::<u16>("API_PORT");
    let workers = get_env_as::<u16>("API_WORKERS");
    let timeout = (HTTP_KEEP_ALIVE / 1000) as u32;
    let secret = get_env("API_SECRET");

    Config::build(environment)
        .address(address)
        .port(port)
        .workers(workers)
        .keep_alive(timeout)
        .secret_key(secret)
        .extra("databases", get_database_config())
        .extra("andesite", get_andesite_config())
        .extra("discord", get_discord_config())
        .finalize()
        .expect("Failed to build rocket config")
}

fn get_postgres_uri() -> Result<String, ()> {
    let mut uri = Url::parse("postgres://").or(Err(()))?;

    uri.set_host(Some(get_env("POSTGRES_HOST").as_str()))
        .or(Err(()))?;
    uri.set_port(Some(get_env_as::<u16>("POSTGRES_PORT")))?;
    uri.set_username(get_env("POSTGRES_USER").as_str())?;
    uri.set_password(Some(get_env("POSTGRES_PASSWORD").as_str()))?;
    uri.set_path(format!("/{}", get_env("POSTGRES_DATABASE")).as_str());

    Ok(uri.into_string())
}

fn get_redis_uri() -> Result<String, ()> {
    let mut uri = Url::parse("redis://").or(Err(()))?;

    uri.set_host(Some(get_env("REDIS_HOST").as_str()))
        .or(Err(()))?;
    uri.set_port(Some(get_env_as::<u16>("REDIS_PORT")))?;

    Ok(uri.into_string())
}

pub fn get_database_config() -> HashMap<String, Value> {
    let mut database = HashMap::new();
    let mut pg_config = HashMap::new();
    let mut redis_config = HashMap::new();

    let pg_uri = get_postgres_uri().expect("Failed to build postgres config");
    let redis_uri = get_redis_uri().expect("Failed to build redis config");

    pg_config.insert("url".to_owned(), Value::from(pg_uri));
    redis_config.insert("url".to_owned(), Value::from(redis_uri));

    database.insert("postgres".to_owned(), Value::from(pg_config));
    database.insert("redis".to_owned(), Value::from(redis_config));

    database
}

fn get_andesite_uri() -> Result<String, ()> {
    let ip = IpAddr::from_str(get_env("ANDESITE_HOST").as_str()).or(Err(()))?;
    let port = get_env_as::<u16>("ANDESITE_PORT");

    let uri = SocketAddr::new(ip, port);

    Ok(uri.to_string())
}

pub fn get_andesite_config() -> HashMap<String, Value> {
    let mut andesite = HashMap::new();

    let andesite_uri = get_andesite_uri().expect("Failed to build andesite config");
    let andesite_secret = get_env("ANDESITE_SECRET");

    andesite.insert("uri".to_owned(), Value::from(andesite_uri));
    andesite.insert("secret".to_owned(), Value::from(andesite_secret));

    andesite
}

fn get_base_uri() -> Result<String, ()> {
    let uri = Url::parse(get_env("BASE_URI").as_str()).or(Err(()))?;
    Ok(uri.into_string())
}

pub fn get_discord_config() -> HashMap<String, Value> {
    let mut discord = HashMap::new();

    let discord_id = get_env_as::<i64>("BOT_CLIENT_ID");
    let discord_secret = get_env("BOT_CLIENT_SECRET");
    let base_uri = get_base_uri().expect("Failed to parse BASE_URI");

    discord.insert("id".to_owned(), Value::from(discord_id));
    discord.insert("secret".to_owned(), Value::from(discord_secret));
    discord.insert("uri".to_owned(), Value::from(base_uri));

    discord
}
