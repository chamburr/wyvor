use crate::error::ApiError;

use actix_web::{body::BoxBody, HttpRequest, Responder};
use serde::Serialize;
use serde_json::{json, Value};

pub mod basic;
pub mod spaces;
pub mod tracks;
pub mod users;

#[derive(Debug)]
pub struct ApiResponse {
    pub status: StatusCode,
    pub data: Value,
    pub error: Option<ApiError>,
}

impl ApiResponse {
    pub fn finish(self) -> ApiResult<Self> {
        match self.error {
            Some(err) => Err(err),
            None => match self.status {
                StatusCode::OK => Ok(self),
                _ => Err(self.into()),
            },
        }
    }

    pub fn data(mut self, data: impl Serialize) -> Self {
        match serde_json::to_value(data) {
            Ok(value) => self.data = value,
            Err(err) => self.error = Some(err.into()),
        }
        self
    }

    pub fn message(mut self, message: &str) -> Self {
        self.data = json!({ "message": message });
        self
    }

    pub fn ok() -> Self {
        Self {
            status: StatusCode::OK,
            data: json!({ "message": "The request made is successful." }),
            error: None,
        }
    }
}

impl Responder for ApiResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::build(self.status)
            .json(self.data)
            .map_into_boxed_body()
    }
}

pub async fn default_service() -> ApiResult<ApiResponse> {
    ApiResponse::not_found().finish()
}

macro_rules! make_errors {
    ($($name:ident: $message:literal),*) => {
        use crate::error::ApiResult;

        use actix_web::{
            dev::ServiceResponse,
            http::StatusCode,
            middleware::{ErrorHandlerResponse, ErrorHandlers},
            HttpResponse,
        };
        use paste::paste;

        paste! {
            impl ApiResponse {
                $(
                    pub fn [<$name:lower>]() -> Self {
                        Self {
                            status: StatusCode::$name,
                            data: json!({ "message": $message }),
                            error: None,
                        }
                    }
                )*
            }

            pub trait ResultExt<T> {
                $(
                    fn [<or_ $name:lower>](self) -> ApiResult<T>;
                )*
            }

            impl ResultExt<()> for bool {
                $(
                    fn [<or_ $name:lower>](self) -> ApiResult<()> {
                        if self {
                            Ok(())
                        } else {
                            Err(ApiResponse::[<$name:lower>]().into())
                        }
                    }
                )*
            }

            impl<T> ResultExt<T> for Option<T> {
                $(
                    fn [<or_ $name:lower>](self) -> ApiResult<T> {
                        self.ok_or_else(|| ApiResponse::[<$name:lower>]().into())
                    }
                )*
            }

            impl<T, E> ResultExt<T> for Result<T, E> {
                $(
                    fn [<or_ $name:lower>](self) -> ApiResult<T> {
                        self.map_err(|_| ApiResponse::[<$name:lower>]().into())
                    }
                )*
            }

            $(
                fn [<respond_ $name:lower>]<B>(
                    res: ServiceResponse<B>
                ) -> actix_web::Result<ErrorHandlerResponse<B>> {
                    let (req, _res) = res.into_parts();
                    let response = ApiResponse::[<$name:lower>]();

                    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
                        req,
                        HttpResponse::build(response.status)
                            .json(response.data)
                            .map_into_right_body(),
                    )))
                }
            )*

            pub fn error_handlers<B: 'static>() -> ErrorHandlers<B> {
                ErrorHandlers::new()
                    $(.handler(StatusCode::$name, [<respond_ $name:lower>]))*
            }
        }
    }
}

make_errors!(
    BAD_REQUEST: "The request you made is invalid.",
    UNAUTHORIZED: "You are not authorised to access this resource.",
    FORBIDDEN: "You do not have permission to perform this action.",
    NOT_FOUND: "The requested resource could not be found.",
    REQUEST_TIMEOUT: "The server did not receive a complete request.",
    INTERNAL_SERVER_ERROR: "The server encountered an internal error.",
    SERVICE_UNAVAILABLE: "The server cannot handle your request at this time."
);
