use crate::{
    auth::User,
    db::{PgPool, RedisPool},
    error::ApiResult,
    mail::Client,
    models::{Account, Space},
    routes::ApiResponse,
};

use actix_web::{
    get, post,
    web::{Data, Json},
    HttpRequest,
};
use http::header::AUTHORIZATION;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct RegisterData {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotData {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetData {
    pub token: String,
    pub id: i64,
    pub password: String,
}

#[get("")]
pub async fn get_index() -> ApiResult<ApiResponse> {
    ApiResponse::ok().finish()
}

// TODO: rate limiting

#[post("/register")]
pub async fn post_auth_register(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Json(data): Json<RegisterData>,
) -> ApiResult<ApiResponse> {
    if Account::find_by_email(&pool, data.email.clone())
        .await?
        .is_some()
    {
        return ApiResponse::bad_request()
            .message("Email is already registered.")
            .finish();
    }

    if Account::find_by_username(&pool, data.username.clone())
        .await?
        .is_some()
    {
        return ApiResponse::bad_request()
            .message("Username is taken.")
            .finish();
    }

    let mut account = Account::new(data.email, data.username);

    account.validate()?;
    Account::validate_password(data.password.as_str())?;

    account.set_password(data.password)?;
    account.create(&pool).await?;

    let space = Space::new(format!("{}'s Space", account.username));
    space.create(&pool).await?;

    let user = User::new(account);
    let token = user.generate_verify_token(&redis_pool).await?;

    Client::new()
        .send_welcome(user.email.as_str(), user.username.as_str(), token.as_str())
        .await?;

    ApiResponse::ok()
        .data(user.to_json(&["password"])?)
        .finish()
}

#[post("/login")]
pub async fn post_auth_login(
    pool: Data<PgPool>,
    Json(data): Json<LoginData>,
) -> ApiResult<ApiResponse> {
    if let Some(account) = Account::find_by_email(&pool, data.email).await? {
        if account.check_password(data.password.as_str())? {
            let user = User::new(account);
            let token = user.generate_token()?;

            return ApiResponse::ok().data(json!({ "token": token })).finish();
        }
    }

    ApiResponse::bad_request()
        .message("Email or password is invalid.")
        .finish()
}

#[post("/forgot")]
pub async fn post_auth_forgot(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Json(data): Json<ForgotData>,
) -> ApiResult<ApiResponse> {
    if let Some(account) = Account::find_by_email(&pool, data.email).await? {
        let user = User::new(account);
        let token = user.generate_reset_token(&redis_pool).await?;

        Client::new()
            .send_reset(
                user.email.as_str(),
                user.username.as_str(),
                token.as_str(),
                user.id,
            )
            .await?;
    }

    ApiResponse::ok().finish()
}

#[post("/reset")]
pub async fn post_auth_reset(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Json(data): Json<ResetData>,
) -> ApiResult<ApiResponse> {
    if let Some(account) = Account::find(&pool, data.id).await? {
        let mut user = User::new(account);

        if user
            .validate_reset_token(&redis_pool, data.token.as_str())
            .await?
        {
            Account::validate_password(data.password.as_str())?;

            user.set_password(data.password)?;
            user.update(&pool).await?;

            Client::new()
                .send_password(user.email.as_str(), user.username.as_str())
                .await?;

            return ApiResponse::ok().finish();
        }
    }

    ApiResponse::bad_request()
        .message("Reset link is invalid or has expired.")
        .finish()
}

#[post("/refresh")]
pub async fn post_auth_refresh(user: User) -> ApiResult<ApiResponse> {
    let token = user.generate_token()?;

    ApiResponse::ok().data(json!({ "token": token })).finish()
}

#[post("/logout")]
pub async fn post_auth_logout(
    req: HttpRequest,
    redis_pool: Data<RedisPool>,
    user: User,
) -> ApiResult<ApiResponse> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .map(|x| x.to_str())
        .transpose()?
        .unwrap_or("");

    user.invalidate_token(&redis_pool, token).await?;

    ApiResponse::ok().finish()
}

pub async fn default_service() -> ApiResult<ApiResponse> {
    ApiResponse::not_found().finish()
}
