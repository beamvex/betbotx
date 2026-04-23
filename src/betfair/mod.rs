use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use std::fs;

pub mod model;
pub use model::*;

pub struct BetfairClient {
    client: reqwest::Client,
    non_mtls_client: reqwest::Client,
    app_key: HeaderValue,
}

impl BetfairClient {
    pub fn new(app_key: &str, cert_path: &str, key_path: &str, insecure: bool) -> Result<Self> {
        let identity = Self::build_client_identity(cert_path, key_path)?;

        let app_key = HeaderValue::from_str(app_key).context("invalid BETFAIR_APP_KEY for header")?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Application",
            app_key.clone(),
        );

        let non_mtls_client = reqwest::Client::builder()
            .http1_only()
            .danger_accept_invalid_certs(insecure)
            .build()
            .context("building reqwest non-mTLS client")?;

        let client = reqwest::Client::builder()
            .identity(identity)
            .default_headers(headers)
            .danger_accept_invalid_certs(insecure)
            .build()
            .context("building reqwest client")?;

        Ok(Self {
            client,
            non_mtls_client,
            app_key,
        })
    }

    pub async fn cert_login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(reqwest::StatusCode, BetfairSession)> {
        let resp = self
            .client
            .post("https://identitysso-cert.betfair.com/api/certlogin")
            .form(&[("username", username), ("password", password)])
            .send()
            .await
            .context("sending certlogin request")?;

        let status = resp.status();
        
        let session = resp
            .json::<BetfairSession>()
            .await
            .context("parsing certlogin response JSON")?;

        Ok((status, session))
        
    }

    // https://api.betfair.com/exchange/account/rest/v1.0/

    pub async fn navigation_menu(
        &self,
        session_token: &str,
        locale: &str,
        domain: BetfairDomain,
    ) -> Result<(reqwest::StatusCode, NavigationNode)> {
    
        let url = format!(
            "https://{}/exchange/betting/rest/v1/{}/navigation/menu.json",
            domain.host(),
            locale
        );

        let resp = self.call_api(session_token, &url, "GET", None).await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "navigation menu request failed with HTTP {status}: {body}"
            ));
        }

        let menu = resp
            .json::<NavigationNode>()
            .await
            .context("parsing navigation menu response JSON")?;

        Ok((status, menu))

    }

    pub fn build_request(
        &self,
        session_token: &str,
        url: &str,
        method: &str,
    ) -> Result<reqwest::RequestBuilder> {
        

        let session_token =
            HeaderValue::from_str(session_token).context("invalid session token for header")?;

        let request = 
            match method {
                "POST" => self.non_mtls_client.post(url),
                "GET" => self.non_mtls_client.get(url),
                "PUT" => self.non_mtls_client.put(url),
                "DELETE" => self.non_mtls_client.delete(url),
                _ => return Err(anyhow::anyhow!("invalid method")),
            }
            .header("X-Application", self.app_key.clone())
            .header("X-Authentication", session_token)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::CONNECTION, "keep-alive");
            
        
        Ok(request)

    }

    pub async fn call_api(
        &self,
        session_token: &str,
        url: &str,
        method: &str,
        body: Option<String>,
    ) -> Result<reqwest::Response> {
        

        let builder = self.build_request(session_token, url, method)?;

        let builder = match body {
            Some(body) => builder.body(body).header("Content-Type", "application/json"),
            None => builder,
        };

        let resp = builder.send()
            .await
            .context("sending navigation menu request")?;
        
        Ok(resp)

    }

    pub async fn keep_alive(
        &self,
        session_token: &str,
    ) -> Result<(reqwest::StatusCode, KeepAliveResponse)> {
        let session_token =
            HeaderValue::from_str(session_token).context("invalid session token for header")?;

        let resp = self
            .non_mtls_client
            .get("https://identitysso.betfair.com/api/keepAlive")
            .header("X-Application", self.app_key.clone())
            .header("X-Authentication", session_token)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .context("sending keepAlive request")?;

        let status = resp.status();
        let body = resp
            .json::<KeepAliveResponse>()
            .await
            .context("parsing keepAlive response JSON")?;

        Ok((status, body))
    }

    fn build_client_identity(cert_path: &str, key_path: &str) -> Result<reqwest::Identity> {
        let cert_pem =
            fs::read(cert_path).with_context(|| format!("reading cert file: {cert_path}"))?;
        let key_pem =
            fs::read(key_path).with_context(|| format!("reading key file: {key_path}"))?;

        let mut combined = Vec::with_capacity(cert_pem.len() + 1 + key_pem.len());
        combined.extend_from_slice(&cert_pem);
        combined.push(b'\n');
        combined.extend_from_slice(&key_pem);

        reqwest::Identity::from_pem(&combined).context("parsing client identity from PEM")
    }

    pub async fn get_account_details(
        &self,
        session_token: &str,
        locale: &str,
        domain: BetfairDomain,
    ) -> Result<reqwest::Response> {
    
        let url = format!(
            "https://{}/exchange/account/rest/v1.0/getAccountDetails/",
            domain.host()
        );

        let resp = self.call_api(session_token, &url, "GET", None).await?;

        Ok(resp)

    }

    pub async fn get_account_funds(
        &self,
        session_token: &str,
        locale: &str,
        domain: BetfairDomain,
    ) -> Result<reqwest::Response> {
    
        let url = format!(
            "https://{}/exchange/account/rest/v1.0/getAccountFunds/",
            domain.host()
        );

        let resp = self.call_api(session_token, &url, "GET", None).await?;

        Ok(resp)

    }


    pub async fn list_market_books(
        &self,
        session_token: &str,
        locale: &str,
        domain: BetfairDomain,
    ) -> Result<reqwest::Response> {
    
        let url = format!(
            "https://{}/exchange/betting/rest/v1.0/listMarketBook/",
            domain.host()
        );

        let request = MarketBookRequest {
            market_ids: vec!["1.256991740".to_string()],
            price_projection: PriceProjection {
                price_data: vec!["EX_BEST_OFFERS".to_string()],
            },
        };

        let body = serde_json::to_string(&request)?;

        let resp = self.call_api(session_token, &url, "POST", Some(body)).await?;

        Ok(resp)

    }

    pub async fn list_market_catalogue(
        &self,
        session_token: &str,
        locale: &str,
        domain: BetfairDomain,
    ) -> Result<reqwest::Response> {
    
        let url = format!(
            "https://{}/exchange/betting/rest/v1.0/listMarketCatalogue/",
            domain.host()
        );

        let request = MarketCatalogueRequest {
            filter: MarketFilter {
                market_ids: vec!["1.256991740".to_string()],
            },
            market_projection: vec!["MARKET_START_TIME".to_string(), "RUNNER_DESCRIPTION".to_string(), "EVENT".to_string(), "COMPETITION".to_string(), "EVENT_TYPE".to_string(), "MARKET_DESCRIPTION".to_string()],
            max_results: 100,
        };

        

        let body = serde_json::to_string(&request)?;

        let resp = self.call_api(session_token, &url, "POST", Some(body)).await?;

        Ok(resp)

    }

}
