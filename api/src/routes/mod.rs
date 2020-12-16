use backtrace::Backtrace;
use futures::channel::mpsc::TrySendError;
use http::StatusCode;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket_contrib::databases::redis::RedisError;
use rocket_contrib::json::JsonValue;
use serde::Serialize;
use std::error::Error;
use std::io::Cursor;
use std::ops::Try;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use twilight_andesite::client::ClientError;
use twilight_andesite::node::NodeError;
use twilight_oauth2::client::RedirectUriInvalidError;

pub mod admin;
pub mod errors;
pub mod guilds;
pub mod index;
pub mod tracks;
pub mod users;

#[derive(Debug)]
pub struct ApiResponse {
    status: Status,
    data: JsonValue,
}

pub type ApiResult<T> = Result<T, ApiResponse>;

impl ApiResponse {
    pub fn data(mut self, data: impl Serialize) -> ApiResponse {
        let value = serde_json::to_value(data);
        match value {
            Ok(val) => {
                self.data = JsonValue::from(val);
                self
            },
            Err(err) => ApiResponse::from(err),
        }
    }

    pub fn message(mut self, message: &str) -> ApiResponse {
        self.data = json!({ "message": message });
        self
    }

    pub fn ok() -> ApiResponse {
        ApiResponse {
            status: Status::Ok,
            data: json!({"message": "The request made is successful."}),
        }
    }

    pub fn bad_request() -> ApiResponse {
        ApiResponse {
            status: Status::BadRequest,
            data: json!({"message": "The request you made is invalid."}),
        }
    }

    pub fn unauthorized() -> ApiResponse {
        ApiResponse {
            status: Status::Unauthorized,
            data: json!({"message": "You are not authorised to access this resource."}),
        }
    }

    pub fn forbidden() -> ApiResponse {
        ApiResponse {
            status: Status::Forbidden,
            data: json!({"message": "You do not have permission to perform this action."}),
        }
    }

    pub fn not_found() -> ApiResponse {
        ApiResponse {
            status: Status::NotFound,
            data: json!({"message": "The requested resource could not be found."}),
        }
    }

    pub fn internal_server_error() -> ApiResponse {
        ApiResponse {
            status: Status::InternalServerError,
            data: json!({"message": "The server encountered an internal error."}),
        }
    }

    pub fn service_unavailable() -> ApiResponse {
        ApiResponse {
            status: Status::ServiceUnavailable,
            data: json!({"message": "The server cannot handle your request at this time."}),
        }
    }
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, _req: &Request<'_>) -> Result<Response<'r>, Status> {
        let body = self.data;

        Response::build()
            .status(self.status)
            .sized_body(Cursor::new(body.to_string()))
            .header(ContentType::JSON)
            .ok()
    }
}

fn handle_error<T: Error>(error: T) -> ApiResponse {
    eprintln!("{:?}", error);
    eprintln!("{:?}", Backtrace::new());

    sentry::capture_error(&error);

    ApiResponse::internal_server_error()
}

impl From<diesel::result::Error> for ApiResponse {
    fn from(error: diesel::result::Error) -> Self {
        if error == diesel::result::Error::NotFound {
            return Self::not_found();
        }

        handle_error(error)
    }
}

impl From<RedisError> for ApiResponse {
    fn from(error: RedisError) -> Self {
        handle_error(error)
    }
}

impl From<serde_json::Error> for ApiResponse {
    fn from(error: serde_json::Error) -> Self {
        handle_error(error)
    }
}

impl From<twilight_http::Error> for ApiResponse {
    fn from(error: twilight_http::Error) -> Self {
        handle_error(error)
    }
}

impl From<ClientError> for ApiResponse {
    fn from(error: ClientError) -> Self {
        handle_error(error)
    }
}

impl From<reqwest::Error> for ApiResponse {
    fn from(error: reqwest::Error) -> Self {
        if let Some(status) = error.status() {
            if status == StatusCode::NOT_FOUND {
                return Self::not_found();
            }
        }

        handle_error(error)
    }
}

impl<T: 'static> From<TrySendError<T>> for ApiResponse {
    fn from(error: TrySendError<T>) -> Self {
        handle_error(error)
    }
}

impl From<prometheus::Error> for ApiResponse {
    fn from(error: prometheus::Error) -> Self {
        handle_error(error)
    }
}

impl From<FromUtf8Error> for ApiResponse {
    fn from(error: FromUtf8Error) -> Self {
        handle_error(error)
    }
}

impl From<Utf8Error> for ApiResponse {
    fn from(error: Utf8Error) -> Self {
        handle_error(error)
    }
}

impl From<RedirectUriInvalidError<'_>> for ApiResponse {
    fn from(error: RedirectUriInvalidError<'_>) -> Self {
        handle_error(error)
    }
}

impl From<NodeError> for ApiResponse {
    fn from(error: NodeError) -> Self {
        handle_error(error)
    }
}

impl Try for ApiResponse {
    type Ok = ApiResponse;
    type Error = ApiResponse;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        if self.status == Status::Ok {
            Ok(self)
        } else {
            Err(self)
        }
    }

    fn from_error(v: Self::Error) -> Self {
        v
    }

    fn from_ok(v: Self::Ok) -> Self {
        v
    }
}

pub trait OptionExt<T> {
    fn into_bad_request(self) -> ApiResult<T>;
    fn into_not_found(self) -> ApiResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn into_bad_request(self) -> ApiResult<T> {
        self.ok_or_else(ApiResponse::bad_request)
    }

    fn into_not_found(self) -> ApiResult<T> {
        self.ok_or_else(ApiResponse::not_found)
    }
}
