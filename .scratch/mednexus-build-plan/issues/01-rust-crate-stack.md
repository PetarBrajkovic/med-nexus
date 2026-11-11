Type: research
Status: resolved
Blocked by: —

# Rust crate stack for a beginner

## Question

Which crates should the Rust API service use, given that the author is a **Rust beginner** writing everything by hand? For each area, compare the 1–2 leading options on documentation quality and beginner-friendliness, MySQL support and maturity (based on the official docs/repo):

- Web framework: axum vs actix-web (async runtime: tokio)
- MySQL access: sqlx (compile-time checked queries, migrations) vs sea-orm vs diesel
- Password hashing (argon2) and auth (JWT via `jsonwebtoken` vs session cookies, e.g. tower-sessions)
- Multipart PDF upload
- HTTP client for calling the Python AI service (reqwest)
- Serde/JSON, configuration (.env / config), logging/tracing, error handling (thiserror/anyhow)
- Serving static files (tower-http ServeDir) and CORS
- Recommended learning path: which Rust Book chapters to read before the first task

Output: a recommended stack with a short reason per choice and links to the official docs, to be used later in the backlog.

## Answer

Recommendation (details, comparisons and sources: `docs/research/rust-crate-stack.md` on the `research/rust-crate-stack` branch, never merged, read with `git show research/rust-crate-stack:docs/research/rust-crate-stack.md`):

- **axum 0.8** on tokio; **tower-http** for `ServeDir` (frontend) and `CorsLayer`
- **sqlx 0.9** (`mysql`, `migrate`) for MySQL; commit `.sqlx/` (`cargo sqlx prepare`) so builds don't need a live DB; MySQL 8.4 over TLS
- **argon2** for passwords; for auth the research leans toward **tower-sessions** (cookie session + MySQL store) over JWT. That's a decision for 07, not settled here
- **reqwest** with an explicit timeout to Python (answers take 30–60 s); **serde**, **dotenvy**, **tracing**; **thiserror** (domain errors + `IntoResponse`), **anyhow** for main/setup
- Windows: MSVC C++ Build Tools are required; the sqlx MySQL driver is pure Rust
- Learning path: Rust Book ch. 2–11 + 17 (async), then the axum examples (testing, error-handling, multipart) and the sqlx/sqlx-cli READMEs
- Unverified: exact sqlx 0.9 default features and the offline-mode change (see Gaps in the file)
