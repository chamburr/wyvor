use crate::constants::{BLACKLIST_REASON_MAX, BLACKLIST_REASON_MIN};
use crate::db::schema::blacklist;
use crate::models::{string_int, UpdateExt, Validate, ValidateExt};
use crate::routes::ApiResult;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "blacklist"]
pub struct Blacklist {
    pub id: i64,
    pub reason: String,
    pub author: i64,
    pub created: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "blacklist"]
pub struct NewBlacklist {
    #[serde(deserialize_with = "string_int")]
    pub id: i64,
    pub reason: String,
    #[serde(deserialize_with = "string_int")]
    pub author: i64,
}

#[derive(Debug, Clone, Deserialize, AsChangeset)]
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

pub fn create(conn: &PgConnection, new_blacklist: &NewBlacklist) -> QueryResult<Blacklist> {
    diesel::insert_into(blacklist::table)
        .values(new_blacklist)
        .on_conflict_do_nothing()
        .get_result(conn)
}

pub fn update(conn: &PgConnection, id: i64, edit_blacklist: &EditBlacklist) -> QueryResult<usize> {
    diesel::update(blacklist::table.find(id))
        .set(edit_blacklist)
        .execute(conn)
        .safely()
}

pub fn delete(conn: &PgConnection, id: i64) -> QueryResult<usize> {
    diesel::delete(blacklist::table.find(id)).execute(conn)
}

pub fn all(conn: &PgConnection) -> QueryResult<Vec<Blacklist>> {
    blacklist::table.load(conn)
}
