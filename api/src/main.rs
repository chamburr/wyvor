#![feature(proc_macro_hygiene, decl_macro, try_trait)]
#![recursion_limit = "128"]
#![allow(clippy::borrow_interior_mutable_const)]
#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nanoid;
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use dotenv::dotenv;
use rocket::config::Environment;
use rocket::fairing::AdHoc;

mod config;
mod constants;
mod db;
mod models;
mod routes;
mod utils;

use routes::admin;
use routes::errors;
use routes::guilds;
use routes::index;
use routes::tracks;
use routes::users;

fn main() {
    dotenv().ok();

    let _guard;
    if config::get_environment() == Environment::Production {
        _guard = sentry::init(config::get_sentry_dsn());
    }

    let rocket_config = config::get_rocket_config();

    rocket::custom(rocket_config)
        .attach(db::PgConn::fairing())
        .attach(db::RedisConn::fairing())
        .attach(utils::metrics::Metrics::fairing())
        .attach(AdHoc::on_launch(
            "Postgres database migration",
            db::migration::run_migrations,
        ))
        .attach(AdHoc::on_launch(
            "Andesite node client",
            utils::player::init_player,
        ))
        .attach(AdHoc::on_launch(
            "Redis database cache",
            db::cache::init_cache,
        ))
        .attach(AdHoc::on_launch(
            "Redis pub/sub listener",
            db::pubsub::init_pubsub,
        ))
        .attach(AdHoc::on_launch(
            "Discord oauth client",
            utils::auth::init_oauth,
        ))
        .attach(AdHoc::on_response("Response headers", |_, res| {
            res.remove_header("Server")
        }))
        .mount("/", routes![utils::metrics::get_metrics])
        .mount(
            "/api/",
            routes![
                index::index,
                index::get_login,
                index::get_invite,
                index::get_authorize,
                index::get_stats,
                index::get_stats_player,
                index::get_status,
            ],
        )
        .mount(
            "/api/admin",
            routes![
                admin::get_guilds,
                admin::get_top_guilds,
                admin::get_blacklist,
                admin::put_blacklist_item,
                admin::patch_blacklist_item,
                admin::delete_blacklist_item,
            ],
        )
        .mount(
            "/api/guilds",
            routes![
                guilds::get_guild,
                guilds::delete_guild,
                guilds::get_guild_polling,
                guilds::get_guild_stats,
                guilds::delete_guild_stats,
                guilds::get_guild_player,
                guilds::post_guild_player,
                guilds::patch_guild_player,
                guilds::delete_guild_player,
                guilds::get_guild_queue,
                guilds::post_guild_queue,
                guilds::delete_guild_queue,
                guilds::post_guild_queue_shuffle,
                guilds::put_guild_queue_item_position,
                guilds::delete_guild_queue_item,
                guilds::get_guild_playlists,
                guilds::post_guild_playlists,
                guilds::patch_guild_playlist,
                guilds::delete_guild_playlist,
                guilds::post_guild_playlist_load,
                guilds::get_guild_settings,
                guilds::patch_guild_settings,
                guilds::get_guild_logs,
            ],
        )
        .mount(
            "/api/tracks",
            routes![
                tracks::get_tracks,
                tracks::get_track,
                tracks::get_track_lyrics,
            ],
        )
        .mount(
            "/api/users",
            routes![
                users::get_users,
                users::get_user,
                users::get_user_me,
                users::get_user_me_guilds,
                users::get_users_me_guild,
                users::post_user_me_logout,
            ],
        )
        .register(catchers![
            errors::bad_request,
            errors::unauthorized,
            errors::forbidden,
            errors::not_found,
            errors::unprocessable_entity,
            errors::internal_server_error,
            errors::service_unavailable,
        ])
        .launch();
}
