use crate::constants::{
    GUILD_PREFIX_MAX, GUILD_PREFIX_MIN, GUILD_QUEUE_MAX, GUILD_QUEUE_MIN, GUILD_ROLES_MAX,
};
use crate::db::schema::config;
use crate::models::{
    check_duplicate, string_int_opt, string_int_opt_vec, UpdateExt, Validate, ValidateExt,
};
use crate::routes::ApiResult;

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
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

#[derive(Debug, Clone, Deserialize, Insertable)]
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

pub fn create(conn: &PgConnection, new_config: &NewConfig) -> QueryResult<Config> {
    diesel::insert_into(config::table)
        .values(new_config)
        .on_conflict_do_nothing()
        .get_result(conn)
}

pub fn update(conn: &PgConnection, id: i64, edit_config: &EditConfig) -> QueryResult<usize> {
    diesel::update(config::table.find(id))
        .set(edit_config)
        .execute(conn)
        .safely()
}

pub fn delete(conn: &PgConnection, id: i64) -> QueryResult<usize> {
    diesel::delete(config::table.find(id)).execute(conn)
}

pub fn find(conn: &PgConnection, id: i64) -> QueryResult<Config> {
    config::table.find(id).first(conn)
}

pub fn all(conn: &PgConnection) -> QueryResult<Vec<Config>> {
    config::table.load(conn)
}
