use crate::auth::{
    AuthBody, AuthPayload, get_refresh_token, hash_refresh_token, sign_token, verify_password,
};
use crate::errors::AuthError;
use crate::models::{AppState, RefreshPayload, User, UserRefresh};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn curr_unix_secs() -> Result<u64, AuthError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| AuthError::TokenCreation)
}

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

    let access_token = sign_token(user.id.to_string(), user.company, user.role)?;
    let refresh = get_refresh_token()?;

    toasty::create!(UserRefresh {
        hash_token: refresh.hash_token,
        user_id: user.id,
        exp_at: refresh.exp_at,
        revoked: false,
    })
    .exec(&mut db)
    .await
    .map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(AuthBody::new(access_token, refresh.raw_token)))
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    let mut db = state.db.clone();
    let now = curr_unix_secs()?;

    let hashed = hash_refresh_token(&payload.refresh_token);
    let mut stored = UserRefresh::get_by_hash_token(&mut db, &hashed)
        .await
        .map_err(|_| AuthError::InvalidToken)?;

    if stored.revoked {
        let _ = toasty::update!(UserRefresh::filter_by_user_id(stored.user_id) {
            revoked: true,
        })
        .exec(&mut db)
        .await;
        return Err(AuthError::InvalidToken);
    }

    if now >= stored.exp_at {
        let _ = toasty::update!(stored { revoked: true })
            .exec(&mut db)
            .await;
        return Err(AuthError::InvalidToken);
    }

    toasty::update!(stored { revoked: true })
        .exec(&mut db)
        .await
        .map_err(|_| AuthError::TokenCreation)?;

    let user = User::get_by_id(&mut db, &stored.user_id)
        .await
        .map_err(|_| AuthError::InvalidToken)?;

    let new_access = sign_token(user.id.to_string(), user.company, user.role)?;
    let new_refresh = get_refresh_token()?;

    toasty::create!(UserRefresh {
        hash_token: new_refresh.hash_token,
        user_id: user.id,
        exp_at: new_refresh.exp_at,
        revoked: false,
    })
    .exec(&mut db)
    .await
    .map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(AuthBody::new(new_access, new_refresh.raw_token)))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshPayload>,
) -> Result<StatusCode, AuthError> {
    let mut db = state.db.clone();
    let hashed = hash_refresh_token(&payload.refresh_token);

    if let Ok(mut stored) = UserRefresh::get_by_hash_token(&mut db, &hashed).await
        && !stored.revoked
    {
        let _ = toasty::update!(stored { revoked: true })
            .exec(&mut db)
            .await;
    }

    Ok(StatusCode::NO_CONTENT)
}
