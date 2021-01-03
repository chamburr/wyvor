#![recursion_limit = "128"]
#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::config::{get_api_address, Environment, CONFIG};
use crate::db::cache::init_cache;
use crate::db::migration::run_migrations;
use crate::db::pubsub::init_pubsub;
use crate::db::{get_amqp_conn, get_pg_pool, get_redis_pool};
use crate::routes::{admin, errors, guilds, index, tracks, users, ApiResult};
use crate::utils::metrics::Metrics;
use crate::utils::player::init_player;

use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::ErrorHandlers;
use actix_web::middleware::normalize::TrailingSlash;
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use tracing::error;
use tracing_log::env_logger;

mod config;
mod constants;
mod db;
mod models;
mod routes;
mod utils;

#[actix_web::main]
pub async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    env_logger::init();

    let result = real_main().await;

    if let Err(err) = result {
        error!("{:?}", err);
    }
}

pub async fn real_main() -> ApiResult<()> {
    let _guard;
    if CONFIG.environment == Environment::Production {
        _guard = sentry::init(CONFIG.sentry_dsn.clone());
    }

    let pool = get_pg_pool()?;
    let redis_pool = get_redis_pool()?;
    let amqp_conn = get_amqp_conn().await?;
    let amqp_channel = amqp_conn.create_channel().await?;

    run_migrations(&pool).await?;

    init_cache(pool.clone(), redis_pool.clone());
    init_pubsub();
    init_player(pool.clone(), redis_pool.clone(), amqp_channel.clone());

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(redis_pool.clone())
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::BAD_REQUEST, errors::bad_request)
                    .handler(StatusCode::UNAUTHORIZED, errors::unauthorized)
                    .handler(StatusCode::FORBIDDEN, errors::forbidden)
                    .handler(StatusCode::NOT_FOUND, errors::not_found)
                    .handler(StatusCode::REQUEST_TIMEOUT, errors::request_timeout)
                    .handler(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        errors::internal_server_error,
                    )
                    .handler(StatusCode::SERVICE_UNAVAILABLE, errors::service_unavailable),
            )
            .wrap(Metrics)
            .wrap(Logger::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .service(utils::metrics::get_metrics)
            .service(
                web::scope("/api")
                    .service(index::index)
                    .service(index::get_login)
                    .service(index::get_invite)
                    .service(index::get_authorize)
                    .service(index::get_stats)
                    .service(index::get_stats_player)
                    .service(index::get_status)
                    .service(
                        web::scope("/admin")
                            .service(admin::get_guilds)
                            .service(admin::get_top_guilds)
                            .service(admin::get_blacklist)
                            .service(admin::put_blacklist_item)
                            .service(admin::patch_blacklist_item)
                            .service(admin::delete_blacklist_item),
                    )
                    .service(
                        web::scope("/guilds")
                            .service(guilds::get_guild)
                            .service(guilds::delete_guild)
                            .service(guilds::get_guild_polling)
                            .service(guilds::get_guild_stats)
                            .service(guilds::delete_guild_stats)
                            .service(guilds::get_guild_player)
                            .service(guilds::post_guild_player)
                            .service(guilds::patch_guild_player)
                            .service(guilds::delete_guild_player)
                            .service(guilds::get_guild_queue)
                            .service(guilds::post_guild_queue)
                            .service(guilds::delete_guild_queue)
                            .service(guilds::post_guild_queue_shuffle)
                            .service(guilds::put_guild_queue_item_position)
                            .service(guilds::delete_guild_queue_item)
                            .service(guilds::get_guild_playlists)
                            .service(guilds::post_guild_playlists)
                            .service(guilds::patch_guild_playlist)
                            .service(guilds::delete_guild_playlist)
                            .service(guilds::post_guild_playlist_load)
                            .service(guilds::get_guild_settings)
                            .service(guilds::patch_guild_settings)
                            .service(guilds::get_guild_logs),
                    )
                    .service(
                        web::scope("/tracks")
                            .service(tracks::get_tracks)
                            .service(tracks::get_track)
                            .service(tracks::get_track_lyrics),
                    )
                    .service(
                        web::scope("/users")
                            .service(users::get_users)
                            .service(users::get_user)
                            .service(users::get_user_me)
                            .service(users::get_user_me_guilds)
                            .service(users::get_users_me_guild)
                            .service(users::post_user_me_logout),
                    ),
            )
            .default_service(web::to(errors::default_service))
    })
    .workers(CONFIG.api_workers as usize)
    .bind(get_api_address()?)?
    .run()
    .await?;

    Ok(())
}
