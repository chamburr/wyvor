use crate::constants::{
    GUILD_PREFIX_MAX, GUILD_PREFIX_MIN, GUILD_QUEUE_MAX, GUILD_QUEUE_MIN, GUILD_ROLES_MAX,
};
use crate::db::schema::config;
use crate::db::PgPool;
use crate::models::{check_duplicate, string_int_opt, string_int_opt_vec, Validate, ValidateExt};
use crate::routes::ApiResult;

use actix_web::web::block;
use diesel::prelude::*;
use diesel::result::Error::QueryBuilderError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "config"]
pub struct Config {
    pub id: i64,
    pub prefix: String,
    pub max_queue: i32,
    pub no_duplicate: bool,
    pub keep_alive: bool,
    pub guild_roles: Vec<i64>,
    pub playlist_roles: Vec<i64>,
    pub player_roles: Vec<i64>,
    pub queue_roles: Vec<i64>,
    pub track_roles: Vec<i64>,
    pub playing_log: i64,
    pub player_log: i64,
    pub queue_log: i64,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name = "config"]
pub struct NewConfig {
    pub id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, AsChangeset)]
#[table_name = "config"]
pub struct EditConfig {
    pub prefix: Option<String>,
    pub max_queue: Option<i32>,
    pub no_duplicate: Option<bool>,
    pub keep_alive: Option<bool>,
    #[serde(default, deserialize_with = "string_int_opt_vec")]
    pub guild_roles: Option<Vec<i64>>,
    #[serde(default, deserialize_with = "string_int_opt_vec")]
    pub playlist_roles: Option<Vec<i64>>,
    #[serde(default, deserialize_with = "string_int_opt_vec")]
    pub player_roles: Option<Vec<i64>>,
    #[serde(default, deserialize_with = "string_int_opt_vec")]
    pub queue_roles: Option<Vec<i64>>,
    #[serde(default, deserialize_with = "string_int_opt_vec")]
    pub track_roles: Option<Vec<i64>>,
    #[serde(default, deserialize_with = "string_int_opt")]
    pub playing_log: Option<i64>,
    #[serde(default, deserialize_with = "string_int_opt")]
    pub player_log: Option<i64>,
    #[serde(default, deserialize_with = "string_int_opt")]
    pub queue_log: Option<i64>,
}

impl Validate for EditConfig {
    fn check(&self) -> ApiResult<()> {
        if let Some(prefix) = &self.prefix {
            prefix
                .len()
                .check_btw(GUILD_PREFIX_MIN, GUILD_PREFIX_MAX, "length of prefix")?;
        }

        if let Some(max_queue) = &self.max_queue {
            (max_queue.to_owned() as usize).check_btw(
                GUILD_QUEUE_MIN,
                GUILD_QUEUE_MAX,
                "max queue",
            )?;
        }

        if let Some(guild_roles) = &self.guild_roles {
            guild_roles
                .len()
                .check_max(GUILD_ROLES_MAX, "length of manage server roles")?;

            check_duplicate(guild_roles.as_slice(), "manage server roles")?;
        }

        if let Some(playlist_roles) = &self.playlist_roles {
            playlist_roles
                .len()
                .check_max(GUILD_ROLES_MAX, "length of manage playlist roles")?;

            check_duplicate(playlist_roles.as_slice(), "manage playlist roles")?;
        }

        if let Some(player_roles) = &self.player_roles {
            player_roles
                .len()
                .check_max(GUILD_ROLES_MAX, "length of manage player roles")?;

            check_duplicate(player_roles.as_slice(), "manage player roles")?;
        }

        if let Some(queue_roles) = &self.queue_roles {
            queue_roles
                .len()
                .check_max(GUILD_ROLES_MAX, "length of manage queue roles")?;

            check_duplicate(queue_roles.as_slice(), "manage queue roles")?;
        }

        if let Some(track_roles) = &self.track_roles {
            track_roles
                .len()
                .check_max(GUILD_ROLES_MAX, "length of add track roles")?;

            check_duplicate(track_roles.as_slice(), "add track roles")?;
        }

        Ok(())
    }
}

pub async fn create(pool: &PgPool, new_config: NewConfig) -> ApiResult<Config> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Config> {
        let conn = pool.get()?;
        let res = diesel::insert_into(config::table)
            .values(new_config)
            .on_conflict_do_nothing()
            .get_result(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn update(pool: &PgPool, id: i64, edit_config: EditConfig) -> ApiResult<usize> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<usize> {
        let conn = pool.get()?;
        let res = diesel::update(config::table.find(id))
            .set(edit_config)
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
        let res = diesel::delete(config::table.find(id)).execute(&*conn)?;

        Ok(res)
    })
    .await?)
}

pub async fn find(pool: &PgPool, id: i64) -> ApiResult<Option<Config>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Option<Config>> {
        let conn = pool.get()?;
        let res = config::table.find(id).first(&*conn).optional()?;

        Ok(res)
    })
    .await?)
}

pub async fn all(pool: &PgPool) -> ApiResult<Vec<Config>> {
    let pool = pool.clone();

    Ok(block(move || -> ApiResult<Vec<Config>> {
        let conn = pool.get()?;
        let res = config::table.load(&*conn)?;

        Ok(res)
    })
    .await?)
}
