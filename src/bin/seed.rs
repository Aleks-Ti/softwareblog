/// Seeder binary.
///
/// Usage:
///   cargo run --bin seed
///
/// Reads from .env:
///   DATABASE_URL      — required
///   ADMIN_PASSWORD    — if set, prints the argon2 hash to paste into ADMIN_PASSWORD_HASH
///
/// Idempotent: skips posts that already exist (checked by slug).

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // ── Admin password hash ──────────────────────────────────────────────────
    if let Ok(password) = std::env::var("ADMIN_PASSWORD") {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {e}"))?
            .to_string();
        println!("──────────────────────────────────────────────────────────");
        println!("Add to .env:");
        println!("ADMIN_PASSWORD_HASH={hash}");
        println!("──────────────────────────────────────────────────────────\n");
    } else {
        println!("Tip: set ADMIN_PASSWORD in .env to generate ADMIN_PASSWORD_HASH\n");
    }

    // ── Database ─────────────────────────────────────────────────────────────
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPool::connect(&database_url).await?;

    // ── Sample posts ─────────────────────────────────────────────────────────
    seed_posts(&pool).await?;

    println!("Done.");
    Ok(())
}

async fn seed_posts(pool: &PgPool) -> anyhow::Result<()> {
    let posts = vec![
        SeedPost {
            title: "Привет, блог",
            slug: "hello-blog",
            excerpt: Some("Первый пост — почему Rust и зачем вообще писать свой блог."),
            content: r#"# Привет, блог

Это первый пост. Я решил написать свой блог на **Rust**, потому что:

1. Хочу попрактиковаться в языке на реальном проекте
2. Устал от магии фреймворков — хочу понимать каждую строчку
3. Axum + HTMX — минималистичный стек без лишнего жира

## Стек

- **Axum** — HTTP-фреймворк
- **SQLx** — типобезопасные запросы к PostgreSQL
- **Tera** — шаблоны (синтаксис как у Jinja2)
- **HTMX** — интерактивность без SPA-оверхеда

Посмотрим что из этого выйдет.
"#,
        },
        SeedPost {
            title: "О DDD без фанатизма",
            slug: "pragmatic-ddd",
            excerpt: Some("Как применять Domain-Driven Design не утонув в агрегатах и value objects."),
            content: r#"# DDD без фанатизма

Когда читаешь книгу Эванса, хочется везде лепить агрегаты, value objects и доменные события.
На практике для большинства проектов это оверхед.

## Что реально полезно

**Разделение слоёв** — самое ценное из DDD:

```
domain/        ← чистые модели, никаких инфра-зависимостей
application/   ← use case'ы, оркестрация
infrastructure/ ← SQLx, файловое хранилище
web/           ← HTTP, handlers
```

**Repository trait** — позволяет тестировать логику без базы данных.

**Ubiquitous language** — называй вещи своими именами из предметной области.

## Что можно пропустить

- Агрегаты — нужны когда есть сложные инварианты консистентности
- Value objects — полезны, но `String` тоже ок для slug-а
- Domain events — только если реально нужна реакция на изменения

Главный вопрос: *решает ли эта абстракция реальную проблему?*
Если нет — не добавляй.
"#,
        },
    ];

    for post in posts {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM posts WHERE slug = $1)")
            .bind(post.slug)
            .fetch_one(pool)
            .await?;

        if exists {
            println!("skip  '{}'", post.slug);
            continue;
        }

        sqlx::query(
            "INSERT INTO posts (title, slug, content, excerpt, status, published_at)
             VALUES ($1, $2, $3, $4, 'published', NOW())",
        )
        .bind(post.title)
        .bind(post.slug)
        .bind(post.content)
        .bind(post.excerpt)
        .execute(pool)
        .await?;

        println!("seeded '{}'", post.slug);
    }

    Ok(())
}

struct SeedPost {
    title: &'static str,
    slug: &'static str,
    excerpt: Option<&'static str>,
    content: &'static str,
}
