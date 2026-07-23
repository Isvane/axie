use super::*;
use crate::auth::sign_token;
use crate::models::User;
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use std::net::SocketAddr;
use tower::{Service, ServiceExt};

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
        models::Role::Admin,
    )
    .expect("Failed to sign test token")
}

fn get_owner_token(user_id: &str, company: &str) -> String {
    sign_token(
        user_id.to_string(),
        company.to_string(),
        models::Role::Owner,
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

    app(db)
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
            create_test_request("GET", "/users")
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
            create_test_request("GET", "/users/greet/isvane")
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
            create_test_request("POST", "/users/create")
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
            create_test_request("POST", "/users/create")
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
            create_test_request("DELETE", "/users/delete/1")
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
            create_test_request("DELETE", "/users/delete/1")
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
            create_test_request("POST", "/users/create")
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
            create_test_request("PATCH", "/users/update/1")
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
            create_test_request("POST", "/users/create")
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
            create_test_request("POST", "/users/create")
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
            create_test_request("POST", "/users/create")
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
            create_test_request("POST", "/users/create")
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
            create_test_request("POST", "/users/create")
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
        models::Role::User,
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
            create_test_request("POST", "/users/create")
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
            create_test_request("POST", "/users/create")
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
            create_test_request("POST", "/users/create")
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
