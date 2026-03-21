use std::sync::Arc;

use axum_extra::extract::cookie::Key;
use sqlx::postgres::PgPoolOptions;
use tera::Tera;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod application;
mod config;
mod domain;
mod error;
mod infrastructure;
mod web;

use application::{comment_service::CommentService, post_service::PostService};
use infrastructure::postgres::{
    comment_repo::PostgresCommentRepository, post_repo::PostgresPostRepository,
    tag_repo::PostgresTagRepository,
};
use web::{router, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "softwareblog=debug,tower_http=debug".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env()?;

    // Database
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Running migrations…");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Templates
    let tera = Tera::new("templates/**/*")?;

    // Cookie signing key (must be exactly 64 bytes after base64 decode)
    let cookie_key = Key::derive_from(config.cookie_secret.as_bytes());

    // Repositories (infrastructure layer)
    let post_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    let comment_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));
    let tag_repo = Arc::new(PostgresTagRepository::new(pool.clone()));

    // Services (application layer)
    let post_service = Arc::new(PostService::new(post_repo.clone(), tag_repo));
    let comment_service = Arc::new(CommentService::new(comment_repo, post_repo));

    let state = AppState {
        tera: Arc::new(tera),
        posts: post_service,
        comments: comment_service,
        config: Arc::new(config.clone()),
        cookie_key,
    };

    let app = router::build(state);
    let addr = config.socket_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
