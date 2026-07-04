use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use std::sync::Arc;

use crate::errors::{ApiResponse, AppError};
use crate::models::ChangeRolePayload;
use crate::models::{AppState, Pagination, Role, User};

pub async fn list_users(
    _claims: crate::auth::Claims,
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<ApiResponse, AppError> {
    tracing::info!("Attempting to fetch user data");
    let mut db = state.db.clone();

    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);

    let offset = if page > 0 { (page - 1) * per_page } else { 0 };

    let users = User::all()
        .limit(per_page.try_into().unwrap())
        .offset(offset.try_into().unwrap())
        .exec(&mut db)
        .await?;

    Ok(ApiResponse::Json(users))
}

pub async fn change_user_role(
    claims: crate::auth::Claims,
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Json(payload): Json<ChangeRolePayload>,
) -> Result<ApiResponse, AppError> {
    if claims.role != Role::Admin && claims.role != Role::Owner {
        return Err(AppError::Forbidden("Admin permission required".to_string()));
    }

    let mut db = state.db.clone();
    let mut user = User::get_by_id(&mut db, id)
        .await
        .map_err(|_| AppError::UserNotFound("User not found".to_string()))?;

    if user.company != claims.company {
        return Err(AppError::Forbidden(
            "Cannot modify users outside your company".to_string(),
        ));
    }

    if claims.role == Role::Admin && (user.role == Role::Owner || payload.role == Role::Owner) {
        return Err(AppError::Forbidden(
            "Admins cannot modify or assign Owner roles".to_string(),
        ));
    }

    user.update().role(payload.role).exec(&mut db).await?;

    Ok(ApiResponse::Message(
        StatusCode::OK,
        "Role updated".to_string(),
    ))
}
