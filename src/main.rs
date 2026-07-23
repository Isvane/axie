use axum::{
    Router,
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod errors;
mod handlers;
mod models;

#[cfg(test)]
mod test;

use models::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap();

    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    match db.push_schema().await {
        Ok(_) => println!("Database schema initialized"),
        Err(e) => {
            if !e.to_string().contains("already exist") {
                panic!("Failed to sync database: {}", e)
            }
            println!("Schema already exists, skipping initialization");
        }
    }

    let app = app(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

pub(crate) fn app(db: toasty::db::Db) -> Router {
    let state = Arc::new(AppState { db });

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(50)
        .burst_size(200)
        .key_extractor(tower_governor::key_extractor::SmartIpKeyExtractor)
        .finish()
        .unwrap();

    let user_routes = Router::new()
        .route("/", get(handlers::users::about))
        .route("/create", post(handlers::users::create_user))
        .route("/delete/{id}", delete(handlers::users::delete_user))
        .route("/update/{id}", patch(handlers::users::update_users))
        .route("/greet/{name}", get(handlers::users::greet_user));

    let admin_routes = Router::new()
        .route("/list", get(handlers::admin::list_users))
        .route("/{id}/role", patch(handlers::admin::change_user_role))
        .route_layer(middleware::from_fn(|c, r, n| {
            auth::require_role(models::Role::Admin, c, r, n)
        }));

    let owner_routes = Router::new()
        .route(
            "/transfer-ownership",
            post(handlers::owner::transfer_ownership),
        )
        .route("/rename-company", post(handlers::owner::rename_company))
        .route_layer(middleware::from_fn(|c, r, n| {
            auth::require_role(models::Role::Owner, c, r, n)
        }));

    Router::new()
        .route("/", get(handlers::items::index))
        .route("/pages", get(handlers::items::list_items))
        .route("/login", post(handlers::authentication::login))
        .nest("/owner", owner_routes)
        .nest("/admin", admin_routes)
        .nest("/users", user_routes)
        .nest_service("/assets", ServeDir::new("public"))
        .fallback_service(
            ServeDir::new("public").not_found_service(ServeFile::new("public/index.html")),
        )
        .layer(GovernorLayer::new(governor_conf))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10)),
        ))
        .with_state(state)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
