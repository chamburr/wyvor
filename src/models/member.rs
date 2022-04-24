use crate::{
    db::{schema::member, PgPool},
    db_run,
    error::ApiResult,
    models,
};

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Queryable, Identifiable, Insertable, AsChangeset)]
#[table_name = "member"]
#[primary_key(space, account)]
pub struct Member {
    pub space: i64,
    pub account: i64,
    pub role: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, FromPrimitive)]
pub enum MemberRole {
    Guest = 0,
    Invited = 1,
    Member = 2,
    Admin = 3,
    Owner = 4,
}

impl Member {
    pub fn new(space: i64, account: i64, role: MemberRole) -> Self {
        Self {
            space,
            account,
            role: role as i32,
            created_at: Utc::now().naive_utc(),
        }
    }

    pub fn role(&self) -> MemberRole {
        FromPrimitive::from_i32(self.role).unwrap()
    }

    pub fn set_role(&mut self, role: MemberRole) {
        self.role = role as i32
    }

    pub fn validate(&self) -> ApiResult<()> {
        Ok(())
    }

    pub fn to_json(&self, exclude: &[&str]) -> ApiResult<Value> {
        models::to_json(self, exclude)
    }
}

impl Member {
    pub async fn find(pool: &PgPool, space: i64, account: i64) -> ApiResult<Option<Self>> {
        db_run!(
            pool,
            member::table
                .find((space, account))
                .first(&*pool)
                .optional()
        )
    }

    pub async fn filter_by_space(pool: &PgPool, space: i64) -> ApiResult<Vec<Self>> {
        db_run!(
            pool,
            member::table.filter(member::space.eq(space)).load(&*pool)
        )
    }

    pub async fn filter_by_account(pool: &PgPool, account: i64) -> ApiResult<Vec<Self>> {
        db_run!(
            pool,
            member::table
                .filter(member::account.eq(account))
                .load(&*pool)
        )
    }

    pub async fn create(&self, pool: &PgPool) -> ApiResult<usize> {
        let record = self.clone();

        db_run!(
            pool,
            diesel::insert_into(member::table)
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

    // pub async fn delete_by_space_role(
    //     pool: &PgPool,
    //     space: i64,
    //     role: MemberRole,
    // ) -> ApiResult<usize> {
    //     db_run!(
    //         pool,
    //         diesel::delete(
    //             member::space
    //                 .eq(space)
    //                 .and(member::role.eq(role as i32)),
    //         )
    //         .execute(&*pool)
    //     )
    // }
}
