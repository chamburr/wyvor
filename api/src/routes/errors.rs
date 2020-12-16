use crate::routes::ApiResponse;

#[catch(400)]
pub fn bad_request() -> ApiResponse {
    ApiResponse::bad_request()
}

#[catch(401)]
pub fn unauthorized() -> ApiResponse {
    ApiResponse::unauthorized()
}

#[catch(403)]
pub fn forbidden() -> ApiResponse {
    ApiResponse::forbidden()
}

#[catch(404)]
pub fn not_found() -> ApiResponse {
    ApiResponse::not_found()
}

#[catch(422)]
pub fn unprocessable_entity() -> ApiResponse {
    ApiResponse::bad_request()
}

#[catch(500)]
pub fn internal_server_error() -> ApiResponse {
    ApiResponse::internal_server_error()
}

#[catch(503)]
pub fn service_unavailable() -> ApiResponse {
    ApiResponse::service_unavailable()
}
