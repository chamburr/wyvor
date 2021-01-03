use crate::routes::ApiResult;
use crate::utils::queue::QueueItem;

use actix_web::web::block;
use chrono::Duration;
use std::thread;

pub mod auth;
pub mod log;
pub mod metrics;
pub mod player;
pub mod polling;
pub mod queue;

pub fn html_unescape(content: &str) -> String {
    content
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
}

pub fn format_duration(milliseconds: u64) -> String {
    let duration = Duration::milliseconds(milliseconds as i64);

    if duration.num_hours() > 0 {
        format!(
            "{:02}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    } else {
        format!(
            "{:02}:{:02}",
            duration.num_minutes(),
            duration.num_seconds() % 60
        )
    }
}

pub fn format_track(track: &QueueItem) -> String {
    format!("[{}]({})", track.title, track.uri)
}

pub async fn sleep(duration: std::time::Duration) -> ApiResult<()> {
    block(move || -> ApiResult<()> {
        thread::sleep(duration);

        Ok(())
    })
    .await?;

    Ok(())
}
