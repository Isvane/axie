use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    RequestPartsExt, extract::FromRequestParts, extract::State, http::Request, middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use casbin::CoreApi;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fmt::Display, sync::Arc, sync::LazyLock};

use crate::errors::{AppError, AuthError};
use crate::models::{AppState, Role};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub role: Role,
    exp: u64,
}

#[derive(Debug, Clone)]
pub struct GeneratedRefreshToken {
    pub raw_token: String,
    pub hash_token: String,
    pub exp_at: u64,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

pub async fn casbin_enforce(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let enforce_result = state.enforcer.enforce((
        claims.role.as_str(),
        request.uri().path(),
        request.method().as_str(),
    ));

    match enforce_result {
        Ok(true) => Ok(next.run(request).await),
        Ok(false) => Err(AppError::Forbidden(
            "Forbidden: Insufficient permissions".into(),
        )),
        Err(e) => Err(AppError::InternalDbError(format!("Casbin error: {e}"))),
    }
}

pub fn sign_token(user_id: String, company: String, role: Role) -> Result<String, AuthError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| AuthError::TokenCreation)?
        .as_secs();

    let expiration = now + (900);

    let claims = Claims {
        sub: user_id,
        company,
        role,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &KEYS.encoding).map_err(|_| AuthError::TokenCreation)
}

pub fn get_refresh_token() -> Result<GeneratedRefreshToken, AuthError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| AuthError::TokenCreation)?
        .as_secs();

    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);

    let raw_token = URL_SAFE_NO_PAD.encode(bytes);
    let hash_token = hash_refresh_token(&raw_token);
    let exp_at = now + (604800);

    Ok(GeneratedRefreshToken {
        raw_token,
        hash_token,
        exp_at,
    })
}

pub fn hash_refresh_token(raw_token: &str) -> String {
    let digest = Sha256::digest(raw_token.as_bytes());
    hex::encode(digest)
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}\nCompany: {}", self.sub, self.company)
    }
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    refresh_token: String,
    token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes())
            .map_err(|e| AppError::InternalDbError(format!("Hashing failed: {e}")))?;

        Ok(password_hash.to_string())
    })
    .await
    .map_err(|e| AppError::InternalDbError(format!("Thread join error: {e}")))?
}

pub async fn verify_password(password: String, hashed_password: String) -> Result<bool, AuthError> {
    tokio::task::spawn_blocking(move || {
        let parsed_hash =
            PasswordHash::new(&hashed_password).map_err(|_| AuthError::InvalidHashFormat)?;

        match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Err(AuthError::PasswordHashDontMatch),
        }
    })
    .await
    .map_err(|_| AuthError::InvalidHashFormat)?
}
