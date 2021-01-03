use crate::routes::{ApiResponse, ApiResult};

use serde::{de, Deserialize, Deserializer};
use std::cmp::PartialOrd;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

pub mod account;
pub mod blacklist;
pub mod config;
pub mod guild;
pub mod guild_log;
pub mod guild_stat;
pub mod playlist;
pub mod playlist_item;

pub trait Validate {
    fn check(&self) -> ApiResult<()>;
}

pub trait ValidateExt<T> {
    fn check_min(&self, num: T, name: &str) -> ApiResult<()>;
    fn check_max(&self, num: T, name: &str) -> ApiResult<()>;
    fn check_btw(&self, min: T, max: T, name: &str) -> ApiResult<()>;
}

impl<T: Display, U: PartialOrd<T>> ValidateExt<T> for U {
    fn check_min(&self, num: T, name: &str) -> ApiResult<()> {
        if self.lt(&num) {
            Err(ApiResponse::bad_request()
                .message(format!("The {} should be above {}.", &name, &num).as_str())
                .into())
        } else {
            Ok(())
        }
    }

    fn check_max(&self, num: T, name: &str) -> ApiResult<()> {
        if self.gt(&num) {
            Err(ApiResponse::bad_request()
                .message(format!("The {} should be below {}.", &name, &num).as_str())
                .into())
        } else {
            Ok(())
        }
    }

    fn check_btw(&self, min: T, max: T, name: &str) -> ApiResult<()> {
        if self.lt(&min) || self.gt(&max) {
            Err(ApiResponse::bad_request()
                .message(format!("The {} should be between {} and {}.", &name, &min, &max).as_str())
                .into())
        } else {
            Ok(())
        }
    }
}

pub fn check_duplicate<T: Hash + Eq>(items: &[T], name: &str) -> ApiResult<()> {
    let items_set: HashSet<&T> = items.iter().collect();

    if items_set.len() != items.len() {
        Err(ApiResponse::bad_request()
            .message(format!("The {} cannot contain duplicates.", &name).as_str())
            .into())
    } else {
        Ok(())
    }
}

pub fn string_int<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringInt<T> {
        String(String),
        Number(T),
    }

    match StringInt::<T>::deserialize(deserializer)? {
        StringInt::String(s) => s.parse::<T>().map_err(de::Error::custom),
        StringInt::Number(i) => Ok(i),
    }
}

pub fn string_int_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringInt<T> {
        StringOpt(Option<String>),
        NumberOpt(Option<T>),
    }

    match StringInt::<T>::deserialize(deserializer)? {
        StringInt::StringOpt(Some(s)) => {
            Some(s.parse::<T>().map_err(de::Error::custom)).transpose()
        },
        StringInt::NumberOpt(Some(i)) => Ok(Some(i)),
        _ => Ok(None),
    }
}

pub fn string_int_opt_vec<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringInt<T> {
        StringOptVec(Option<Vec<String>>),
        NumberOptVec(Option<Vec<T>>),
    }

    match StringInt::<T>::deserialize(deserializer)? {
        StringInt::StringOptVec(Some(s)) => Some(
            s.iter()
                .map(|str| str.parse::<T>().map_err(de::Error::custom))
                .collect(),
        )
        .transpose(),
        StringInt::NumberOptVec(Some(i)) => Ok(Some(i)),
        _ => Ok(None),
    }
}
