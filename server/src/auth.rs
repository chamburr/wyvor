use crate::{
    config::CONFIG,
    db::{cache, PgPool, RedisPool},
    error::{ApiError, ApiResult},
    models::{Account, Member, MemberRole},
    routes::ApiResponse,
};

use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};
use chrono::{Duration, Utc};
use http::header::AUTHORIZATION;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    account: i64,
    iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyClaims {
    account: i64,
    email: String,
    iat: i64,
}

#[derive(Debug)]
pub struct User(Account);

impl Deref for User {
    type Target = Account;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for User {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl User {
    async fn from_token(
        pool: &PgPool,
        redis_pool: &RedisPool,
        token: &str,
    ) -> ApiResult<Option<Self>> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();

        let claims =
            jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(&[]), &validation)?
                .claims;

        if claims.iat < (Utc::now() - Duration::days(7)).timestamp_millis() {
            return Ok(None);
        }

        if let Some(account) = Account::find(pool, claims.account).await? {
            let user = Self::new(account);
            let invalid: Option<String> =
                cache::get(redis_pool, format!("invalid_token:{}", token)).await?;
            let signature = jsonwebtoken::decode::<Claims>(
                token,
                &DecodingKey::from_base64_secret(user.secret()?.as_str())?,
                &Validation::default(),
            );

            if invalid.is_none() && signature.is_ok() {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    fn secret(&self) -> ApiResult<String> {
        Ok(base64::encode(
            CONFIG
                .api_secret
                .as_str()
                .as_bytes()
                .iter()
                .chain(self.password.as_bytes())
                .cloned()
                .collect::<Vec<u8>>(),
        ))
    }

    async fn has_permission(&self, pool: &PgPool, space: i64, role: MemberRole) -> ApiResult<()> {
        if let Some(member) = Member::find(pool, space, self.id).await? {
            if member.role() >= role && member.role() != MemberRole::Invited {
                Ok(())
            } else {
                Err(ApiResponse::forbidden().into())
            }
        } else {
            Err(ApiResponse::not_found().into())
        }
    }
}

impl User {
    pub fn new(account: Account) -> Self {
        Self(account)
    }

    pub fn generate_token(&self) -> ApiResult<String> {
        Ok(jsonwebtoken::encode(
            &Header::default(),
            &Claims {
                account: self.id,
                iat: Utc::now().timestamp_millis(),
            },
            &EncodingKey::from_base64_secret(self.secret()?.as_str())?,
        )?)
    }

    pub async fn invalidate_token(&self, pool: &RedisPool, token: &str) -> ApiResult<()> {
        let claims = jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_base64_secret(self.secret()?.as_str())?,
            &Validation::default(),
        )?
        .claims;

        let expiry =
            claims.iat + Duration::days(7).num_milliseconds() - Utc::now().timestamp_millis();

        cache::set_ex(
            pool,
            format!("invalid_token:{}", token),
            &"".to_string(),
            expiry as usize,
        )
        .await?;

        Ok(())
    }

    pub async fn generate_verify_token(&self, pool: &RedisPool) -> ApiResult<String> {
        let token = nanoid!(16);

        cache::set_ex(
            pool,
            format!("verify_token:{}", token),
            self.deref(),
            Duration::days(1).num_milliseconds() as usize,
        )
        .await?;

        Ok(token)
    }

    pub async fn generate_reset_token(&self, pool: &RedisPool) -> ApiResult<String> {
        let token = nanoid!(16);

        cache::set_ex(
            pool,
            format!("reset_token:{}", token),
            self.deref(),
            Duration::hours(1).num_milliseconds() as usize,
        )
        .await?;

        Ok(token)
    }

    pub async fn validate_verify_token(&self, pool: &RedisPool, token: &str) -> ApiResult<bool> {
        let key = format!("verify_token:{}", token);
        let account: Option<Account> = cache::get(pool, key.as_str()).await?;

        if let Some(account) = account {
            if account.id == self.id && account.email == self.email {
                return Ok(true);
            } else {
                cache::del(pool, key.as_str()).await?;
            }
        }

        Ok(false)
    }

    pub async fn validate_reset_token(&self, pool: &RedisPool, token: &str) -> ApiResult<bool> {
        let key = format!("reset_token:{}", token);
        let account: Option<Account> = cache::get(pool, key.as_str()).await?;

        if let Some(account) = account {
            if account.password == self.password {
                return Ok(true);
            } else {
                cache::del(pool, key.as_str()).await?;
            }
        }

        Ok(false)
    }

    pub async fn can_read_space(&self, pool: &PgPool, space: i64) -> ApiResult<()> {
        self.has_permission(pool, space, MemberRole::Guest).await
    }

    pub async fn can_write_space(&self, pool: &PgPool, space: i64) -> ApiResult<()> {
        self.has_permission(pool, space, MemberRole::Member).await
    }

    pub async fn can_manage_space(&self, pool: &PgPool, space: i64) -> ApiResult<()> {
        self.has_permission(pool, space, MemberRole::Admin).await
    }

    pub async fn can_delete_space(&self, pool: &PgPool, space: i64) -> ApiResult<()> {
        self.has_permission(pool, space, MemberRole::Owner).await
    }
}

impl FromRequest for User {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let pool = req.app_data::<Data<PgPool>>().unwrap().clone();
        let redis_pool = req.app_data::<Data<RedisPool>>().unwrap().clone();
        let token = req.headers().get(AUTHORIZATION).cloned();

        Box::pin(async move {
            if let Some(token) = token {
                if let Some(user) = Self::from_token(&pool, &redis_pool, token.to_str()?).await? {
                    Ok(user)
                } else {
                    Err(ApiResponse::unauthorized()
                        .message("Session has expired, please login again.")
                        .into())
                }
            } else {
                Err(ApiResponse::unauthorized().into())
            }
        })
    }
}
