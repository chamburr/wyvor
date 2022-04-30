use crate::{
    db::{schema::playlist, PgPool},
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
#[table_name = "playlist"]
pub struct Playlist {
    pub id: i64,
    pub space: i64,
    pub name: String,
    pub items: Vec<i32>,
    pub created_at: NaiveDateTime,
}

impl Playlist {
    pub fn new(space: i64, name: String, items: Vec<i32>) -> Self {
        Self {
            id: models::generate_id(),
            space,
            name,
            items,
            created_at: Utc::now().naive_utc(),
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn validate(&self) -> ApiResult<()> {
        if validator::validate_range(self.name.chars().count(), Some(1), Some(64)) {
            return Err(ApiResponse::bad_request()
                .message("Name must be between 1 and 64 characters long.")
                .into());
        }

        Ok(())
    }

    pub fn to_json(&self, exclude: &[&str]) -> Value {
        models::to_json(self, exclude)
    }
}

impl Playlist {
    pub async fn find(pool: &PgPool, id: i64) -> ApiResult<Option<Self>> {
        db_run!(pool, playlist::table.find(id).first(&*pool).optional())
    }

    pub async fn filter_by_space(pool: &PgPool, space: i64) -> ApiResult<Vec<Self>> {
        db_run!(
            pool,
            playlist::table
                .filter(playlist::space.eq(space))
                .load(&*pool)
        )
    }

    pub async fn create(&self, pool: &PgPool) -> ApiResult<usize> {
        let record = self.clone();

        db_run!(
            pool,
            diesel::insert_into(playlist::table)
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
