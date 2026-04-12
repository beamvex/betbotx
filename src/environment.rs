use anyhow::{Context, Result};
use std::env;

pub struct Environment {
    pub username: String,
    pub password: String,
    pub app_key: String,
    pub cert_path: String,
    pub key_path: String,
    pub insecure: bool,
}

fn expand_tilde_path(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}

impl Environment {
    pub fn from_env() -> Result<Self> {
        let username = env::var("BETFAIR_USERNAME").context("BETFAIR_USERNAME not set")?;
        let password = env::var("BETFAIR_PASSWORD").context("BETFAIR_PASSWORD not set")?;
        let app_key = env::var("BETFAIR_APP_KEY").context("BETFAIR_APP_KEY not set")?;

        let cert_path = expand_tilde_path(
            &env::var("BETFAIR_CERT").unwrap_or_else(|_| "client-2048.crt".to_string()),
        );
        let key_path =
            expand_tilde_path(&env::var("BETFAIR_KEY").unwrap_or_else(|_| "client-2048.key".to_string()));

        let insecure = env::var("BETFAIR_INSECURE").ok().as_deref() == Some("1");

        Ok(Self {
            username,
            password,
            app_key,
            cert_path,
            key_path,
            insecure,
        })
    }
}
