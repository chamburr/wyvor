use crate::{
    auth::User,
    db::PgPool,
    error::ApiResult,
    models::{Member, MemberRole, Space},
    routes::{ApiResponse, ResultExt},
};

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json},
};
use actix_web_lab::extract::Path;
use serde::Deserialize;

mod member;
mod playlist;
// mod player;
// mod queue;

pub use member::*;
pub use playlist::*;
// pub use player::*;
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

    ApiResponse::ok().data(space.to_json(&[])).finish()
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

    ApiResponse::ok().data(space.to_json(&[])).finish()
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
