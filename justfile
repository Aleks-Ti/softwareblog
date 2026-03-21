# Install tools: cargo install just cargo-watch sqlx-cli

# ─── Database ────────────────────────────────────────────────────────────────

db-up:
    docker compose up -d postgres

db-down:
    docker compose down

db-reset: db-down db-up migrate seed

# ─── Migrations ──────────────────────────────────────────────────────────────

migrate:
    sqlx migrate run

migrate-revert:
    sqlx migrate revert

migrate-add name:
    sqlx migrate add {{name}}

# ─── Seeding ─────────────────────────────────────────────────────────────────

seed:
    cargo run --bin seed

# ─── Development ─────────────────────────────────────────────────────────────

dev:
    cargo watch -x run

# ─── Quality ─────────────────────────────────────────────────────────────────

check:
    cargo check

lint:
    cargo clippy -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

# Run all checks (use before commit)
ci: fmt-check lint
    cargo test

# ─── Build ───────────────────────────────────────────────────────────────────

build:
    cargo build --release

# ─── Helpers ─────────────────────────────────────────────────────────────────

# Generate a proper COOKIE_SECRET (64 hex chars = 32 bytes)
gen-secret:
    @openssl rand -hex 32

# Generate admin password hash
# Usage: just gen-hash mypassword
gen-hash password:
    @cargo run -q --bin gen-hash -- "{{password}}" 2>/dev/null || echo "binary not yet implemented"
