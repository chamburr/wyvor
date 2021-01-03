use crate::constants::{BLACKLIST_REASON_MAX, BLACKLIST_REASON_MIN};
use crate::db::schema::blacklist;
use crate::db::PgPool;
use crate::models::{string_int, Validate, ValidateExt};
use crate::routes::ApiResult;

use actix_web::web::block;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error::QueryBuilderError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "blacklist"]
pub struct Blacklist {
    pub id: i64,
    pub reason: String,
    pub author: i64,
    pub created: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "blacklist"]
pub struct NewBlacklist {
    #[serde(deserialize_with = "string_int")]
    pub id: i64,
    pub reason: String,
    #[serde(deserialize_with = "string_int")]
    pub author: i64,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[table_name = "blacklist"]
pub struct EditBlacklist {
    pub reason: Option<String>,
}

impl Validate for NewBlacklist {
    fn check(&self) -> ApiResult<()> {
        self.reason.len().check_btw(
            BLACKLIST_REASON_MIN,
            BLACKLIST_REASON_MAX,
            "length of reason",
        )?;
        Ok(())
    }
}

impl Validate for EditBlacklist {
    fn check(&self) -> ApiResult<()> {
        if let Some(reason) = &self.reason {
            reason.len().check_btw(
                BLACKLIST_REASON_MIN,
                BLACKLIST_REASON_MAX,
                "length of reason",
            )?;
        }
        Ok(())
    }
}

pub async fn create(pool: &PgPool, new_blacklist: NewBlacklist) -> ApiResult<Blacklist> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Blacklist> {
        let conn = pool.get()?;
        let res = diesel::insert_into(blacklist::table)
            .values(new_blacklist)
            .on_conflict_do_nothing()
            .get_result(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn update(pool: &PgPool, id: i64, edit_blacklist: EditBlacklist) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::update(blacklist::table.find(id))
            .set(edit_blacklist)
            .execute(&*conn)
            .or_else(|err| {
                if let QueryBuilderError(_) = err {
                    Ok(0)
                } else {
                    Err(err)
                }
            })?;

        Ok(res)
    })
    .await?)
}

pub async fn delete(pool: &PgPool, id: i64) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::delete(blacklist::table.find(id)).execute(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn all(pool: &PgPool) -> ApiResult<Vec<Blacklist>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<Blacklist>> {
        let conn = pool.get()?;
        let res = blacklist::table.load(&*conn)?;

        Ok(res)
    })
    .await?)
}
