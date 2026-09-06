use axum::{
    Router,
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use casbin::{CoreApi, DefaultModel, Enforcer, FileAdapter};
use std::sync::Arc;
use std::time::Duration;
use tokio::{signal, sync::RwLock};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axie::{auth, handlers, models::AppState};

#[cfg(test)]
mod test;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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

    match toasty::embed_migrations!().apply(&db).await {
        Ok(report) => {
            if report.applied() > 0 {
                println!("Successfully applied {} migrations", report.applied());
            } else {
                println!("Database schema is up to date");
            }
        }
        Err(e) => panic!("Failed to apply database migrations: {}", e),
    }

    let app = app(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");

    axum::serve(
        listener,
        app.await
            .into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

pub(crate) async fn app(db: toasty::db::Db) -> Router {
    let model = DefaultModel::from_file("rbac_model.conf")
        .await
        .expect("Failed to load Casbin model");
    let adapter = FileAdapter::new("rbac_policy.csv");
    let enforcer = Enforcer::new(model, adapter)
        .await
        .expect("Failed to initialize Casbin enforcer");

    let state = Arc::new(AppState {
        db,
        enforcer: Arc::new(RwLock::new(enforcer)),
    });

    let governor_conf = GovernorConfigBuilder::default()
        .per_millisecond(20)
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
        .route("/{id}/role", patch(handlers::admin::change_user_role));

    let owner_routes = Router::new()
        .route(
            "/transfer-ownership",
            post(handlers::owner::transfer_ownership),
        )
        .route("/rename-company", post(handlers::owner::rename_company));

    let protected_routes = Router::new()
        .nest("/owner", owner_routes)
        .nest("/admin", admin_routes)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::casbin_enforce,
        ));

    Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/", get(handlers::items::index))
        .route("/pages", get(handlers::items::list_items))
        .route("/login", post(handlers::authentication::login))
        .nest("/users", user_routes)
        .merge(protected_routes)
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
