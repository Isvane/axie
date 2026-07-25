use crate::auth::{AuthBody, AuthPayload, sign_token, verify_password};
use crate::errors::AuthError;
use crate::models::{AppState, User};
use axum::{Json, extract::State};
use std::sync::Arc;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    let mut db = state.db.clone();

    let user = User::get_by_email(&mut db, &payload.email)
        .await
        .map_err(|_| AuthError::WrongCredentials)?;

    verify_password(payload.password, user.password_hash)
        .await
        .map_err(|e| match e {
            AuthError::InvalidHashFormat => AuthError::InvalidHashFormat,
            _ => AuthError::WrongCredentials,
        })?;

    let company_name = user.company.clone();
    let role = user.role;

    let token = sign_token(user.id.to_string(), company_name, role)?;

    Ok(Json(AuthBody::new(token)))
}
