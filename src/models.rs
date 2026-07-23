use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, toasty::Embed)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Owner,
    User,
}

#[derive(Debug, toasty::Model, Serialize, Deserialize, Clone)]
pub(crate) struct User {
    #[key]
    #[auto]
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) company: String,
    pub(crate) role: Role,

    #[serde(skip_serializing, default)]
    pub(crate) password_hash: String,
    #[unique]
    pub(crate) email: String,
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
pub(crate) struct CreateUser {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub(crate) name: String,
    #[validate(email(message = "Invalid email address"))]
    pub(crate) email: String,
    #[validate(length(min = 8, message = "Password have to be 8 character long minimum"))]
    pub(crate) password: String,
    pub(crate) company: String,
    pub(crate) role: Option<Role>,
}

#[derive(Deserialize, validator::Validate)]
pub(crate) struct UpdateUser {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub(crate) name: String,
    #[validate(email(message = "Invalid email address"))]
    pub(crate) email: String,
}

#[derive(Deserialize)]
pub(crate) struct ChangeRolePayload {
    pub(crate) role: Role,
}

#[derive(Deserialize)]
pub(crate) struct TransferOwnershipPayload {
    pub(crate) new_owner_id: u64,
}

#[derive(Deserialize, validator::Validate)]
pub(crate) struct UpdateCompanyPayload {
    #[validate(length(min = 1, message = "Company name cannot be empty"))]
    pub(crate) company: String,
}
