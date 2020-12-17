use crate::config::{get_api_secret, get_config, get_value};
use crate::constants::{
    BOT_ADMIN_KEY, BOT_OWNER_KEY, CALLBACK_PATH, COOKIE_NAME, CSRF_TOKEN_KEY, CSRF_TOKEN_KEY_TTL,
    GUILD_KEY, USER_GUILDS_KEY, USER_GUILDS_KEY_TTL, USER_KEY, USER_KEY_TTL, USER_TOKEN_KEY,
    USER_TOKEN_KEY_TTL,
};
use crate::db::cache::{self, get_blacklist_item};
use crate::db::pubsub::models::{Connected, Member, Permission};
use crate::db::pubsub::Message;
use crate::db::RedisConn;
use crate::models::account::Account;
use crate::models::guild::NewGuild;
use crate::routes::{ApiResponse, ApiResult, OptionExt};
use crate::utils::block_on;

use chrono::Duration;
use diesel::PgConnection;
use http::header::AUTHORIZATION;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rocket::config::Value;
use rocket::http::{Cookie, SameSite, Status};
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, Rocket};
use rocket_contrib::databases::redis::Commands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::RwLock;
use twilight_model::guild::Permissions;
use twilight_model::id::{ApplicationId, GuildId};
use twilight_oauth2::request::access_token_exchange::AccessTokenExchangeResponse;
use twilight_oauth2::{Client, Scope};
use url::Url;

lazy_static! {
    static ref BASE_URI: RwLock<String> = RwLock::new("".to_owned());
    static ref OAUTH_CLIENT: RwLock<Option<Client>> = RwLock::new(None);
    static ref OAUTH_SCOPES: Vec<Scope> = vec![Scope::Identify, Scope::Guilds];
}

pub fn init_oauth(rocket: &Rocket) {
    let config = get_config(rocket);
    let discord_config: HashMap<String, Value> = get_value(&config, "discord");

    let id: u64 = get_value(&discord_config, "id");
    let secret: String = get_value(&discord_config, "secret");
    let base_uri: String = get_value(&discord_config, "uri");

    let mut uri = Url::parse(base_uri.as_str()).unwrap();
    uri.set_path(CALLBACK_PATH);

    let client = Client::new(ApplicationId(id), secret, &[uri.as_str()])
        .expect("Failed to create oauth client");

    BASE_URI.write().unwrap().push_str(base_uri.as_str());
    OAUTH_CLIENT.write().unwrap().replace(client);
}

pub fn token_exchange(code: &str) -> Result<AccessTokenExchangeResponse, reqwest::Error> {
    let client = OAUTH_CLIENT.read().unwrap().clone().unwrap();

    let mut builder = client
        .access_token_exchange(code, client.redirect_uris()[0].as_str())
        .unwrap();
    let request = builder.scopes(&OAUTH_SCOPES).build();

    let headers: HeaderMap = request
        .headers
        .iter()
        .map(|(k, v)| {
            (
                HeaderName::from_str(*k).unwrap(),
                HeaderValue::from_str(*v).unwrap(),
            )
        })
        .collect();

    let response: AccessTokenExchangeResponse = reqwest::blocking::Client::new()
        .post(request.url_base)
        .form(&request.body)
        .headers(headers)
        .send()?
        .json()?;

    Ok(response)
}

pub fn get_token_cookie(token: AccessTokenExchangeResponse) -> Cookie<'static> {
    let url = Url::parse(BASE_URI.read().unwrap().as_str()).unwrap();
    let domain = url.domain().unwrap().to_owned();

    Cookie::build(COOKIE_NAME, token.access_token)
        .domain(domain)
        .http_only(true)
        .max_age(Duration::seconds(token.expires_in as i64))
        .same_site(SameSite::Lax)
        .secure(true)
        .finish()
}

pub fn get_remove_token_cookie() -> Cookie<'static> {
    let url = Url::parse(BASE_URI.read().unwrap().as_str()).unwrap();
    let domain = url.domain().unwrap().to_owned();

    let mut cookie = Cookie::named(COOKIE_NAME);
    cookie.set_domain(domain);

    cookie
}

pub fn get_csrf_redirect(conn: &RedisConn, token: &str) -> ApiResult<Option<String>> {
    let token_key = format!("{}{}", CSRF_TOKEN_KEY, token);

    cache::get(conn, token_key.as_str())
}

pub fn get_redirect_uri(conn: &RedisConn, redirect: Option<String>) -> ApiResult<String> {
    let client = OAUTH_CLIENT.read().unwrap().clone().unwrap();
    let mut uri = client.authorization_url(client.redirect_uris()[0].as_str())?;
    uri.scopes(OAUTH_SCOPES.as_slice());

    let uri = if let Some(redirect) = redirect {
        let id = nanoid!();
        let token_key = format!("{}{}", CSRF_TOKEN_KEY, id.as_str());
        cache::set_and_expire(conn, token_key.as_str(), &redirect, CSRF_TOKEN_KEY_TTL)?;

        uri.state(id.as_str());
        uri.build()
    } else {
        uri.build()
    };

    Ok(uri.replace("%20", "+"))
}

pub fn get_invite_uri(guild: Option<u64>) -> ApiResult<String> {
    let client = OAUTH_CLIENT.read().unwrap().clone().unwrap();
    let mut uri = client.bot_authorization_url();
    uri.permissions(
        Permissions::ADMINISTRATOR
            | Permissions::VIEW_CHANNEL
            | Permissions::SEND_MESSAGES
            | Permissions::EMBED_LINKS
            | Permissions::ATTACH_FILES
            | Permissions::READ_MESSAGE_HISTORY
            | Permissions::ADD_REACTIONS
            | Permissions::CONNECT
            | Permissions::SPEAK
            | Permissions::USE_VAD
            | Permissions::PRIORITY_SPEAKER,
    );
    uri.redirect_uri(client.redirect_uris()[0].as_str())?;

    if let Some(guild) = guild {
        uri.guild_id(GuildId(guild));
    }

    Ok(uri.build().replace("%20", "+"))
}

pub struct User {
    pub token: String,
    pub user: Account,
    pub is_bot: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Guild {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub permissions: i64,
    pub has_bot: bool,
}

impl User {
    fn from_token(conn: &RedisConn, token: &str) -> Result<Self, twilight_http::Error> {
        let token_key = format!("{}{}", USER_TOKEN_KEY, token);
        let user: Option<String> = cache::get(conn, &token_key).unwrap_or(None);

        if let Some(user) = user {
            let user: Option<Account> =
                cache::get(conn, format!("{}{}", USER_KEY, user).as_str()).unwrap_or(None);

            if let Some(user) = user {
                return Ok(Self {
                    token: token.to_owned(),
                    user,
                    is_bot: false,
                });
            }
        }

        let client = twilight_http::Client::new(format!("Bearer {}", token));
        let user = block_on(client.current_user())?;
        let user = Account {
            id: user.id.0 as i64,
            username: user.name,
            discriminator: user.discriminator.parse().unwrap(),
            avatar: user.avatar.unwrap_or_else(|| "".to_owned()),
        };
        let user_key = format!("{}{}", USER_KEY, user.id);

        let _ = cache::set_and_expire(conn, &token_key, &user.id.to_string(), USER_TOKEN_KEY_TTL);
        let _ = cache::set_and_expire(conn, &user_key, &user, USER_KEY_TTL);

        Ok(Self {
            token: token.to_owned(),
            user,
            is_bot: false,
        })
    }

    fn from_bot(
        conn: &RedisConn,
        id: &str,
        username: &str,
        discriminator: &str,
        avatar: &str,
    ) -> Self {
        let user = Account {
            id: id.parse().unwrap(),
            username: username.to_owned(),
            discriminator: discriminator.parse().unwrap(),
            avatar: avatar.to_owned(),
        };

        let _ = cache::set_and_expire(conn, &format!("{}{}", USER_KEY, id), &user, USER_KEY_TTL);

        Self {
            token: "".to_owned(),
            user,
            is_bot: true,
        }
    }

    pub fn revoke_token(self) -> Result<(), reqwest::Error> {
        let client = OAUTH_CLIENT.read().unwrap().clone().unwrap();
        let uri = Client::BASE_URI.replace("/authorize", "/token/revoke");

        let mut params = HashMap::new();
        params.insert("client_id".to_owned(), client.client_id().to_string());
        params.insert(
            "client_secret".to_owned(),
            client.client_secret().to_string(),
        );
        params.insert("token".to_owned(), self.token);

        reqwest::blocking::Client::new()
            .post(uri.as_str())
            .form(&params)
            .send()?;

        Ok(())
    }

    pub fn get_guilds(&self, conn: &RedisConn) -> ApiResult<Vec<Guild>> {
        let guilds_key = format!("{}{}", USER_GUILDS_KEY, self.user.id);
        let guilds: Option<Vec<Guild>> = cache::get(conn, &guilds_key).unwrap_or(None);

        if let Some(guilds) = guilds {
            return Ok(guilds);
        }

        let client = twilight_http::Client::new(format!("Bearer {}", self.token));
        let guilds = block_on(client.current_user_guilds())?
            .iter()
            .map(|guild| {
                let bot_guild: Option<NewGuild> =
                    cache::get(conn, format!("{}{}", GUILD_KEY, guild.id.0).as_str())?;

                Ok(Guild {
                    id: guild.id.0 as i64,
                    name: guild.name.clone(),
                    icon: guild.icon.clone(),
                    permissions: guild.permissions.bits() as i64,
                    has_bot: bot_guild.is_some(),
                })
            })
            .collect::<ApiResult<Vec<Guild>>>()?;

        let _ = cache::set_and_expire(conn, &guilds_key, &guilds, USER_GUILDS_KEY_TTL);

        Ok(guilds)
    }

    pub fn get_member(&self, conn: &RedisConn, guild: u64) -> ApiResult<Member> {
        let member = Message::get_member(guild, self.user.id as u64)
            .send_and_wait(conn)?
            .into_not_found()?;

        Ok(member)
    }

    pub fn is_connected(&self, conn: &RedisConn, guild: u64, user: bool) -> ApiResult<()> {
        let connected: Option<Connected> =
            Message::get_connected(guild, None).send_and_wait(conn)?;
        let user_connected: Option<Connected> =
            Message::get_connected(guild, self.user.id as u64).send_and_wait(conn)?;

        if let Some(connected) = connected {
            return if user {
                if let Some(user_connected) = user_connected {
                    if connected.channel == user_connected.channel {
                        return Ok(());
                    }

                    return Err(ApiResponse::bad_request()
                        .message("You need to be connected to the same channel as the bot."));
                }

                Err(ApiResponse::bad_request().message("You need to be connected to a channel."))
            } else {
                Ok(())
            };
        }

        Err(ApiResponse::bad_request().message("The bot is not connected to any channel."))
    }

    pub fn is_not_bot(&self) -> ApiResult<()> {
        if !self.is_bot {
            Ok(())
        } else {
            Err(ApiResponse::forbidden())
        }
    }

    pub fn has_bot_owner(&self, conn: &RedisConn) -> ApiResult<()> {
        if conn.sismember(BOT_OWNER_KEY, self.user.id)? {
            Ok(())
        } else {
            Err(ApiResponse::forbidden())
        }
    }

    pub fn has_bot_admin(&self, conn: &RedisConn) -> ApiResult<()> {
        if self.has_bot_owner(conn).is_ok() {
            return Ok(());
        }

        if conn.sismember(BOT_ADMIN_KEY, self.user.id)? {
            return Ok(());
        }

        Err(ApiResponse::forbidden())
    }

    pub fn has_manage_guild(
        &self,
        conn: &PgConnection,
        redis_conn: &RedisConn,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_conn, guild)?;
        let config = cache::get_config(conn, redis_conn, guild)?;

        if self.has_bot_owner(redis_conn).is_ok() {
            return Ok(());
        }

        if config
            .guild_roles
            .iter()
            .any(|role| member.roles.contains(role))
        {
            return Ok(());
        }

        let permission: Permission = Message::get_permission(guild, self.user.id as u64, None)
            .send_and_wait(redis_conn)?
            .into_not_found()?;
        let permission = Permissions::from_bits(permission.permission as u64)
            .ok_or_else(ApiResponse::internal_server_error)?;

        if permission.contains(Permissions::MANAGE_GUILD) {
            return Ok(());
        }

        Err(ApiResponse::forbidden())
    }

    pub fn has_manage_playlist(
        &self,
        conn: &PgConnection,
        redis_conn: &RedisConn,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_conn, guild)?;
        let config = cache::get_config(conn, redis_conn, guild)?;

        if self.has_manage_guild(conn, redis_conn, guild).is_ok() {
            return Ok(());
        }

        if config
            .playlist_roles
            .iter()
            .any(|role| member.roles.contains(role))
        {
            return Ok(());
        }

        Err(ApiResponse::forbidden())
    }

    pub fn has_manage_player(
        &self,
        conn: &PgConnection,
        redis_conn: &RedisConn,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_conn, guild)?;
        let config = cache::get_config(conn, redis_conn, guild)?;

        if self.has_manage_guild(conn, redis_conn, guild).is_ok() {
            return Ok(());
        }

        if config
            .player_roles
            .iter()
            .any(|role| member.roles.contains(role))
        {
            return Ok(());
        }

        Err(ApiResponse::forbidden())
    }

    pub fn has_manage_queue(
        &self,
        conn: &PgConnection,
        redis_conn: &RedisConn,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_conn, guild)?;
        let config = cache::get_config(conn, redis_conn, guild)?;

        if self.has_manage_guild(conn, redis_conn, guild).is_ok() {
            return Ok(());
        }

        if config
            .queue_roles
            .iter()
            .any(|role| member.roles.contains(role))
        {
            return Ok(());
        }

        Err(ApiResponse::forbidden())
    }

    pub fn has_manage_track(
        &self,
        conn: &PgConnection,
        redis_conn: &RedisConn,
        guild: u64,
    ) -> ApiResult<()> {
        if self.has_manage_guild(conn, redis_conn, guild).is_ok() {
            return Ok(());
        }

        let member = self.get_member(redis_conn, guild)?;
        let config = cache::get_config(conn, redis_conn, guild)?;

        if config
            .track_roles
            .iter()
            .any(|role| member.roles.contains(role))
        {
            return Ok(());
        }

        Err(ApiResponse::forbidden())
    }

    pub fn has_read_guild(&self, conn: &RedisConn, guild: u64) -> ApiResult<()> {
        if self.has_bot_admin(conn).is_ok() {
            Message::get_guild(guild)
                .send_and_pause(conn)?
                .into_not_found()?;

            return Ok(());
        }

        self.get_member(conn, guild)?;

        Ok(())
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut cookies = request.cookies();
        let token = cookies.get_private(COOKIE_NAME);

        if let Some(token) = token {
            let conn = request.guard::<RedisConn>()?;
            let user = Self::from_token(&conn, token.value());
            if let Ok(user) = user {
                if let Ok(Some(_)) = get_blacklist_item(&conn, user.user.id as u64) {
                    return Outcome::Failure((Status::Forbidden, ()));
                }
                return Outcome::Success(user);
            } else {
                cookies.remove_private(get_remove_token_cookie());
            }
        }

        let headers = request.headers();
        if let Some(auth) = headers.get_one(AUTHORIZATION.as_str()) {
            if auth == get_api_secret() {
                let id = headers.get_one("User-Id").unwrap();
                let username = headers.get_one("User-Username").unwrap();
                let discriminator = headers.get_one("User-Discriminator").unwrap();
                let avatar = headers.get_one("User-Avatar").unwrap_or("");

                let conn = request.guard::<RedisConn>()?;
                let user = Self::from_bot(&conn, id, username, discriminator, avatar);

                return Outcome::Success(user);
            }
        }

        Outcome::Failure((Status::Unauthorized, ()))
    }
}
