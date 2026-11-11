Type: research
Status: open
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
