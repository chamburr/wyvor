use crate::constants::{STATS_KEY, STATUS_KEY};
use crate::db::cache::models::{Stats, Status};
use crate::db::{cache, RedisConn};
use crate::routes::ApiResponse;
use crate::utils::auth::{
    get_csrf_redirect, get_invite_uri, get_redirect_uri, get_token_cookie, token_exchange,
};
use crate::utils::player::{get_node, NodeExt};

use percent_encoding::percent_decode_str;
use rocket::http::Cookies;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct SimpleRedirect {
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimpleAuth {
    pub code: String,
    pub state: Option<String>,
}

#[get("/")]
pub fn index() -> ApiResponse {
    ApiResponse::ok()
}

#[get("/login?<redirect>")]
pub fn get_login(redis_conn: RedisConn, redirect: Option<String>) -> ApiResponse {
    if let Some(redirect) = redirect {
        let redirect = percent_decode_str(redirect.as_str()).decode_utf8()?;
        if redirect.starts_with('/') {
            let uri = get_redirect_uri(&redis_conn, Some(redirect.to_string()))?;

            return ApiResponse::ok().data(SimpleRedirect { uri: Some(uri) });
        }
    }

    let uri = get_redirect_uri(&redis_conn, None)?;

    ApiResponse::ok().data(SimpleRedirect { uri: Some(uri) })
}

#[get("/invite?<guild>")]
pub fn get_invite(guild: Option<u64>) -> ApiResponse {
    let uri = get_invite_uri(guild)?;

    ApiResponse::ok().data(SimpleRedirect { uri: Some(uri) })
}

#[post("/authorize", data = "<auth>")]
pub fn get_authorize(
    redis_conn: RedisConn,
    auth: Json<SimpleAuth>,
    mut cookies: Cookies<'_>,
) -> ApiResponse {
    let auth = auth.into_inner();

    let token = token_exchange(auth.code.as_str()).map_err(|_| {
        ApiResponse::bad_request().message("The authorization code provided is invalid.")
    })?;

    let cookie = get_token_cookie(token);
    cookies.add_private(cookie);

    if let Some(token) = auth.state {
        let uri = get_csrf_redirect(&redis_conn, token.as_str())?;
        ApiResponse::ok().data(SimpleRedirect { uri })
    } else {
        ApiResponse::ok().data(SimpleRedirect { uri: None })
    }
}

#[get("/stats")]
pub fn get_stats(redis_conn: RedisConn) -> ApiResponse {
    let stats: Option<Stats> = cache::get(&redis_conn, STATS_KEY)?;

    if let Some(stats) = stats {
        ApiResponse::ok().data(stats)
    } else {
        ApiResponse::internal_server_error()
    }
}

#[get("/stats/player")]
pub fn get_stats_player(redis_conn: RedisConn) -> ApiResponse {
    let stats = get_node().get_stats(&redis_conn)?;

    ApiResponse::ok().data(stats)
}

#[get("/status")]
pub fn get_status(redis_conn: RedisConn) -> ApiResponse {
    let status: Option<Vec<Status>> = cache::get(&redis_conn, STATUS_KEY)?;

    if let Some(status) = status {
        ApiResponse::ok().data(status)
    } else {
        ApiResponse::internal_server_error()
    }
}
