use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, toasty::Embed)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Owner,
    User,
}

#[derive(Debug, toasty::Model, Serialize, Deserialize, Clone)]
pub struct User {
    #[key]
    #[auto]
    pub id: u64,
    pub name: String,
    pub company: String,
    pub role: Role,

    #[serde(skip_serializing, default)]
    pub password_hash: String,
    #[unique]
    pub email: String,
}

pub struct AppState {
    pub db: toasty::db::Db,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Deserialize, validator::Validate)]
pub struct CreateUser {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password have to be 8 character long minimum"))]
    pub password: String,
    pub company: String,
    pub role: Option<Role>,
}

#[derive(Deserialize, validator::Validate)]
pub struct UpdateUser {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
}

#[derive(Deserialize)]
pub struct ChangeRolePayload {
    pub role: Role,
}

#[derive(Deserialize)]
pub struct TransferOwnershipPayload {
    pub new_owner_id: u64,
}

#[derive(Deserialize, validator::Validate)]
pub struct UpdateCompanyPayload {
    #[validate(length(min = 1, message = "Company name cannot be empty"))]
    pub company: String,
}
