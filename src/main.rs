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

use application::{
    comment_service::CommentService,
    get_post_by_slug::GetPostBySlug,
    post_service::{PostService, PostServicePort},
};
use domain::{comment::CommentRepository, post::PostRepository, tag::TagRepository};
use infrastructure::postgres::{
    comment_repo::PostgresCommentRepository, post_repo::PostgresPostRepository,
    tag_repo::PostgresTagRepository,
};
use web::{router, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "softwareblog=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Running migrations…");
    sqlx::migrate!("./migrations").run(&pool).await?;

    let tera = Tera::new("templates/**/*")?;
    // Key::from требует >= 64 байт. COOKIE_SECRET в .env должен быть достаточно длинным.
    // Для генерации: openssl rand -base64 64
    let cookie_key = Key::from(config.cookie_secret.as_bytes());

    // Repositories (infrastructure layer).
    // Явно приводим к Arc<dyn Trait>, чтобы можно было клонировать как трейт-объект.
    // Это нужно для передачи одного репозитория в несколько мест (сервис + use case).
    let post_repo: Arc<dyn PostRepository> = Arc::new(PostgresPostRepository::new(pool.clone()));
    let tag_repo: Arc<dyn TagRepository> = Arc::new(PostgresTagRepository::new(pool.clone()));
    let comment_repo: Arc<dyn CommentRepository> =
        Arc::new(PostgresCommentRepository::new(pool.clone()));

    // Application layer: сервисы и use cases.
    //
    // Обрати внимание: post_repo клонируется и передаётся в несколько мест.
    // Arc::clone() — O(1), только увеличивает счётчик ссылок, объект не копируется.
    //
    // Это аналог DI-контейнера в Python: один объект, несколько зависимых.
    let post_service: Arc<dyn PostServicePort> =
        Arc::new(PostService::new(post_repo.clone(), tag_repo.clone()));

    let get_post_by_slug = Arc::new(GetPostBySlug::new(post_repo.clone()));

    let comment_service = Arc::new(CommentService::new(comment_repo, post_repo));

    let state = AppState {
        tera: Arc::new(tera),
        posts: post_service,
        get_post_by_slug,
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
