use crate::{
    auth::User,
    db::{PgPool, RedisPool},
    error::ApiResult,
    mail::Client,
    models::{Account, AccountStatus, Member, MemberRole, Space},
    routes::{ApiResponse, ResultExt},
};

use actix_web::{
    delete, get, patch, post, put,
    web::{Data, Json},
};
use actix_web_lab::extract::Path;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub email: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordData {
    pub old_password: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyData {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct SpaceData {
    pub id: u64,
}

#[get("/{id}")]
pub async fn get_user(
    _user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    if let Some(account) = Account::find(&pool, id as i64).await? {
        ApiResponse::ok()
            .data(account.to_json(&["email", "password"])?)
            .finish()
    } else {
        ApiResponse::not_found().finish()
    }
}

#[get("/me")]
pub async fn get_user_me(user: User) -> ApiResult<ApiResponse> {
    ApiResponse::ok()
        .data(user.to_json(&["password"])?)
        .finish()
}

#[patch("/me")]
pub async fn patch_user_me(
    mut user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Json(data): Json<UserData>,
) -> ApiResult<ApiResponse> {
    if let Some(email) = data.email.as_deref() {
        if Account::find_by_email(&pool, email.to_string())
            .await?
            .is_some()
        {
            return ApiResponse::bad_request()
                .message("Email is already registered.")
                .finish();
        }

        user.set_email(email.to_string());
        user.set_status(AccountStatus::Normal);
    }

    if let Some(username) = data.username {
        if Account::find_by_username(&pool, username.clone())
            .await?
            .is_some()
        {
            return ApiResponse::bad_request()
                .message("Username is taken.")
                .finish();
        }

        user.set_username(username);
    }

    if let Some(description) = data.description {
        user.set_description(description);
    }

    user.validate()?;
    user.update(&pool).await?;

    if data.email.is_some() {
        let token = user.generate_verify_token(&redis_pool).await?;

        Client::new()
            .send_verify(user.email.as_str(), user.username.as_str(), token.as_str())
            .await?;
    }

    ApiResponse::ok().finish()
}

#[put("/me/password")]
pub async fn put_user_me_password(
    mut user: User,
    pool: Data<PgPool>,
    Json(data): Json<PasswordData>,
) -> ApiResult<ApiResponse> {
    if !user.check_password(data.old_password.as_str())? {
        return ApiResponse::bad_request()
            .message("Old password is invalid")
            .finish();
    }

    Account::validate_password(data.password.as_str())?;

    user.set_password(data.password)?;
    user.update(&pool).await?;

    Client::new()
        .send_password(user.email.as_str(), user.username.as_str())
        .await?;

    ApiResponse::ok().finish()
}

#[post("/me/verify")]
pub async fn post_user_me_verify(
    mut user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Json(data): Json<VerifyData>,
) -> ApiResult<ApiResponse> {
    if user.status() == AccountStatus::Verified {
        return ApiResponse::bad_request()
            .message("Email is already verified.")
            .finish();
    }

    if !user
        .validate_verify_token(&redis_pool, data.token.as_str())
        .await?
    {
        return ApiResponse::bad_request()
            .message("Verify link is invalid or has expired.")
            .finish();
    }

    user.set_status(AccountStatus::Verified);
    user.update(&pool).await?;

    ApiResponse::ok().finish()
}

#[post("/me/verify/resend")]
pub async fn post_user_me_verify_resend(
    user: User,
    redis_pool: Data<RedisPool>,
) -> ApiResult<ApiResponse> {
    if user.status() == AccountStatus::Verified {
        return ApiResponse::bad_request()
            .message("Email is already verified.")
            .finish();
    }

    let token = user.generate_verify_token(&redis_pool).await?;

    Client::new()
        .send_verify(user.email.as_str(), user.username.as_str(), token.as_str())
        .await?;

    ApiResponse::ok().finish()
}

#[get("/me/spaces")]
pub async fn get_user_me_spaces(user: User, pool: Data<PgPool>) -> ApiResult<ApiResponse> {
    let mut members = Member::filter_by_account(&pool, user.id).await?;
    let mut spaces = Space::find_batch(&pool, members.iter().map(|x| x.space).collect()).await?;

    if spaces.len() != members.len() {
        return ApiResponse::internal_server_error().finish();
    }

    spaces.sort_by(|a, b| a.id.cmp(&b.id));
    members.sort_by(|a, b| a.space.cmp(&b.space));

    let values = spaces
        .into_iter()
        .zip(members.into_iter())
        .map(|(space, member)| {
            let mut value = space.to_json(&[])?;
            value
                .as_object_mut()
                .unwrap()
                .insert("member".to_string(), member.to_json(&["space", "account"])?);
            Ok(value)
        })
        .collect::<ApiResult<Vec<Value>>>()?;

    ApiResponse::ok().data(values).finish()
}

#[post("/me/spaces")]
pub async fn post_user_me_spaces(
    user: User,
    pool: Data<PgPool>,
    Json(data): Json<SpaceData>,
) -> ApiResult<ApiResponse> {
    let space = Space::find(&pool, data.id as i64).await?.or_bad_request()?;

    if let Some(mut member) = Member::find(&pool, data.id as i64, user.id).await? {
        if member.role() != MemberRole::Invited {
            return ApiResponse::bad_request()
                .message("You are already in the space.")
                .finish();
        }

        member.set_role(MemberRole::Member);

        member.update(&pool).await?;

        return ApiResponse::ok().finish();
    }

    if !space.public {
        return ApiResponse::bad_request().finish();
    }

    let member = Member::new(space.id, user.id, MemberRole::Guest);

    member.create(&pool).await?;

    // TODO: impose max limit

    ApiResponse::ok().finish()
}

#[get("/me/spaces/{id}")]
pub async fn get_user_me_space(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    let space = Space::find(&pool, id as i64).await?.or_not_found()?;
    let member = Member::find(&pool, id as i64, user.id)
        .await?
        .or_not_found()?;

    let mut value = space.to_json(&[])?;
    value
        .as_object_mut()
        .unwrap()
        .insert("member".to_string(), member.to_json(&["space", "account"])?);

    ApiResponse::ok().data(value).finish()
}

#[delete("/me/spaces/{id}")]
pub async fn delete_user_me_space(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    let member = Member::find(&pool, id as i64, user.id)
        .await?
        .or_not_found()?;

    if member.role() == MemberRole::Owner {
        return ApiResponse::bad_request()
            .message("You cannot leave your own space.")
            .finish();
    }

    member.delete(&pool).await?;

    ApiResponse::ok().finish()
}
