use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;

use crate::auth::Claims;
use crate::errors::{ApiResponse, AppError};
use crate::models::{AppState, Role, TransferOwnershipPayload, UpdateCompanyPayload, User};

pub async fn transfer_ownership(
    claims: Claims,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TransferOwnershipPayload>,
) -> Result<ApiResponse, AppError> {
    if claims.role != Role::Owner {
        return Err(AppError::Forbidden(
            "Only the company owner can transfer ownership".to_string(),
        ));
    }

    let current_owner_id: u64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Forbidden("Invalid user ID".to_string()))?;

    if current_owner_id == payload.new_owner_id {
        return Err(AppError::Forbidden(
            "You are already the owner of this company".to_string(),
        ));
    }

    let mut db = state.db.clone();

    let mut target_user = User::get_by_id(&mut db, &payload.new_owner_id)
        .await
        .map_err(|_| AppError::UserNotFound("Target user not found".to_string()))?;

    if target_user.company != claims.company {
        return Err(AppError::Forbidden(
            "Cannot transfer ownership to a user outside your company".to_string(),
        ));
    }

    let mut current_owner = User::get_by_id(&mut db, &current_owner_id)
        .await
        .map_err(|_| AppError::UserNotFound("Current owner record not found".to_string()))?;

    target_user.update().role(Role::Owner).exec(&mut db).await?;

    current_owner
        .update()
        .role(Role::Admin)
        .exec(&mut db)
        .await?;

    Ok(ApiResponse::Message(
        StatusCode::OK,
        format!(
            "Ownership successfully transferred to user {}",
            payload.new_owner_id
        ),
    ))
}

pub async fn rename_company(
    claims: Claims,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateCompanyPayload>,
) -> Result<ApiResponse, AppError> {
    if claims.role != Role::Owner {
        return Err(AppError::Forbidden(
            "Only the company owner can change the company name".to_string(),
        ));
    }

    let company_name = claims.company;

    if company_name == payload.company {
        return Err(AppError::Forbidden(
            "You are already using this name".to_string(),
        ));
    }

    let mut db = state.db.clone();

    User::filter(User::fields().company().eq(&company_name))
        .update()
        .company(&payload.company)
        .exec(&mut db)
        .await?;

    Ok(ApiResponse::Message(
        StatusCode::OK,
        format!("Successfully changed company name to {}", payload.company),
    ))
}
