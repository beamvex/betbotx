use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ClientConfig, RootCertStore};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

fn read_certs_from_pem(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let f = File::open(path).with_context(|| format!("opening cert file: {path}"))?;
    let mut reader = BufReader::new(f);
    let certs = rustls_pemfile::certs(&mut reader).collect::<std::result::Result<Vec<_>, _>>()?;
    if certs.is_empty() {
        return Err(anyhow!("no certificates found in {path}"));
    }
    Ok(certs)
}

fn read_private_key_from_pem(path: &str) -> Result<PrivateKeyDer<'static>> {
    let f = File::open(path).with_context(|| format!("opening key file: {path}"))?;
    let mut reader = BufReader::new(f);

    let mut keys = rustls_pemfile::private_key(&mut reader)?;
    match keys.take() {
        Some(k) => Ok(k),
        None => Err(anyhow!("no private key found in {path}")),
    }
}

fn build_rustls_config(cert_path: &str, key_path: &str) -> Result<ClientConfig> {
    let cert_chain = read_certs_from_pem(cert_path)?;
    let key = read_private_key_from_pem(key_path)?;

    let mut roots = RootCertStore::empty();
    let native = rustls_native_certs::load_native_certs();
    for cert in native.certs {
        roots.add(cert).ok();
    }
    if roots.is_empty() {
        return Err(anyhow!(
            "no native root certificates could be loaded ({} errors)",
            native.errors.len()
        ));
    }

    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_client_auth_cert(cert_chain, key)
        .context("building rustls client config")?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let username = env::var("BETFAIR_USERNAME").context("BETFAIR_USERNAME not set")?;
    let password = env::var("BETFAIR_PASSWORD").context("BETFAIR_PASSWORD not set")?;
    let app_key = env::var("BETFAIR_APP_KEY").context("BETFAIR_APP_KEY not set")?;

    let cert_path = env::var("BETFAIR_CERT").unwrap_or_else(|_| "client-2048.crt".to_string());
    let key_path = env::var("BETFAIR_KEY").unwrap_or_else(|_| "client-2048.key".to_string());
    let insecure = env::var("BETFAIR_INSECURE").ok().as_deref() == Some("1");

    let tls_config = build_rustls_config(&cert_path, &key_path)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Application",
        HeaderValue::from_str(&app_key).context("invalid BETFAIR_APP_KEY for header")?,
    );

    let client = reqwest::Client::builder()
        .use_preconfigured_tls(Arc::new(tls_config))
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
