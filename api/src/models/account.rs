use crate::db::schema::account;
use crate::db::PgPool;
use crate::routes::ApiResult;

use actix_web::web::block;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable, Insertable)]
#[table_name = "account"]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub discriminator: i32,
    pub avatar: String,
}

pub async fn batch_create(pool: &PgPool, accounts: Vec<Account>) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::insert_into(account::table)
            .values(accounts)
            .on_conflict(account::id)
            .do_update()
            .set((
                account::username.eq(excluded(account::username)),
                account::discriminator.eq(excluded(account::discriminator)),
                account::avatar.eq(excluded(account::avatar)),
            ))
            .execute(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find(pool: &PgPool, id: i64) -> ApiResult<Option<Account>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Option<Account>> {
        let conn = pool.get()?;
        let res = account::table.find(id).first(&*conn).optional()?;

        Ok(res)
    })
    .await?)
}
