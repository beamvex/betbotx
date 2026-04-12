use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use std::fs;

pub struct BetfairClient {
    client: reqwest::Client,
}

impl BetfairClient {
    pub fn new(app_key: &str, cert_path: &str, key_path: &str, insecure: bool) -> Result<Self> {
        let identity = build_client_identity(cert_path, key_path)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Application",
            HeaderValue::from_str(app_key).context("invalid BETFAIR_APP_KEY for header")?,
        );

        let client = reqwest::Client::builder()
            .identity(identity)
            .default_headers(headers)
            .danger_accept_invalid_certs(insecure)
            .build()
            .context("building reqwest client")?;

        Ok(Self { client })
    }

    pub async fn cert_login(&self, username: &str, password: &str) -> Result<(reqwest::StatusCode, String)> {
        let resp = self
            .client
            .post("https://identitysso-cert.betfair.com/api/certlogin")
            .form(&[("username", username), ("password", password)])
            .send()
            .await
            .context("sending certlogin request")?;

        let status = resp.status();
        let body = resp.text().await.context("reading response body")?;

        Ok((status, body))
    }
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
