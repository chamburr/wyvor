use nanoid::nanoid;
use serde::Serialize;
use serde_json::Value;

mod account;
mod member;
mod playlist;
mod space;

pub use self::{
    account::{Account, AccountStatus},
    member::{Member, MemberRole},
    playlist::Playlist,
    space::Space,
};

#[macro_export]
macro_rules! db_run {
    ($pool:ident, $body:expr) => {{
        use $crate::error::ApiResult;

        use actix_web::web::block;

        let $pool = $pool.clone().get()?;

        block(move || -> ApiResult<_> { Ok($body?) }).await?
    }};
}

pub fn generate_id() -> i64 {
    nanoid!(16, &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
        .parse()
        .unwrap()
}

pub fn to_json<T: Serialize>(data: &T, exclude: &[&str]) -> Value {
    let mut value = serde_json::to_value(data).unwrap();

    for &key in exclude {
        value.as_object_mut().unwrap().remove(key);
    }

    value
}
