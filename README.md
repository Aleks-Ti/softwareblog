# softwareblog

Личный блог на Rust: Axum 0.7, SQLx 0.8, PostgreSQL, Tera (server-side rendering).

## Быстрый старт

```bash
cp .env.example .env
# Заполни .env: DATABASE_URL, ADMIN_PASSWORD_HASH, COOKIE_SECRET (>= 64 байт)
docker compose up -d   # PostgreSQL
cargo run
```

Генерация хеша пароля:
```bash
cargo run --bin seed
```

---

## Архитектура

### Общая схема

```
HTTP Request
  → Axum Router
  → [auth_guard middleware]
  → Handler (web layer)       — только: распарсить запрос, вызвать use case, рендерить ответ
  → Use Case / Service        — бизнес-логика, оркестрация
  → Repository trait (порт)  — абстракция над хранилищем
  → PostgresRepository (адаптер) — конкретный SQL
  → PostgreSQL
```

### Слои и их границы

```
src/
  domain/          — бизнес-сущности, трейты-репозитории, DomainError
  application/     — use cases, сервисы-оркестраторы, ApplicationError
  infrastructure/  — реализации репозиториев (PostgreSQL + SQLx)
  web/             — HTTP handlers, роутер, AppState, AppError
```

Ключевое правило: зависимости идут **только сверху вниз**.
`web` знает про `application`, `application` знает про `domain`,
`infrastructure` реализует трейты из `domain` — и больше ничего.
`domain` не знает ни о ком.

---

### Чем DDD на Rust отличается от Python/FastAPI

В FastAPI типичный паттерн — сервис напрямую импортирует SQLAlchemy модели,
handler получает сервис через `Depends()`, ошибки это `HTTPException`.

Здесь границы жёстче, потому что Rust проверяет их на этапе компиляции:

| FastAPI | Rust/DDD |
|---|---|
| `HTTPException(404)` в сервисе | `DomainError::NotFound` — без HTTP |
| `Depends(PostService)` в handler | `Arc<dyn PostServicePort>` — через трейт |
| Один класс сервиса на всё | Отдельный `GetPostBySlug` use case struct |
| Ошибки через exception hierarchy | Три явных enum с `From<>` конверсиями |

---

### Три уровня ошибок

```
DomainError (domain/errors.rs)
  ↓ From<>
ApplicationError (application/errors.rs)
  ↓ From<>
AppError (web/errors.rs)  →  IntoResponse → HTTP
```

**Зачем три, а не один?**

Один глобальный тип (`AppError` везде, как было изначально) приводит к тому,
что домен знает про HTTP-статусы, а инфраструктура знает про шаблоны Tera.
Это нарушение изоляции слоёв.

- `DomainError` — бизнес-язык: "пост не найден", "slug уже занят".
  Не знает про HTTP, SQLx, Tera.
- `ApplicationError` — оркестрация: что пошло не так в use case.
  Знает о `DomainError`, не знает про HTTP.
- `AppError` — HTTP-язык: какой статус отдать, что показать пользователю.
  Знает обо всём, живёт только в web-слое.

Конверсия через `?` оператор работает автоматически:
```rust
// В handler'е (возвращает Result<_, AppError>):
let post = state.get_post_by_slug.execute(&slug).await?;
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ возвращает ApplicationError
//                                                    ? вызывает From<ApplicationError> for AppError
```

Добавление нового варианта `DomainError` → ошибка компиляции в `From<DomainError> for AppError`
→ компилятор не даст забыть обработать новый случай.

---

### Почему use case — отдельная struct, а не метод сервиса

`PostService` — это сервис-оркестратор с доступом к `PostRepository` + `TagRepository`.
`GetPostBySlug` — use case с доступом только к `PostRepository`.

Преимущества отдельной struct:

1. **Явные зависимости**: конструктор показывает точно, что нужно use case'у.
2. **Меньше поверхность**: для теста нужен только `MockPostRepository`, не весь сервис.
3. **Single Responsibility**: один use case — одна операция.

В FastAPI аналог: вместо `Depends(PostService)` использовать
`Depends(GetPostBySlugUseCase)` — специализированный класс с минимальными зависимостями.

---

### Как читать поток запроса: GET /posts/:slug

```
1. HTTP GET /posts/my-first-post
   → Axum router совпадает с паттерном /posts/:slug
   → вызывает handlers::posts::show

2. show(State(state), Path("my-first-post"))
   → state.get_post_by_slug.execute("my-first-post").await?

3. GetPostBySlug::execute("my-first-post")
   → self.post_repo.find_by_slug("my-first-post").await?
   → если None → DomainError::NotFound → ApplicationError → AppError → HTTP 404
   → если Draft → DomainError::NotFound (черновики скрыты) → HTTP 404
   → если Published → Ok(post)

4. Обратно в handler:
   → state.posts.all_tags().await?       // тоже ApplicationError → AppError через ?
   → state.comments.for_post(...).await?
   → tera.render("posts/detail.html", ctx) // tera::Error → AppError::Template
   → Ok(Html(html))

5. HTTP 200 с HTML
```

---

### AppState — DI-контейнер

```rust
pub struct AppState {
    pub posts: Arc<dyn PostServicePort>,    // трейт, не конкретный PostService
    pub get_post_by_slug: Arc<GetPostBySlug>, // use case как отдельная зависимость
    pub comments: Arc<CommentService>,
    ...
}
```

`Arc<dyn PostServicePort>` вместо `Arc<PostService>`:
- handler'ы не знают какая конкретно реализация внутри
- в тестах подаём `Arc<MockPostService>`
- можно добавить кэширующий декоратор без изменения handler'ов

---

### Что ещё сделать (самостоятельная работа)

1. **`application/create_post.rs`** — реализовать `CreatePost::execute()` по образцу `get_post_by_slug.rs`
2. **Admin handler** — подключить use case `CreatePost` вместо прямого вызова через PostServicePort
3. **`CommentServicePort`** — ввести трейт для CommentService по аналогии с PostServicePort
4. **`domain/post.rs`** — добавить `validate_for_publish(&self) -> Result<(), DomainError>`
5. **Тесты** — написать unit тест для `GetPostBySlug` с мок-репозиторием
