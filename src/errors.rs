use crate::models::User;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub enum ApiResponse {
    Json(Vec<User>),
    Message(StatusCode, String),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Json(data) => (StatusCode::OK, Json(data)).into_response(),
            Self::Message(status, msg) => (status, msg).into_response(),
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    TokenCreation,
    InvalidToken,
    InvalidHashFormat,
    PasswordHashDontMatch,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials | AuthError::PasswordHashDontMatch => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password")
            }
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::InvalidHashFormat => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occured",
            ),
        };
        let body = Json(json!( {
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

pub enum AppError {
    EmailAlreadyExist(String),
    InvalidInput(String),
    UserNotFound(String),
    InternalDbError(String),
    Forbidden(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::EmailAlreadyExist(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            Self::InvalidInput(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg).into_response(),
            Self::UserNotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            Self::InternalDbError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg).into_response(),
        }
    }
}

impl From<toasty::Error> for AppError {
    fn from(err: toasty::Error) -> Self {
        AppError::InternalDbError(err.to_string())
    }
}
