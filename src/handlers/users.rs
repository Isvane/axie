use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use std::sync::Arc;
use validator::Validate;

use crate::auth::hash_password;
use crate::models::{AppState, CreateUser, Role, User};
use crate::{
    errors::{ApiResponse, AppError},
    models::UpdateUser,
};

pub async fn about() -> (StatusCode, &'static str) {
    (StatusCode::OK, "I'm the user")
}

pub async fn greet_user(Path(name): Path<String>) -> ApiResponse {
    ApiResponse::Message(StatusCode::OK, format!("Hello {name}"))
}

pub async fn delete_user(
    claims: crate::auth::Claims,
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<ApiResponse, AppError> {
    let auth_id: u64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Forbidden("Invalid user ID".to_string()))?;

    if auth_id != id {
        return Err(AppError::Forbidden(
            "You do not have permission to delete this profile".to_string(),
        ));
    }

    let mut db = state.db.clone();

    let user = User::get_by_id(&mut db, &id)
        .await
        .map_err(|_| AppError::UserNotFound("User not found".to_string()))?;

    user.delete().exec(&mut db).await?;

    Ok(ApiResponse::Message(
        StatusCode::OK,
        format!("Deleted user: {id}"),
    ))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateUser>,
) -> Result<ApiResponse, AppError> {
    if let Err(errors) = input.validate() {
        return Err(AppError::InvalidInput(errors.to_string()));
    }

    tracing::info!("Attempting to create user: {}", input.email);
    let mut db = state.db.clone();

    if User::get_by_email(&mut db, &input.email).await.is_ok() {
        return Err(AppError::EmailAlreadyExist(
            "User with this email already exists".to_string(),
        ));
    }

    let CreateUser {
        name,
        email,
        password,
        company,
        role: _input_role,
    } = input;

    let password_hash = hash_password(password).await?;
    let role = Role::User;

    let _new_user = toasty::create!(User {
        name,
        email,
        password_hash,
        company,
        role,
    })
    .exec(&mut db)
    .await?;

    Ok(ApiResponse::Message(
        StatusCode::CREATED,
        "Created user successfully".to_string(),
    ))
}

pub async fn update_users(
    claims: crate::auth::Claims,
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUser>,
) -> Result<ApiResponse, AppError> {
    let auth_id: u64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Forbidden("Invalid user ID".to_string()))?;

    if auth_id != id {
        return Err(AppError::Forbidden(
            "You do not have permission to update this profile".to_string(),
        ));
    }

    let mut db = state.db.clone();

    let mut user = User::get_by_id(&mut db, &id)
        .await
        .map_err(|_| AppError::UserNotFound("User not found".to_string()))?;

    user.update()
        .name(payload.name)
        .email(payload.email)
        .exec(&mut db)
        .await?;

    Ok(ApiResponse::Message(
        StatusCode::OK,
        format!("Updated user: {id}"),
    ))
}
