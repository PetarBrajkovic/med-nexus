use dotenvy::dotenv;
use std::{
    collections::HashMap,
    env::{self},
    net::{AddrParseError, SocketAddr},
};

#[derive(Debug)]
#[allow(dead_code)]
// used later when we input the server and DB
struct Config {
    database_url: String,
    python_url: String,
    bind_address: SocketAddr,
}

#[derive(Debug)]
enum ConfigError {
    Missing { key: String },
    Invalid { reason: String, key: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Missing { key } => {
                write!(f, "Missing {key} from the environment variables")
            }
            ConfigError::Invalid { reason, key } => {
                write!(f, "The {key} is invalid due to: {reason}")
            }
        }
    }
}

fn main() {
    dotenv().ok();

    let vars: HashMap<String, String> = env::vars().collect();
    let config = match load_config(&vars) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1)
        }
    };

    print!("Config is {config:?}");
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
    vars.get(key).cloned().ok_or_else(|| ConfigError::Missing {
        key: String::from(key),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_when_all_present() {
        let vars = fill_map();

        let config_param = load_config(&vars).unwrap();
        assert_eq!(config_param.database_url, "postgres://x");
    }

    #[test]
    fn fails_when_one_missing() {
        let mut vars = fill_map();
        vars.remove(&String::from("PYTHON_URL"));

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

    fn fill_map() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("DATABASE_URL".to_string(), "postgres://x".to_string());
        vars.insert("PYTHON_URL".to_string(), "postgres://x".to_string());
        vars.insert("BIND_ADDR".to_string(), "127.0.0.1:3000".to_string());
        vars
    }
}
