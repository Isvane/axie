use super::*;
use crate::auth::sign_token;
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use std::net::SocketAddr;
use tower::{Service, ServiceExt};

use axie::models::User;

#[derive(serde::Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
}

fn create_test_request(method: &str, uri: &str) -> axum::http::request::Builder {
    let peer_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    Request::builder()
        .method(method)
        .uri(uri)
        .extension(ConnectInfo(peer_addr))
}

fn get_test_token(user_id: &str) -> String {
    sign_token(
        user_id.to_string(),
        "Microsoft".to_string(),
        axie::models::Role::Admin,
    )
    .expect("Failed to sign test token")
}

fn get_owner_token(user_id: &str, company: &str) -> String {
    sign_token(
        user_id.to_string(),
        company.to_string(),
        axie::models::Role::Owner,
    )
    .expect("Failed to sign owner test token")
}

async fn setup_test_app() -> axum::Router {
    unsafe {
        std::env::set_var("JWT_SECRET", "test_super_secret_key_123");
    }

    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    db.push_schema()
        .await
        .expect("Failed to sync test database schema");

    app(db).await
}

#[tokio::test]
async fn test_index_handler() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(create_test_request("GET", "/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Goodbye, World!");
}

#[tokio::test]
async fn test_about_handler() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            create_test_request("GET", "/user")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_greet_handler() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            create_test_request("GET", "/user/greet/isvane")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"Hello isvane")
}

#[tokio::test]
async fn test_create_user_handle() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Isvane", "email": "isvane@testmail.com", "password": "supersecurepassword123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"Created user successfully");
}

#[tokio::test]
async fn test_delete_user_handle() {
    let mut app = setup_test_app().await;

    let response1 = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Isvane", "email": "isvane@testmail.com", "password": "supersecurepassword123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let token = get_test_token("1");

    let response2 = app
        .call(
            create_test_request("DELETE", "/user/1")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"Deleted user: 1");

    let response3 = app
        .call(
            create_test_request("DELETE", "/user/1")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response3.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_user_handle() {
    let mut app = setup_test_app().await;

    let response1 = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Isvane", "email": "isvane@testmail.com", "password": "supersecurepassword123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let token = get_test_token("1");

    let response2 = app
        .call(
            create_test_request("PATCH", "/user/1")
                .header("Authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Johny", "email": "john@testmail.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"Updated user: 1");
}

#[tokio::test]
async fn test_list_users_handle() {
    let mut app = setup_test_app().await;

    let response1 = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Isvane", "email": "isvane@testmail.com", "password": "supersecurepassword123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let token = get_test_token("1");

    let response2 = app
        .call(
            create_test_request("GET", "/admin/list")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();

    let users: Vec<User> = serde_json::from_slice(&body).unwrap();

    assert_eq!(users.len(), 1);
}

#[tokio::test]
async fn test_list_items() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            create_test_request("GET", "/pages?page=2&per_page=50")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"Page 2, 50 items")
}

#[tokio::test]
async fn test_validator_name() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "", "email": "isvane@testmail.com", "password": "supersecurepassword123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let body_string = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_string.contains("Name cannot be empty"));
}

#[tokio::test]
async fn test_validator_email() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Isvane", "email": "not-email", "password": "supersecurepassword123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let body_string = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_string.contains("Invalid email address"));
}

#[tokio::test]
async fn test_change_user_role_success_as_admin() {
    let mut app = setup_test_app().await;

    let response1 = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Regular User", "email": "regular@testmail.com", "password": "password123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let admin_token = get_test_token("2");

    let response2 = app
        .call(
            create_test_request("PATCH", "/admin/1/role")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role": "admin"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"Role updated");
}

#[tokio::test]
async fn test_change_user_role_forbidden_as_regular_user() {
    let mut app = setup_test_app().await;

    let response1 = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Regular User", "email": "regular@testmail.com", "password": "password123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let user_token = crate::auth::sign_token(
        "1".to_string(),
        "TestCompany".to_string(),
        axie::models::Role::User,
    )
    .expect("Failed to sign test token");

    let response2 = app
        .call(
            create_test_request("PATCH", "/admin/1/role")
                .header("Authorization", format!("Bearer {}", user_token))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role": "admin"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_transfer_ownership_success() {
    let mut app = setup_test_app().await;

    let _ = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Owner User", "email": "owner@microsoft.com", "password": "password123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "New Owner", "email": "new_owner@microsoft.com", "password": "password123", "company": "Microsoft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let owner_token = get_owner_token("1", "Microsoft");

    let response = app
        .call(
            create_test_request("POST", "/owner/transfer-ownership")
                .header("Authorization", format!("Bearer {}", owner_token))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"new_owner_id": 2}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Ownership successfully transferred"));
}

#[tokio::test]
async fn test_transfer_ownership_forbidden_for_admin() {
    let mut app = setup_test_app().await;

    let admin_token = get_test_token("1");

    let response = app
        .call(
            create_test_request("POST", "/owner/transfer-ownership")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"new_owner_id": 2}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_transfer_ownership_cross_company_forbidden() {
    let mut app = setup_test_app().await;

    let _ = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Google User", "email": "user@google.com", "password": "password123", "company": "Google"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let owner_token = get_owner_token("2", "Microsoft");

    let response = app
        .call(
            create_test_request("POST", "/owner/transfer-ownership")
                .header("Authorization", format!("Bearer {}", owner_token))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"new_owner_id": 1}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn register_and_login(
    app: &mut axum::Router,
    name: &str,
    email: &str,
    password: &str,
    company: &str,
) -> AuthResponse {
    let create_res = app
        .call(
            create_test_request("POST", "/user/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": name,
                        "email": email,
                        "password": password,
                        "company": company,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let login_res = app
        .call(
            create_test_request("POST", "/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "email": email,
                        "password": password,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login_res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(login_res.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_login_success_and_access_protected_route() {
    let mut app = setup_test_app().await;

    let auth = register_and_login(
        &mut app,
        "Isvane",
        "isvane@testmail.com",
        "supersecurepassword123",
        "Microsoft",
    )
    .await;

    assert_eq!(auth.token_type, "Bearer");
    assert!(!auth.access_token.is_empty());
    assert!(!auth.refresh_token.is_empty());

    let update_res = app
        .call(
            create_test_request("PATCH", "/user/1")
                .header("Authorization", format!("Bearer {}", auth.access_token))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Isvane Updated", "email": "isvane@testmail.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_wrong_credentials() {
    let mut app = setup_test_app().await;

    let _ = register_and_login(
        &mut app,
        "Isvane",
        "isvane@testmail.com",
        "supersecurepassword123",
        "Microsoft",
    )
    .await;

    let response = app
        .call(
            create_test_request("POST", "/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email": "isvane@testmail.com", "password": "wrongpassword"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_refresh_token_rotation_success() {
    let mut app = setup_test_app().await;

    let initial = register_and_login(
        &mut app,
        "Isvane",
        "isvane@testmail.com",
        "supersecurepassword123",
        "Microsoft",
    )
    .await;

    let refresh_res = app
        .call(
            create_test_request("POST", "/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "refresh_token": initial.refresh_token }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refresh_res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(refresh_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let rotated: AuthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(rotated.token_type, "Bearer");
    assert!(!rotated.access_token.is_empty());
    assert_ne!(initial.refresh_token, rotated.refresh_token);

    let update_res = app
        .call(
            create_test_request("PATCH", "/user/1")
                .header("Authorization", format!("Bearer {}", rotated.access_token))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Rotated User", "email": "isvane@testmail.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_refresh_token_reuse_detection_revokes_all_sessions() {
    let mut app = setup_test_app().await;

    let initial = register_and_login(
        &mut app,
        "Isvane",
        "isvane@testmail.com",
        "supersecurepassword123",
        "Microsoft",
    )
    .await;

    let first_refresh = app
        .call(
            create_test_request("POST", "/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "refresh_token": initial.refresh_token }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_refresh.status(), StatusCode::OK);

    let body = axum::body::to_bytes(first_refresh.into_body(), usize::MAX)
        .await
        .unwrap();
    let rotated: AuthResponse = serde_json::from_slice(&body).unwrap();

    let replay_res = app
        .call(
            create_test_request("POST", "/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "refresh_token": initial.refresh_token }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(replay_res.status(), StatusCode::UNAUTHORIZED);

    let subsequent_res = app
        .call(
            create_test_request("POST", "/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "refresh_token": rotated.refresh_token }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(subsequent_res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_refresh_updates_user_role_claims() {
    let mut app = setup_test_app().await;

    let initial = register_and_login(
        &mut app,
        "Regular User",
        "regular@testmail.com",
        "supersecurepassword123",
        "Microsoft",
    )
    .await;

    let forbidden_res = app
        .call(
            create_test_request("GET", "/admin/list")
                .header("Authorization", format!("Bearer {}", initial.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden_res.status(), StatusCode::FORBIDDEN);

    let admin_token = get_test_token("99");
    let promote_res = app
        .call(
            create_test_request("PATCH", "/admin/1/role")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role": "admin"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(promote_res.status(), StatusCode::OK);

    let refresh_res = app
        .call(
            create_test_request("POST", "/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "refresh_token": initial.refresh_token }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(refresh_res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(refresh_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let refreshed: AuthResponse = serde_json::from_slice(&body).unwrap();

    let allowed_res = app
        .call(
            create_test_request("GET", "/admin/list")
                .header(
                    "Authorization",
                    format!("Bearer {}", refreshed.access_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(allowed_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_refresh_invalid_token() {
    let mut app = setup_test_app().await;

    let response = app
        .call(
            create_test_request("POST", "/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"refresh_token": "non_existent_random_token"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_revokes_refresh_token() {
    let mut app = setup_test_app().await;

    let auth = register_and_login(
        &mut app,
        "Isvane",
        "isvane@testmail.com",
        "supersecurepassword123",
        "Microsoft",
    )
    .await;

    let logout_res = app
        .call(
            create_test_request("POST", "/logout")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "refresh_token": auth.refresh_token }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout_res.status(), StatusCode::NO_CONTENT);

    let refresh_res = app
        .call(
            create_test_request("POST", "/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "refresh_token": auth.refresh_token }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(refresh_res.status(), StatusCode::UNAUTHORIZED);
}
