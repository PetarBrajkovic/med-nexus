# Rust crate stack for the MedNexus API service

Status: research note. Data fetched 2026-11-11 (crates.io API, docs.rs, GitHub primary sources).
Scope: `backend/` (Rust API service), written by hand by a Rust beginner.

## Summary

Use **axum 0.8 + tokio + sqlx 0.9 (MySQL, compile-time checked queries, `migrate!`) + argon2 + tower-sessions (cookie sessions, MySQL store) + axum `Multipart` + reqwest with an explicit timeout + serde + dotenvy + tracing/tracing-subscriber + thiserror (app errors) and anyhow (startup) + tower-http (ServeDir, CorsLayer)**. axum and sqlx each have the best documentation and the most official examples in their area. For a browser frontend, server-side sessions in an HttpOnly cookie avoid keeping tokens in JavaScript, so sessions are recommended over JWT. Confirm that choice before the first task (see Gaps).

## Recommended stack

| Area | Crate | Version (crates.io, 2026-11-11) | Why | Doc link |
|---|---|---|---|---|
| Async runtime | tokio | 1.54.1 | Runtime that axum, sqlx, reqwest, tower-http build on | https://docs.rs/tokio |
| Web framework | axum | 0.8.9 | tokio/tower based; many official examples | https://docs.rs/axum |
| Middleware | tower-http | 0.7.1 | `ServeDir`, `CorsLayer` | https://docs.rs/tower-http |
| MySQL | sqlx (`runtime-tokio`, `mysql`, `migrate`, `tls-rustls-ring-webpki`) | 0.9.0 | Pure-Rust MySQL driver; `query_as!` checks SQL; `migrate!` | https://docs.rs/sqlx |
| Sessions | tower-sessions + `tower-sessions-sqlx-store` | 0.15.0 (store version: unverified) | Sessions as axum middleware and extractor | https://docs.rs/tower-sessions |
| Passwords | argon2 | 0.6.0 | Argon2id default, pure Rust | https://docs.rs/argon2 |
| JWT (alternative only) | jsonwebtoken | 11.1.0 | Needs one crypto backend feature | https://docs.rs/jsonwebtoken |
| HTTP client to Python AI | reqwest | 0.13.5 | Async client with per-client timeout | https://docs.rs/reqwest |
| JSON / structs | serde (`derive`), serde_json | 1.0.229 | Standard choice | https://docs.rs/serde |
| Config | dotenvy | 0.15.7 | Loads `.env` (maintained fork of dotenv) | https://docs.rs/dotenvy |
| Logging | tracing 0.1.44 + tracing-subscriber 0.3.23 | — | Structured logs; subscriber prints them | https://docs.rs/tracing, https://docs.rs/tracing-subscriber |
| App errors | thiserror | 2.0.21 | Derive `Error` for an `AppError` enum | https://docs.rs/thiserror |
| Startup / main errors | anyhow | 1.0.104 | One boxed error type for `main` | https://docs.rs/anyhow |
| Upload | axum `Multipart` (built into axum, feature `multipart`) | with axum 0.8.9 | No extra crate; 2 MB default limit | https://docs.rs/axum/latest/axum/extract/struct.Multipart.html |
| Tests | `#[tokio::test]` + axum `oneshot` via tower | with above | No extra framework | https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs |

Illustrative `Cargo.toml` (use `cargo add` for current versions, and check features on docs.rs):

```toml
[dependencies]
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.7", features = ["fs", "cors"] }
sqlx = { version = "0.9", features = ["runtime-tokio", "tls-rustls-ring-webpki", "mysql", "migrate"] }
tower-sessions = "0.15"
argon2 = "0.6"
reqwest = { version = "0.13", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
anyhow = "1"
```

The sqlx feature names come from the sqlx README. `multipart`, `fs`, `cors`, and `env-filter` are not confirmed against a fetched page (see Gaps).

## Comparisons

### 1. Web framework: axum (chosen) vs actix-web
- **axum 0.8.9**: "Take full advantage of the [`tower`] and [`tower-http`] ecosystem of" — axum README (https://github.com/tokio-rs/axum/blob/main/axum/README.md). Its error-handling, testing and multipart examples are directly reusable here.
- **actix-web 4.19.0** (crates.io and docs.rs both report this as max stable): "Actix Web is a powerful, pragmatic, and extremely fast web framework for Rust" — actix-web README (https://github.com/actix/actix-web/blob/master/actix-web/README.md). Mature, but it has its own App/HttpServer style, and fewer examples cover this exact sqlx + sessions + upload combination.
- **tokio**: "A runtime for writing reliable, asynchronous, and slim applications with the Rust programming language" — tokio README (https://github.com/tokio-rs/tokio/blob/master/README.md).
- Verdict: axum.

### 2. MySQL: sqlx (chosen) vs sea-orm vs diesel
- **sqlx 0.9.0**: "SQLx is an async, pure Rust<sub>†</sub> SQL crate featuring compile-time checked queries without a DSL." — sqlx README (https://github.com/launchbadge/sqlx). MySQL is supported: "**Database Agnostic**. Support for [PostgreSQL], [MySQL], [MariaDB], [SQLite]." (same README). It is raw SQL, so queries stay visible and auditable, which suits a thesis. The README states "SQLx is not an ORM!" (section heading).
- **sea-orm 2.0.4**: "SeaORM is a powerful ORM for building web services in Rust" — sea-orm README (https://github.com/SeaQL/sea-orm). Richer, but a larger abstraction to learn at once.
- **diesel 2.3.14**: MySQL via a feature flag: `diesel = { version = "<version>", features = ["<postgres|mysql|sqlite>"] }` — diesel README (https://github.com/diesel-rs/diesel). Mature, but its own DSL and schema macros are a steeper start than raw SQL.
- Migrations in sqlx: "`migrate`: Add support for the migration management and `migrate!` macro, which allow compile-time embedded migrations." — sqlx README, feature list.
- Verdict: sqlx.

### 3. Password hashing: argon2 (chosen)
- argon2 0.6.0: "Pure Rust implementation of the [Argon2] password hashing function." — argon2 README (https://github.com/RustCrypto/password-hashes/blob/master/argon2/README.md).
- Default variant: "**Argon2id**: (default) hybrid version combining both Argon2i and Argon2d" (same README).
- Minimum Rust version in the README badge: "rustc-1.87+" (same README).
- Verdict: argon2. bcrypt was not fetched or compared (see Gaps).

### 4. Auth: tower-sessions (recommended) vs jsonwebtoken
- **tower-sessions 0.15.0**: "This crate provides sessions, key-value pairs associated with a site visitor, as a `tower` middleware." — tower-sessions README (https://github.com/maxcountryman/tower-sessions). It offers "An `axum` Extractor for `Session`" (same README).
- MySQL store: the README's store table lists "`tower-sessions-sqlx-store` ... SQLite, Postgres, and MySQL session stores" (same README).
- **jsonwebtoken 11.1.0**: "Two crypto backends are available via features, `aws_lc_rs` and `rust_crypto`, at most one of which must be enabled." — jsonwebtoken README (https://github.com/Keats/jsonwebtoken). Stateless, but you handle expiry and revocation yourself, and the browser holds a bearer token.
- Verdict: sessions for a same-origin browser frontend. Use JWT only if other clients need stateless tokens. The tower-sessions example uses `.with_secure(false)` (plain HTTP for local dev), so set it to `true` when you serve over HTTPS (README code sample; the production advice itself is unverified).

### 5. Multipart PDF upload (axum built-in)
- "For security reasons, by default, `Multipart` limits the request body size to 2MB." — axum `multipart.rs` doc comment (https://github.com/tokio-rs/axum/blob/main/axum/src/extract/multipart.rs).
- "For security reasons, [`Bytes`] will, by default, not accept bodies larger than 2MB." and "For more details, including how to disable this limit, see [`DefaultBodyLimit`]." — axum `extract.md`, "Request body limits" (https://github.com/tokio-rs/axum/blob/main/axum/src/docs/extract.md).
- The multipart example uses `DefaultBodyLimit::disable()` (https://github.com/tokio-rs/axum/blob/main/examples/multipart-form/src/main.rs). Suggestion: use `DefaultBodyLimit::max(N)` with a size you choose, and also check size in the handler. Unverified as a recommendation.

### 6. HTTP client to the Python service: reqwest
- `ClientBuilder::timeout`: "Enables a total request timeout." and "Default is no timeout." — reqwest docs (https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html).
- Plan: build one `reqwest::Client` at startup with a timeout above the worst case (for 30–60 s answers, e.g. 120 s), reuse it for connection pooling, and map timeouts to 504 in `AppError`. `connect_timeout` is a separate option on the same page.

### 7. Config: dotenvy (chosen) vs config crate
- dotenvy 0.15.7: "A well-maintained fork of the [dotenv](https://github.com/dotenv-rs/dotenv) crate." — dotenvy README (https://github.com/allan2/dotenvy).
- "This crate is the suggested alternative for `dotenv` in security advisory RUSTSEC-2021-0141" (same README).
- The main branch holds an unpublished 0.16; use the published 0.15.7 (same README).
- config 0.15.27 is a layered config system this project does not need. Not read in full; metadata from crates.io only.

### 8. Logging: tracing + tracing-subscriber
- "Utilities for implementing and composing [`tracing`][tracing] subscribers." — tracing-subscriber README (https://github.com/tokio-rs/tracing/blob/master/tracing-subscriber/README.md).
- Use the fmt output plus an env filter so `RUST_LOG=info,sqlx=warn` works. The `env-filter` feature name is unverified (see Gaps).

### 9. Error handling: thiserror + anyhow + axum `IntoResponse`
- anyhow 1.0.104: "This library provides [`anyhow::Error`][Error], a trait object based error type" and "for easy idiomatic error handling in Rust applications." — anyhow README (https://github.com/dtolnay/anyhow).
- thiserror 2.0.21: "This library provides a convenient derive macro for the standard library's" error trait, and "See also the [`anyhow`] library for a convenient single error type to use in" application code. — thiserror README (https://github.com/dtolnay/thiserror).
- axum: "Instead of a direct `StatusCode`, it makes sense to use intermediate error type that can ultimately be converted to `Response`. This allows using `?` operator in handlers." — axum `error_handling.md` (https://github.com/tokio-rs/axum/blob/main/axum/src/docs/error_handling.md).
- Pattern: an `AppError` enum (thiserror), `impl IntoResponse for AppError`, handlers return `Result<T, AppError>`. Reference: axum examples `error-handling` (https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs) and `anyhow-error-response` (https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs), which contain `impl IntoResponse for AppError` and `impl<E> From<E> for AppError`.

### 10. Static files and CORS: tower-http
- `ServeDir`: "Service that serves files from a given directory and all its sub directories." — tower-http docs (https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html).
- `CorsLayer`: "Cors Middleware which adds headers for CORS" — tower-http cors docs (https://docs.rs/tower-http/latest/tower_http/cors/index.html).
- Inference, not from a source: serving the frontend from the same origin as the API removes the need for CORS.

### 11. Testing an axum app
- "use tower::{Service, ServiceExt}; // for `call`, `oneshot`, and `ready`" and "`Router` implements `tower::Service<Request<Body>>`" — axum testing example (https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs).
- Pattern: build the `Router`, call `.oneshot(Request::get("/").body(...))` inside `#[tokio::test]`. Database tests need a real MySQL test database.

## Windows gotchas

1. **MSVC Build Tools.** "For msvc targets these can be acquired through Visual Studio." — rustup Windows MSVC page (https://rust-lang.github.io/rustup/installation/windows-msvc.html). "You can replace “Community” with “BuildTools” in the above command if you already have a Visual Studio license." (same page). Install the C++ build tools workload before running `cargo`.
2. **C linker.** "you should install a C compiler, which will typically include a linker" — The Rust Book, ch01 installation (https://doc.rust-lang.org/book/ch01-01-installation.html).
3. **No MySQL C library.** "The Postgres and MySQL/MariaDB drivers are written in pure Rust using **zero** unsafe<sub>††</sub> code." — sqlx README (https://github.com/launchbadge/sqlx). You do not need `libmysqlclient` for this driver.
4. **`DATABASE_URL` at build time.** "The `DATABASE_URL` environment variable must be set at build time to a database which it can prepare queries against" — sqlx README, "Compile-time verification".
5. **Offline mode (`.sqlx`).** "Invoking `prepare` saves query metadata to `.sqlx` in the current directory." and "Check this directory into version control and an active database connection will no longer be needed to build your project." — sqlx-cli README (https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md), "Enable building in offline mode". Install with `cargo install sqlx-cli`; run `cargo sqlx prepare` after each query change; commit `.sqlx/`.
6. **Accidental `DATABASE_URL`.** "The presence of a `DATABASE_URL` environment variable will take precedence over the presence of `.sqlx`" and set `SQLX_OFFLINE` to `true` to force offline builds — sqlx-cli README, "Force building in offline mode".
7. **Offline mode in 0.9.** "Offline mode is now always enabled. See [sqlx-cli/README.md]" — sqlx README, release notes list. Not confirmed against the CHANGELOG (see Gaps).
8. **MySQL 8.4 auth.** "Note: RSA auth without TLS requires `mysql-rsa` (not enabled by `mysql`)." and "Prefer using TLS." — sqlx README, feature list. Use the TLS feature from the snippet and avoid `mysql-rsa`.
9. **Rust version.** argon2 README badge: "rustc-1.87+". Run `rustup update stable` before adding the crates.
10. **JWT backend on Windows.** jsonwebtoken needs `aws_lc_rs` or `rust_crypto`. Whether `aws_lc_rs` needs CMake/NASM on Windows is unverified; prefer `rust_crypto` if you use JWT.

## Learning path (before the first task)

Rust Book, in this order (titles verified on doc.rust-lang.org/book):
1. Ch 2 "Programming a Guessing Game" — first `cargo` project (https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html)
2. Ch 3 "Common Programming Concepts" (https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html)
3. Ch 4 "Understanding Ownership" — hardest part; needed for every `String`/`&str` in handlers (https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
4. Ch 5 "Using Structs to Structure Related Data" — request/response/row types (https://doc.rust-lang.org/book/ch05-00-structs.html)
5. Ch 6 "Enums and Pattern Matching" — `Option`, `Result`, `AppError` (https://doc.rust-lang.org/book/ch06-00-enums.html)
6. Ch 8 "Common Collections" — `Vec`, `HashMap` (https://doc.rust-lang.org/book/ch08-00-common-collections.html)
7. Ch 9 "Error Handling" — "Rust groups errors into two major categories: recoverable and unrecoverable" (https://doc.rust-lang.org/book/ch09-00-error-handling.html)
8. Ch 10 "Generic Types, Traits, and Lifetimes" — needed to read `impl IntoResponse` (https://doc.rust-lang.org/book/ch10-00-generics.html)
9. Ch 7 "Packages, Crates, and Modules" (https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
10. Ch 11 "Writing Automated Tests" (https://doc.rust-lang.org/book/ch11-00-testing.html)
11. Ch 17 "Fundamentals of Asynchronous Programming: Async, Await, Futures, and Streams" — read before axum/sqlx (https://doc.rust-lang.org/book/ch17-00-async-await.html)
Optional for later: Ch 13 iterators/closures; Ch 15 smart pointers; Ch 16 concurrency.

Then official examples and READMEs:
1. axum `testing` example (https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs)
2. axum `error-handling` and `anyhow-error-response` examples (links in section 9)
3. axum `multipart-form` example (https://github.com/tokio-rs/axum/blob/main/examples/multipart-form/src/main.rs)
4. sqlx README: Quickstart, Connecting, Compile-time verification (https://github.com/launchbadge/sqlx)
5. sqlx-cli README, offline mode (https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
6. tower-sessions README and its axum example (https://github.com/maxcountryman/tower-sessions)

First practice task: `GET /health`, then a `POST /login` that reads one row with `sqlx::query_as!`. This meets the stack in order.

## Sources

Kept (primary, fetched in this run):
- crates.io API, `max_stable_version` and update dates for all crates in the table: https://crates.io/api/v1/crates/<name>
- docs.rs: axum 0.8.9 (https://docs.rs/axum), reqwest `ClientBuilder` (https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html), tower-http `ServeDir` and `cors`, actix-web and sea-orm version pages (https://docs.rs/crate/<name>/latest)
- sqlx README (https://github.com/launchbadge/sqlx) and sqlx-cli README (https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- axum README, docs and examples (https://github.com/tokio-rs/axum)
- tower-sessions README (https://github.com/maxcountryman/tower-sessions); jsonwebtoken README (https://github.com/Keats/jsonwebtoken); argon2 README (https://github.com/RustCrypto/password-hashes/blob/master/argon2/README.md); dotenvy README (https://github.com/allan2/dotenvy); anyhow (https://github.com/dtolnay/anyhow) and thiserror (https://github.com/dtolnay/thiserror) READMEs; tracing-subscriber README (https://github.com/tokio-rs/tracing); tokio README (https://github.com/tokio-rs/tokio); actix-web README (https://github.com/actix/actix-web); sea-orm README (https://github.com/SeaQL/sea-orm); diesel README (https://github.com/diesel-rs/diesel)
- rustup Windows MSVC page (https://rust-lang.github.io/rustup/installation/windows-msvc.html)
- The Rust Book (https://doc.rust-lang.org/book/), chapters listed in the learning path.

Dropped:
- Tutorials and blog posts: not fetched; the primary sources above cover each point.
- Pages that returned 404 in this run: a raw `actix` README path and the axum `examples/README.md`. Not used.

## Gaps

- **Not confirmed in a fetched source:** sqlx 0.9 default features (whether `macros` is default); the `multipart`, `fs`, `cors`, and `env-filter` feature names as written; `tower-sessions-sqlx-store` current version and compatibility with tower-sessions 0.15; the `aws_lc_rs` Windows build requirements; `.with_secure(true)` advice; the `DefaultBodyLimit::max(N)` suggestion.
- **sqlx 0.9 offline change:** the README says offline mode "is now always enabled". Read the sqlx CHANGELOG (https://github.com/launchbadge/sqlx/blob/main/CHANGELOG.md) to confirm what this means for Windows builds with `DATABASE_URL`.
- **argon2 alternatives** (bcrypt, password-hash crates) were not fetched.
- **Version check:** actix-web is reported as 4.19.0 by crates.io and docs.rs (update date 2026-11-09). Not investigated further. Run `cargo add` and check docs.rs before pinning any version.
- **Decision needed:** sessions (recommended) vs JWT; confirm with the project scope before task 1.
