#![recursion_limit = "128"]
#![deny(clippy::all, nonstandard_style, rust_2018_idioms)] // unused, warnings)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::{
    config::{Environment, CONFIG},
    db::{get_pg_pool, get_redis_pool, run_migrations},
    error::ApiResult,
    routes::{basic, spaces, tracks, users},
};

use actix_web::{
    middleware::{Logger, NormalizePath},
    web::{self, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use std::env;
use tracing::error;

mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod utils;
mod websockets;
use actix::prelude::*;
#[actix_web::main]
pub async fn main() {
    dotenv().ok();

    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or_else(|_| "INFO".to_string()),
    );

    tracing_subscriber::fmt::init();
    tracing_log::env_logger::init();

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
    run_migrations(&pool).await?;
    let server = websockets::server::ChatServer::new().start();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(redis_pool.clone()))
            .app_data(Data::new(server.clone()))
            .wrap(routes::error_handlers())
            .wrap(Logger::default())
            .wrap(NormalizePath::trim())
            .service(basic::get_index)
            .service(
                web::scope("/auth")
                    .service(basic::post_auth_register)
                    .service(basic::post_auth_login)
                    .service(basic::post_auth_refresh)
                    .service(basic::post_auth_logout),
            )
            .route("/ws", web::get().to(websockets::chat_route))
            .service(
                web::scope("/spaces")
                    .service(spaces::post_spaces)
                    .service(spaces::get_space)
                    .service(spaces::patch_space)
                    .service(spaces::delete_space)
                    .service(spaces::get_space_members)
                    .service(spaces::post_space_members)
                    .service(spaces::get_space_member)
                    .service(spaces::patch_space_member)
                    .service(spaces::delete_space_member)
                    .service(spaces::get_space_playlists)
                    .service(spaces::post_space_playlists)
                    .service(spaces::get_space_playlist)
                    .service(spaces::patch_space_playlist)
                    .service(spaces::delete_space_playlist),
            )
            .service(
                web::scope("/tracks")
                    .service(tracks::get_tracks)
                    .service(tracks::get_track)
                    .service(tracks::get_track_lyrics),
            )
            .service(
                web::scope("/users")
                    .service(users::get_user)
                    .service(users::get_user_me)
                    .service(users::patch_user_me)
                    .service(users::get_user_me_spaces)
                    .service(users::post_user_me_spaces)
                    .service(users::get_user_me_space)
                    .service(users::delete_user_me_space),
            )
            .default_service(web::to(routes::default_service))
    })
    .workers(CONFIG.api_workers as usize)
    .bind(CONFIG.api_address())?
    .run()
    .await?;

    Ok(())
}
