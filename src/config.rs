use crate::error::ApiResult;

use serde::Deserialize;
use std::{
    env,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
use url::Url;

macro_rules! make_config {
    ($($name:ident: $ty:ty),*) => {
        use lazy_static::lazy_static;
        use paste::paste;
        use std::stringify;

        paste! {
            #[derive(Debug)]
            pub struct Config {
                $(pub $name: $ty),*
            }

            lazy_static! {
                pub static ref CONFIG: Config = Config {
                    $($name: get_env(stringify!([<$name:upper>]))),*
                };
            }
        }
    }
}

make_config! {
    base_uri: String,
    environment: Environment,
    sentry_dsn: String,
    api_host: String,
    api_port: u16,
    api_workers: u64,
    api_secret: String,
    postgres_host: String,
    postgres_port: u16,
    postgres_user: String,
    postgres_password: String,
    postgres_database: String,
    redis_host: String,
    redis_port: u16,
    smtp_enabled: bool,
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_password: String,
    smtp_sender: String,
    smtp_tls: bool,
    music_host: String,
    music_port: u16
}

fn get_env<T>(name: &str) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    env::var(name)
        .unwrap_or_else(|_| panic!("Missing environmental variable: {}", name))
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Invalid environmental variable: {}", name))
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

impl Config {
    pub fn api_address(&self) -> SocketAddr {
        SocketAddr::new(
            IpAddr::from_str(self.api_host.as_str()).unwrap(),
            self.api_port,
        )
    }

    pub fn postgres_uri(&self) -> String {
        let mut uri = Url::parse("postgres://").unwrap();

        uri.set_host(Some(self.postgres_host.as_str())).unwrap();
        uri.set_port(Some(self.postgres_port)).unwrap();
        uri.set_username(self.postgres_user.as_str()).unwrap();
        uri.set_password(Some(self.postgres_password.as_str()))
            .unwrap();
        uri.set_path(format!("/{}", self.postgres_database).as_str());

        uri.into()
    }

    pub fn redis_uri(&self) -> String {
        let mut uri = Url::parse("redis://").unwrap();

        uri.set_host(Some(self.redis_host.as_str())).unwrap();
        uri.set_port(Some(self.redis_port)).unwrap();

        uri.into()
    }

    pub fn music_uri(&self) -> String {
        let mut uri = Url::parse("http://").unwrap();

        uri.set_host(Some(self.music_host.as_str())).unwrap();
        uri.set_port(Some(self.music_port)).unwrap();

        uri.into()
    }
}
