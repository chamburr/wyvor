use crate::{
    db::{schema::space, PgPool},
    db_run,
    error::ApiResult,
    models,
    routes::ApiResponse,
};

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Queryable, Identifiable, Insertable, AsChangeset)]
#[table_name = "space"]
pub struct Space {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub public: bool,
    pub created_at: NaiveDateTime,
}

impl Space {
    pub fn new(name: String) -> Self {
        Self {
            id: models::generate_id(),
            name,
            description: "".to_string(),
            public: false,
            created_at: Utc::now().naive_utc(),
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description
    }

    pub fn set_public(&mut self, public: bool) {
        self.public = public
    }

    pub fn validate(&self) -> ApiResult<()> {
        if !validator::validate_range(self.name.chars().count(), Some(1), Some(64)) {
            return Err(ApiResponse::bad_request()
                .message("Name must be between 1 and 64 characters long.")
                .into());
        }

        if !validator::validate_range(self.description.chars().count(), None, Some(256)) {
            return Err(ApiResponse::bad_request()
                .message("Description must be at most 256 characters long.")
                .into());
        }

        Ok(())
    }

    pub fn to_json(&self, exclude: &[&str]) -> ApiResult<Value> {
        models::to_json(self, exclude)
    }
}

impl Space {
    pub async fn find(pool: &PgPool, id: i64) -> ApiResult<Option<Self>> {
        db_run!(pool, space::table.find(id).first(&*pool).optional())
    }

    pub async fn find_batch(pool: &PgPool, ids: Vec<i64>) -> ApiResult<Vec<Self>> {
        db_run!(
            pool,
            space::table.filter(space::id.eq_any(ids)).load(&*pool)
        )
    }

    pub async fn create(&self, pool: &PgPool) -> ApiResult<usize> {
        let record = self.clone();

        db_run!(
            pool,
            diesel::insert_into(space::table)
                .values(&record)
                .execute(&*pool)
        )
    }

    pub async fn update(&self, pool: &PgPool) -> ApiResult<usize> {
        let record = self.clone();

        db_run!(pool, diesel::update(&record).set(&record).execute(&*pool))
    }

    pub async fn delete(&self, pool: &PgPool) -> ApiResult<usize> {
        let record = self.clone();

        db_run!(pool, diesel::delete(&record).execute(&*pool))
    }
}
