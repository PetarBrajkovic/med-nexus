use axum::{Json, Router, routing::get};
use dotenvy::dotenv;
use std::{
    collections::HashMap,
    env,
    net::{AddrParseError, SocketAddr},
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, info};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[allow(dead_code)]
#[derive(Debug)]
// used later when we input the DB and Python server
struct Config {
    database_url: String,
    python_url: String,
    bind_address: SocketAddr,
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, thiserror::Error)]
enum ConfigError {
    #[error("missing or empty {key} from the environment variables")]
    Missing { key: String },
    #[error("the {key} is invalid due to: {reason}")]
    Invalid { key: String, reason: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let vars: HashMap<String, String> = env::vars().collect();
    let config = load_config(&vars)?;

    let listener = tokio::net::TcpListener::bind(config.bind_address).await?;
    info!("listening on: {}", listener.local_addr()?);

    axum::serve(listener, app()).await?;
    Ok(())
}

fn app() -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    Router::new()
        .route("/api/health", get(health))
        .layer(trace_layer)
}

async fn health() -> Json<HealthResponse> {
    let response = HealthResponse { status: "ok" };
    Json(response)
}

fn load_config(map: &HashMap<String, String>) -> Result<Config, ConfigError> {
    let database_url = read_env(map, "DATABASE_URL")?;
    let python_url = read_env(map, "PYTHON_URL")?;

    let bind_address_raw = read_env(map, "BIND_ADDR")?;
    let bind_address: SocketAddr =
        bind_address_raw
            .parse()
            .map_err(|e: AddrParseError| ConfigError::Invalid {
                reason: e.to_string(),
                key: (String::from("BIND_ADDR")),
            })?;

    Ok(Config {
        database_url,
        python_url,
        bind_address,
    })
}

fn read_env(vars: &HashMap<String, String>, key: &str) -> Result<String, ConfigError> {
    vars.get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| ConfigError::Missing {
            key: String::from(key),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, body::to_bytes, http::Request, http::StatusCode};
    use tower::ServiceExt;

    #[test]
    fn loads_when_all_present() {
        let vars = fill_map();

        let config_param = load_config(&vars).unwrap();
        assert_eq!(config_param.database_url, "mysql://x");
    }

    #[test]
    fn fails_when_one_missing() {
        let mut vars = fill_map();
        vars.remove("PYTHON_URL");

        let result = load_config(&vars);
        assert!(matches!(result, Err(ConfigError::Missing { key }) if key == "PYTHON_URL"));
    }

    #[test]
    fn fails_when_bind_addr_malformed() {
        let mut vars = fill_map();
        vars.insert(String::from("BIND_ADDR"), String::from("nestotamo"));

        let result = load_config(&vars);
        assert!(matches!(result, Err(ConfigError::Invalid { key, .. }) if key == "BIND_ADDR"));
    }

    #[test]
    fn fails_when_one_is_empty() {
        let mut vars = fill_map();
        vars.insert(String::from("PYTHON_URL"), String::from("  "));

        let result = load_config(&vars);
        assert!(matches!(result, Err(ConfigError::Missing { key, .. }) if key == "PYTHON_URL"));
    }

    fn fill_map() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("DATABASE_URL".to_string(), "mysql://x".to_string());
        vars.insert(
            "PYTHON_URL".to_string(),
            "http://127.0.0.1:8000".to_string(),
        );
        vars.insert("BIND_ADDR".to_string(), "127.0.0.1:3000".to_string());
        vars
    }

    #[tokio::test]
    async fn router_path_not_found() {
        let req = Request::builder()
            .method("GET")
            .uri("/nesto_tamo/ne_postoji")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let req = Request::builder()
            .method("GET")
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, r#"{"status":"ok"}"#);
    }
}
