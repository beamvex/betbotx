use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;
use std::fs;

fn expand_tilde_path(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}

fn build_client_identity(cert_path: &str, key_path: &str) -> Result<reqwest::Identity> {
    let cert_pem = fs::read(cert_path).with_context(|| format!("reading cert file: {cert_path}"))?;
    let key_pem = fs::read(key_path).with_context(|| format!("reading key file: {key_path}"))?;

    let mut combined = Vec::with_capacity(cert_pem.len() + 1 + key_pem.len());
    combined.extend_from_slice(&cert_pem);
    combined.push(b'\n');
    combined.extend_from_slice(&key_pem);

    reqwest::Identity::from_pem(&combined).context("parsing client identity from PEM")
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let username = env::var("BETFAIR_USERNAME").context("BETFAIR_USERNAME not set")?;
    let password = env::var("BETFAIR_PASSWORD").context("BETFAIR_PASSWORD not set")?;
    let app_key = env::var("BETFAIR_APP_KEY").context("BETFAIR_APP_KEY not set")?;

    let cert_path = expand_tilde_path(
        &env::var("BETFAIR_CERT").unwrap_or_else(|_| "client-2048.crt".to_string()),
    );
    let key_path =
        expand_tilde_path(&env::var("BETFAIR_KEY").unwrap_or_else(|_| "client-2048.key".to_string()));
    let insecure = env::var("BETFAIR_INSECURE").ok().as_deref() == Some("1");

    let identity = build_client_identity(&cert_path, &key_path)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Application",
        HeaderValue::from_str(&app_key).context("invalid BETFAIR_APP_KEY for header")?,
    );

    let client = reqwest::Client::builder()
        .identity(identity)
        .default_headers(headers)
        .danger_accept_invalid_certs(insecure)
        .build()
        .context("building reqwest client")?;

    let resp = client
        .post("https://identitysso-cert.betfair.com/api/certlogin")
        .form(&[("username", username), ("password", password)])
        .send()
        .await
        .context("sending certlogin request")?;

    let status = resp.status();
    let body = resp.text().await.context("reading response body")?;

    println!("HTTP {status}");
    println!("{body}");

    Ok(())
}
