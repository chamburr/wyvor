use crate::config::CONFIG;
use crate::constants::{
    csrf_token_key, guild_key, user_guilds_key, user_key, user_token_key, BOT_ADMIN_KEY,
    BOT_OWNER_KEY, CALLBACK_PATH, COOKIE_NAME, CSRF_TOKEN_KEY_TTL, USER_GUILDS_KEY_TTL,
    USER_KEY_TTL, USER_TOKEN_KEY_TTL,
};
use crate::db::cache::{self, get_blacklist_item};
use crate::db::pubsub::models::{Connected, Member, Permission};
use crate::db::pubsub::Message;
use crate::db::{PgPool, RedisPool};
use crate::models::account::Account;
use crate::models::guild::NewGuild;
use crate::routes::{ApiError, ApiResponse, ApiResult, OptionExt};

use actix_web::cookie::{Cookie, SameSite};
use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use chrono::Utc;
use http::header::AUTHORIZATION;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use nanoid::nanoid;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;
use twilight_model::guild::Permissions;
use twilight_model::id::{ApplicationId, GuildId};
use twilight_oauth2::request::access_token_exchange::AccessTokenExchangeResponse;
use twilight_oauth2::{Client, Scope};
use url::Url;

lazy_static! {
    static ref BASE_URI: String = {
        let uri = Url::parse(CONFIG.base_uri.as_str()).unwrap();
        uri.into_string()
    };
    static ref REDIRECT_URI: String = {
        let mut uri = Url::parse(CONFIG.base_uri.as_str()).unwrap();
        uri.set_path(CALLBACK_PATH);
        uri.into_string()
    };
    static ref OAUTH_CLIENT: Client = {
        let mut uri = Url::parse(CONFIG.base_uri.as_str()).unwrap();
        uri.set_path(CALLBACK_PATH);
        Client::new(
            ApplicationId(CONFIG.bot_client_id),
            CONFIG.bot_client_secret.as_str(),
            &[uri.into_string().as_str()],
        )
        .unwrap()
    };
    static ref OAUTH_SCOPES: Vec<Scope> = vec![Scope::Identify, Scope::Guilds];
}

pub async fn get_csrf_redirect(pool: &RedisPool, token: &str) -> ApiResult<Option<String>> {
    let token = cache::get(pool, csrf_token_key(token)).await?;

    Ok(token)
}

pub async fn get_redirect_uri(pool: &RedisPool, redirect: Option<String>) -> ApiResult<String> {
    let mut uri = OAUTH_CLIENT.authorization_url(REDIRECT_URI.as_str())?;
    uri.scopes(OAUTH_SCOPES.as_slice());

    let id = nanoid!();
    if let Some(redirect) = redirect {
        cache::set_and_expire(
            pool,
            csrf_token_key(id.as_str()),
            &redirect,
            CSRF_TOKEN_KEY_TTL,
        )
        .await?;
        uri.state(id.as_str());
    }

    Ok(uri.build().replace("%20", "+"))
}

pub fn get_invite_uri(guild: Option<u64>) -> ApiResult<String> {
    let mut uri = OAUTH_CLIENT.bot_authorization_url();
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
    uri.redirect_uri(REDIRECT_URI.as_str())?;

    if let Some(guild) = guild {
        uri.guild_id(GuildId(guild));
    }

    Ok(uri.build().replace("%20", "+"))
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    token: String,
    iat: u64,
}

pub async fn token_exchange(code: &str) -> ApiResult<AccessTokenExchangeResponse> {
    let mut builder = OAUTH_CLIENT.access_token_exchange(code, REDIRECT_URI.as_str())?;
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

    let response = reqwest::Client::new()
        .post(request.url_base)
        .form(&request.body)
        .headers(headers)
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}

pub async fn get_token_cookie(exchange: AccessTokenExchangeResponse) -> ApiResult<Cookie<'static>> {
    let url = Url::parse(BASE_URI.as_str())?;
    let domain = url.domain().unwrap_or_default().to_owned();

    let token = encode(
        &Header::default(),
        &Claims {
            token: exchange.access_token,
            iat: Utc::now().timestamp_millis() as u64,
        },
        &EncodingKey::from_base64_secret(CONFIG.api_secret.as_str())?,
    )?;

    let cookie = Cookie::build(COOKIE_NAME, token)
        .domain(domain)
        .http_only(true)
        .max_age(
            Duration::from_secs(exchange.expires_in)
                .try_into()
                .unwrap_or_default(),
        )
        .same_site(SameSite::Lax)
        .secure(true)
        .finish();

    Ok(cookie)
}

pub struct User {
    pub token: String,
    pub user: Account,
    pub is_bot: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Guild {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub permissions: i64,
    pub has_bot: bool,
}

impl User {
    async fn from_token(pool: &RedisPool, token: &str) -> ApiResult<Self> {
        let user: Option<u64> = cache::get(pool, user_token_key(token))
            .await
            .unwrap_or(None);

        if let Some(user) = user {
            let user: Option<Account> = cache::get(pool, user_key(user)).await.unwrap_or(None);

            if let Some(user) = user {
                return Ok(Self {
                    token: token.to_owned(),
                    user,
                    is_bot: false,
                });
            }
        }

        let client = twilight_http::Client::new(format!("Bearer {}", token));
        let user = client.current_user().await?;
        let user = Account {
            id: user.id.0 as i64,
            username: user.name,
            discriminator: user.discriminator.parse()?,
            avatar: user.avatar.unwrap_or_else(|| "".to_owned()),
        };

        cache::set_and_expire(
            pool,
            user_token_key(token),
            &(user.id as u64),
            USER_TOKEN_KEY_TTL,
        )
        .await?;
        cache::set_and_expire(pool, user_key(user.id as u64), &user, USER_KEY_TTL).await?;

        Ok(Self {
            token: token.to_owned(),
            user,
            is_bot: false,
        })
    }

    async fn from_bot(
        pool: &RedisPool,
        id: &str,
        username: &str,
        discriminator: &str,
        avatar: &str,
    ) -> ApiResult<Self> {
        let user = Account {
            id: id.parse()?,
            username: username.to_owned(),
            discriminator: discriminator.parse()?,
            avatar: avatar.to_owned(),
        };

        cache::set_and_expire(pool, user_key(user.id as u64), &user, USER_KEY_TTL).await?;

        Ok(Self {
            token: "".to_owned(),
            user,
            is_bot: true,
        })
    }

    pub async fn revoke_token(self) -> ApiResult<()> {
        let uri = Client::BASE_URI.replace("/authorize", "/token/revoke");

        let mut params = HashMap::new();
        params.insert("client_id".to_owned(), OAUTH_CLIENT.client_id().to_string());
        params.insert(
            "client_secret".to_owned(),
            OAUTH_CLIENT.client_secret().to_string(),
        );
        params.insert("token".to_owned(), self.token);

        reqwest::Client::new()
            .post(uri.as_str())
            .form(&params)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_guilds(&self, pool: &RedisPool) -> ApiResult<Vec<Guild>> {
        let guilds: Option<Vec<Guild>> =
            cache::get(pool, user_guilds_key(self.user.id as u64)).await?;

        if let Some(guilds) = guilds {
            return Ok(guilds);
        }

        let client = twilight_http::Client::new(format!("Bearer {}", self.token));

        let mut guilds = vec![];
        for guild in client.current_user_guilds().await? {
            let bot_guild: Option<NewGuild> = cache::get(pool, guild_key(guild.id.0)).await?;
            let guild = Guild {
                id: guild.id.0 as i64,
                name: guild.name,
                icon: guild.icon,
                permissions: guild.permissions.bits() as i64,
                has_bot: bot_guild.is_some(),
            };
            guilds.push(guild);
        }

        cache::set_and_expire(
            pool,
            user_guilds_key(self.user.id as u64),
            &guilds,
            USER_GUILDS_KEY_TTL,
        )
        .await?;

        Ok(guilds)
    }

    pub async fn get_member(&self, pool: &RedisPool, guild: u64) -> ApiResult<Member> {
        let member = Message::get_member(guild, self.user.id as u64)
            .send_and_wait(pool)
            .await?
            .or_not_found()?;

        Ok(member)
    }

    pub async fn is_connected(&self, pool: &RedisPool, guild: u64, user: bool) -> ApiResult<()> {
        let connected: Option<Connected> = Message::get_connected(guild, None)
            .send_and_wait(pool)
            .await?;
        let user_connected: Option<Connected> = Message::get_connected(guild, self.user.id as u64)
            .send_and_wait(pool)
            .await?;

        if let Some(connected) = connected {
            if user {
                if let Some(user_connected) = user_connected {
                    if connected.channel == user_connected.channel {
                        Ok(())
                    } else {
                        Err(ApiResponse::bad_request()
                            .message("You need to be connected to the same channel as the bot.")
                            .into())
                    }
                } else {
                    Err(ApiResponse::bad_request()
                        .message("You need to be connected to a channel.")
                        .into())
                }
            } else {
                Ok(())
            }
        } else {
            Err(ApiResponse::bad_request()
                .message("The bot is not connected to any channel.")
                .into())
        }
    }

    pub fn is_not_bot(&self) -> ApiResult<()> {
        if !self.is_bot {
            Ok(())
        } else {
            Err(ApiResponse::forbidden().into())
        }
    }

    pub async fn has_bot_owner(&self, pool: &RedisPool) -> ApiResult<()> {
        if cache::sismember(pool, BOT_OWNER_KEY, &self.user.id).await? {
            Ok(())
        } else {
            Err(ApiResponse::forbidden().into())
        }
    }

    pub async fn has_bot_admin(&self, pool: &RedisPool) -> ApiResult<()> {
        if self.has_bot_owner(pool).await.is_ok() {
            return Ok(());
        }

        if cache::sismember(pool, BOT_ADMIN_KEY, &self.user.id).await? {
            return Ok(());
        }

        Err(ApiResponse::forbidden().into())
    }

    pub async fn has_manage_guild(
        &self,
        pool: &PgPool,
        redis_pool: &RedisPool,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_pool, guild).await?;
        let config = cache::get_config(pool, redis_pool, guild).await?;

        if self.has_bot_owner(redis_pool).await.is_ok() {
            return Ok(());
        }

        if config
            .guild_roles
            .iter()
            .any(|role| *role as u64 == guild || member.roles.contains(role))
        {
            return Ok(());
        }

        let permission = Message::get_permission(guild, self.user.id as u64, None)
            .send_and_wait::<Permission>(redis_pool)
            .await?
            .or_not_found()?
            .permission;

        if let Some(permissions) = Permissions::from_bits(permission as u64) {
            if permissions.contains(Permissions::MANAGE_GUILD) {
                Ok(())
            } else {
                Err(ApiResponse::forbidden().into())
            }
        } else {
            Err(ApiResponse::forbidden().into())
        }
    }

    pub async fn has_manage_playlist(
        &self,
        pool: &PgPool,
        redis_pool: &RedisPool,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_pool, guild).await?;
        let config = cache::get_config(pool, redis_pool, guild).await?;

        if self.has_manage_guild(pool, redis_pool, guild).await.is_ok() {
            return Ok(());
        }

        if config
            .playlist_roles
            .iter()
            .any(|role| *role as u64 == guild || member.roles.contains(role))
        {
            Ok(())
        } else {
            Err(ApiResponse::forbidden().into())
        }
    }

    pub async fn has_manage_player(
        &self,
        pool: &PgPool,
        redis_pool: &RedisPool,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_pool, guild).await?;
        let config = cache::get_config(pool, redis_pool, guild).await?;

        if self.has_manage_guild(pool, redis_pool, guild).await.is_ok() {
            return Ok(());
        }

        if config
            .player_roles
            .iter()
            .any(|role| *role as u64 == guild || member.roles.contains(role))
        {
            Ok(())
        } else {
            Err(ApiResponse::forbidden().into())
        }
    }

    pub async fn has_manage_queue(
        &self,
        pool: &PgPool,
        redis_pool: &RedisPool,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_pool, guild).await?;
        let config = cache::get_config(pool, redis_pool, guild).await?;

        if self.has_manage_guild(pool, redis_pool, guild).await.is_ok() {
            return Ok(());
        }

        if config
            .queue_roles
            .iter()
            .any(|role| *role as u64 == guild || member.roles.contains(role))
        {
            Ok(())
        } else {
            Err(ApiResponse::forbidden().into())
        }
    }

    pub async fn has_manage_track(
        &self,
        pool: &PgPool,
        redis_pool: &RedisPool,
        guild: u64,
    ) -> ApiResult<()> {
        let member = self.get_member(redis_pool, guild).await?;
        let config = cache::get_config(pool, redis_pool, guild).await?;

        if self.has_manage_guild(pool, redis_pool, guild).await.is_ok() {
            return Ok(());
        }

        if config
            .track_roles
            .iter()
            .any(|role| *role as u64 == guild || member.roles.contains(role))
        {
            Ok(())
        } else {
            Err(ApiResponse::forbidden().into())
        }
    }

    pub async fn has_read_guild(&self, pool: &RedisPool, guild: u64) -> ApiResult<()> {
        if self.has_bot_admin(pool).await.is_ok() {
            Message::get_guild(guild)
                .send_and_pause(pool)
                .await?
                .or_not_found()?;

            return Ok(());
        }

        self.get_member(pool, guild).await?;

        Ok(())
    }
}

impl FromRequest for User {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let pool = Data::<RedisPool>::from_request(req, payload).into_inner();
        let cookie: Option<Cookie<'static>> = req.cookie(COOKIE_NAME);
        let headers = req.headers().clone();

        Box::pin(async move {
            let pool = pool?;

            if let Some(cookie) = cookie {
                if let Ok(token) = decode::<Claims>(
                    cookie.value(),
                    &DecodingKey::from_base64_secret(CONFIG.api_secret.as_str())?,
                    &Validation::default(),
                ) {
                    let user = Self::from_token(&pool, token.claims.token.as_str()).await?;
                    if get_blacklist_item(&pool, user.user.id as u64)
                        .await?
                        .is_some()
                    {
                        return Err(ApiResponse::forbidden()
                            .message("You are banned from the bot.")
                            .into());
                    }

                    return Ok(user);
                }

                return Err(ApiResponse::unauthorized()
                    .message("Your session is invalid, please login again.")
                    .del_cookie(COOKIE_NAME)
                    .into());
            }

            if let Some(auth) = headers.get(AUTHORIZATION) {
                if auth.to_str()? == CONFIG.api_secret.as_str() {
                    let id = headers
                        .get("User-Id")
                        .map(|value| value.to_str())
                        .transpose()?
                        .unwrap_or_default();
                    let username = headers
                        .get("User-Username")
                        .map(|value| value.to_str())
                        .transpose()?
                        .unwrap_or_default();
                    let discriminator = headers
                        .get("User-Discriminator")
                        .map(|value| value.to_str())
                        .transpose()?
                        .unwrap_or_default();
                    let avatar = headers
                        .get("User-Avatar")
                        .map(|value| value.to_str())
                        .transpose()?
                        .unwrap_or_default();

                    let user = Self::from_bot(&pool, id, username, discriminator, avatar).await?;

                    return Ok(user);
                }
            }

            Err(ApiResponse::unauthorized().into())
        })
    }
}
