// pub mod music;

// use crate::{error::ApiResult, routes::ApiResponse};

//use actix_web::web::block;
//use nanoid::nanoid;
//use std::thread;

// pub mod player;
// pub mod polling;
// pub mod queue;

// pub fn html_unescape(content: &str) -> String {
//     content
//         .replace("&lt;", "<")
//         .replace("&gt;", ">")
//         .replace("&amp;", "&")
//         .replace("&quot;", "\"")
//         .replace("&#x27;", "'")
// }

// pub async fn sleep(duration: std::time::Duration) -> ApiResult<()> {
//     block(move || -> ApiResult<()> {
//         thread::sleep(duration);
//         Ok(())
//     })
//     .await?;
//
//     Ok(())
// }

// pub fn generate_id() -> i64 {
//     nanoid!(16, &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
//         .parse()
//         .unwrap()
// }
