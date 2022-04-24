use crate::{
    auth::User,
    db::PgPool,
    error::ApiResult,
    mail::Client,
    models::{Account, Member, MemberRole, Space},
    routes::{ApiResponse, ResultExt},
};

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json},
};
use actix_web_lab::extract::Path;
use num_traits::FromPrimitive;
use serde::Deserialize;
use serde_json::Value;

// mod player;
// mod playlist;
// mod queue;

// pub use player::*;
// pub use playlist::*;
// pub use queue::*;

#[derive(Debug, Deserialize)]
pub struct NewSpaceData {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SpaceData {
    pub name: Option<String>,
    pub description: Option<String>,
    pub public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct NewMemberData {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct MemberData {
    pub role: Option<u8>,
}

#[post("/")]
pub async fn post_spaces(
    user: User,
    pool: Data<PgPool>,
    Json(data): Json<NewSpaceData>,
) -> ApiResult<ApiResponse> {
    // TODO: impose max limit

    let space = Space::new(data.name);
    let member = Member::new(space.id, user.id, MemberRole::Owner);

    space.create(&pool).await?;

    if let Err(err) = member.create(&pool).await {
        space.delete(&pool).await?;
        return Err(err);
    }

    ApiResponse::ok().data(space.to_json(&[])?).finish()
}

#[get("/{id}")]
pub async fn get_space(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    let space = Space::find(&pool, id as i64).await?.or_not_found()?;

    if !space.public {
        user.can_read_space(&pool, id as i64).await?;
    }

    ApiResponse::ok().data(space.to_json(&[])?).finish()
}

#[patch("/{id}")]
pub async fn patch_space(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
    Json(data): Json<SpaceData>,
) -> ApiResult<ApiResponse> {
    user.can_manage_space(&pool, id as i64).await?;

    let mut space = Space::find(&pool, id as i64)
        .await?
        .or_internal_server_error()?;

    if let Some(name) = data.name {
        space.set_name(name);
    }

    if let Some(description) = data.description {
        space.set_description(description);
    }

    if let Some(public) = data.public {
        space.set_public(public);
    }

    space.validate()?;

    space.update(&pool).await?;

    ApiResponse::ok().finish()
}

#[delete("/{id}")]
pub async fn delete_space(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_delete_space(&pool, id as i64).await?;

    let space = Space::find(&pool, id as i64)
        .await?
        .or_internal_server_error()?;

    space.delete(&pool).await?;

    ApiResponse::ok().finish()
}

#[get("/{id}/members")]
pub async fn get_space_members(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.can_manage_space(&pool, id as i64).await?;

    let mut members = Member::filter_by_space(&pool, id as i64).await?;
    let mut accounts =
        Account::find_batch(&pool, members.iter().map(|x| x.account).collect()).await?;

    members.sort_by(|a, b| a.account.cmp(&b.account));
    accounts.sort_by(|a, b| a.id.cmp(&b.id));

    let values = members
        .into_iter()
        .zip(accounts.into_iter())
        .map(|(member, account)| {
            let mut value = member.to_json(&["space", "account"])?;
            value.as_object_mut().unwrap().insert(
                "account".to_string(),
                account.to_json(&["email", "password"])?,
            );
            Ok(value)
        })
        .collect::<ApiResult<Vec<Value>>>()?;

    ApiResponse::ok().data(values).finish()
}

#[post("/{id}/members")]
pub async fn post_space_members(
    user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
    Json(data): Json<NewMemberData>,
) -> ApiResult<ApiResponse> {
    user.can_manage_space(&pool, id as i64).await?;

    let account = Account::find_by_username(&pool, data.username)
        .await?
        .or_not_found()?;

    if Member::find(&pool, id as i64, account.id).await?.is_some() {
        return ApiResponse::bad_request()
            .message("User is already in the space.")
            .finish();
    }

    // impose max members

    let space = Space::find(&pool, id as i64).await?.or_not_found()?;
    let member = Member::new(id as i64, account.id, MemberRole::Invited);

    member.create(&pool).await?;

    Client::new()
        .send_invite(
            account.email.as_str(),
            account.username.as_str(),
            space.name.as_str(),
        )
        .await?;

    ApiResponse::ok().finish()
}

#[get("/{id}/members/{item}")]
pub async fn get_space_member(
    user: User,
    pool: Data<PgPool>,
    Path((id, item)): Path<(u64, u64)>,
) -> ApiResult<ApiResponse> {
    user.can_manage_space(&pool, id as i64).await?;

    let member = Member::find(&pool, id as i64, item as i64)
        .await?
        .or_not_found()?;
    let account = Account::find(&pool, item as i64).await?.or_not_found()?;

    let mut value = member.to_json(&["space", "account"])?;
    value.as_object_mut().unwrap().insert(
        "account".to_string(),
        account.to_json(&["email", "password"])?,
    );

    ApiResponse::ok().data(value).finish()
}

#[patch("/{id}/members/{item}")]
pub async fn patch_space_member(
    user: User,
    pool: Data<PgPool>,
    Path((id, item)): Path<(u64, u64)>,
    Json(data): Json<MemberData>,
) -> ApiResult<ApiResponse> {
    user.can_manage_space(&pool, id as i64).await?;

    let mut member = Member::find(&pool, id as i64, item as i64)
        .await?
        .or_not_found()?;

    if let Some(role) = data.role {
        let role: MemberRole = FromPrimitive::from_u8(role).or_bad_request()?;

        if member.role() == MemberRole::Invited
            || role == MemberRole::Invited
            || role == MemberRole::Owner
        {
            return ApiResponse::bad_request().finish();
        }

        if member.role() == MemberRole::Owner {
            return ApiResponse::forbidden().finish();
        }

        member.set_role(role);
    }

    member.update(&pool).await?;

    ApiResponse::ok().finish()
}

#[delete("/{id}/members/{item}")]
pub async fn delete_space_member(
    user: User,
    pool: Data<PgPool>,
    Path((id, item)): Path<(u64, u64)>,
) -> ApiResult<ApiResponse> {
    user.can_manage_space(&pool, id as i64).await?;

    let member = Member::find(&pool, id as i64, item as i64)
        .await?
        .or_not_found()?;

    if member.role() == MemberRole::Owner {
        return ApiResponse::forbidden().finish();
    }

    member.delete(&pool).await?;

    ApiResponse::ok().finish()
}
