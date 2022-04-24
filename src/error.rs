use crate::{music, routes::ApiResponse};

use actix_web::{
    error::{BlockingError, HttpError},
    http::StatusCode,
    HttpResponse, ResponseError,
};
use base64::DecodeError;
use bcrypt::BcryptError;
use diesel_migrations::RunMigrationsError;
use handlebars::RenderError;
use http::header::ToStrError;
use lettre::address::AddressError;
use redis::RedisError;
use serde_json::Value;
use std::{
    fmt::{self, Display, Formatter},
    io,
    net::AddrParseError,
};
use url::ParseError;

pub type ApiResult<T> = Result<T, ApiError>;

macro_rules! make_api_error {
    ($($name:ident($ty:ty)),*) => {
        #[derive(Debug)]
        #[allow(clippy::enum_variant_names)]
        pub enum ApiError {
            CustomError((StatusCode, Value)),
            $($name($ty)),*
        }

        $(
            impl From<$ty> for ApiError {
                fn from(err: $ty) -> Self {
                    sentry::capture_error(&err);
                    Self::$name(err)
                }
            }
        )*
    }
}

// TODO: remove unused errors
make_api_error! {
    AddrParseError(AddrParseError),
    AddressError(AddressError),
    BcryptError(BcryptError),
    BlockingError(BlockingError),
    DecodeError(DecodeError),
    DieselResultError(diesel::result::Error),
    HttpError(HttpError),
    IoError(io::Error),
    JsonWebTokenError(jsonwebtoken::errors::Error),
    LettreError(lettre::error::Error),
    ParseError(ParseError),
    R2d2Error(r2d2::Error),
    RedisError(RedisError),
    RenderError(RenderError),
    ReqwestError(reqwest::Error),
    RunMigrationsError(RunMigrationsError),
    SerdeJsonError(serde_json::Error),
    SmtpError(lettre::transport::smtp::Error),
    ToStrError(ToStrError),

    MusicError(music::Error)
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::CustomError((status, value)) => HttpResponse::build(*status).json(value),
            _ => {
                let res = ApiResponse::internal_server_error();
                HttpResponse::build(res.status).json(&res.data)
            },
        }
    }
}

impl From<ApiResponse> for ApiError {
    fn from(err: ApiResponse) -> ApiError {
        ApiError::CustomError((err.status, err.data))
    }
}

impl From<actix_web::Error> for ApiError {
    fn from(err: actix_web::Error) -> Self {
        sentry::capture_error(&err);
        ApiResponse::internal_server_error().into()
    }
}
