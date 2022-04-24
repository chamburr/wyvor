use crate::{
    db::{schema::account, PgPool},
    db_run,
    error::ApiResult,
    models,
    routes::ApiResponse,
};

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(
    Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Insertable, AsChangeset,
)]
#[table_name = "account"]
pub struct Account {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub description: String,
    pub password: String,
    pub status: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, FromPrimitive)]
pub enum AccountStatus {
    Normal = 0,
    Verified = 1,
    Disabled = 2,
    Deleted = 3,
}

impl Account {
    pub fn new(email: String, username: String) -> Self {
        Self {
            id: models::generate_id(),
            email: email.to_lowercase(),
            username,
            description: "".to_string(),
            password: "".to_string(),
            status: 0,
            created_at: Utc::now().naive_utc(),
        }
    }

    pub fn status(&self) -> AccountStatus {
        FromPrimitive::from_i32(self.status).unwrap()
    }

    pub fn set_email(&mut self, email: String) {
        self.email = email.to_lowercase()
    }

    pub fn set_username(&mut self, username: String) {
        self.username = username
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description
    }

    pub fn set_password(&mut self, password: String) -> ApiResult<()> {
        self.password = bcrypt::hash(password.as_bytes(), 12)?;
        Ok(())
    }

    pub fn set_status(&mut self, status: AccountStatus) {
        self.status = status as i32;
    }

    pub fn check_password(&self, password: &str) -> ApiResult<bool> {
        Ok(bcrypt::verify(password.as_bytes(), self.password.as_str())?)
    }

    pub fn validate(&self) -> ApiResult<()> {
        if !validator::validate_range(self.username.chars().count(), Some(2), Some(32)) {
            return Err(ApiResponse::bad_request()
                .message("Username must be between 2 and 32 characters long.")
                .into());
        }

        if !validator::validate_email(self.email.as_str()) {
            return Err(ApiResponse::bad_request()
                .message("Email must be a valid email address.")
                .into());
        }

        if !validator::validate_range(self.description.chars().count(), None, Some(256)) {
            return Err(ApiResponse::bad_request()
                .message("Description must be at most 256 characters long.")
                .into());
        }

        Ok(())
    }

    pub fn validate_password(password: &str) -> ApiResult<()> {
        let length = password.chars().count();
        let num_length = password.chars().filter(|x| x.is_digit(10)).count();

        if !validator::validate_range(length, Some(8), Some(64)) {
            return Err(ApiResponse::bad_request()
                .message("Password must be between 8 and 64 characters long.")
                .into());
        }

        if !validator::validate_range(num_length, Some(1), Some(length - 1)) {
            return Err(ApiResponse::bad_request()
                .message("Password must contain both letters and numbers.")
                .into());
        }

        Ok(())
    }

    pub fn to_json(&self, exclude: &[&str]) -> ApiResult<Value> {
        models::to_json(self, exclude)
    }
}

impl Account {
    pub async fn find(pool: &PgPool, id: i64) -> ApiResult<Option<Self>> {
        db_run!(pool, account::table.find(id).first(&*pool).optional())
    }

    pub async fn find_batch(pool: &PgPool, ids: Vec<i64>) -> ApiResult<Vec<Self>> {
        db_run!(
            pool,
            account::table.filter(account::id.eq_any(ids)).load(&*pool)
        )
    }

    pub async fn find_by_email(pool: &PgPool, email: String) -> ApiResult<Option<Self>> {
        db_run!(
            pool,
            account::table
                .filter(account::email.eq(email.to_lowercase()))
                .first(&*pool)
                .optional()
        )
    }

    pub async fn find_by_username(pool: &PgPool, username: String) -> ApiResult<Option<Self>> {
        db_run!(
            pool,
            account::table
                .filter(account::username.ilike(username))
                .first(&*pool)
                .optional()
        )
    }

    pub async fn create(&self, pool: &PgPool) -> ApiResult<usize> {
        let record = self.clone();

        db_run!(
            pool,
            diesel::insert_into(account::table)
                .values(&record)
                .execute(&*pool)
        )
    }

    pub async fn update(&self, pool: &PgPool) -> ApiResult<usize> {
        let record = self.clone();

        db_run!(pool, diesel::update(&record).set(&record).execute(&*pool))
    }
}
