use crate::db::schema::account;

use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Insertable)]
#[table_name = "account"]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub discriminator: i32,
    pub avatar: String,
}

pub fn batch_create(conn: &PgConnection, accounts: &[Account]) -> QueryResult<usize> {
    diesel::insert_into(account::table)
        .values(accounts)
        .on_conflict(account::id)
        .do_update()
        .set((
            account::username.eq(excluded(account::username)),
            account::discriminator.eq(excluded(account::discriminator)),
            account::avatar.eq(excluded(account::avatar)),
        ))
        .execute(conn)
}

pub fn find(conn: &PgConnection, id: i64) -> QueryResult<Account> {
    account::table.find(id).first(conn)
}
