use crate::utils::queue::QueueItem;

use chrono::Duration;
use core::future::Future;
use futures::executor;
use tokio_compat_02::FutureExt;

pub mod auth;
pub mod log;
pub mod metrics;
pub mod player;
pub mod polling;
pub mod queue;

pub fn block_on<F: Future>(f: F) -> F::Output {
    executor::block_on(f.compat())
}

pub fn html_unescape(content: &str) -> String {
    content
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
}

pub fn to_snake_case(content: &str) -> String {
    let mut snake = String::new();
    for (index, char) in content.char_indices() {
        if index > 0 && char.is_uppercase() {
            snake.push('_');
        }

        snake.push(char.to_ascii_lowercase());
    }

    snake
}

pub fn to_screaming_snake_case(content: &str) -> String {
    to_snake_case(content).to_uppercase()
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
