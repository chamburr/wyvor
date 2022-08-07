use crate::{auth::User, db::PgPool, error::ApiResult, models::Account, routes::ApiResponse};

use actix_web::{get, web::Data};
use actix_web_lab::extract::Path;

mod me;

pub use me::*;

#[get("/{id}")]
pub async fn get_user(
    _user: User,
    pool: Data<PgPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    if let Some(account) = Account::find(&pool, id as i64).await? {
        ApiResponse::ok()
            .data(account.to_json(&["email", "password"]))
            .finish()
    } else {
        ApiResponse::not_found().finish()
    }
}
