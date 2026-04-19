use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

#[derive(Debug, Default, Deserialize)]
pub struct BetfairSession {
    #[serde(rename = "sessionToken")]
    pub session_token: String,
    #[serde(rename = "loginStatus")]
    pub login_status: String,
}

#[derive(Debug, Deserialize)]
pub struct KeepAliveResponse {
    #[serde(rename = "token")]
    pub token: String,
    #[serde(rename = "product")]
    pub product: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "error")]
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum BetfairDomain {
    Com,
    It,
    Es,
}

impl BetfairDomain {
    fn host(self) -> &'static str {
        match self {
            BetfairDomain::Com => "api.betfair.com",
            BetfairDomain::It => "api.betfair.it",
            BetfairDomain::Es => "api.betfair.es",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NavigationId(pub Value);

#[derive(Debug, Deserialize, Serialize)]
pub struct NavigationNode {
    #[serde(default)]
    pub children: Vec<NavigationNode>,
    pub id: NavigationId,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,

    #[serde(default)]
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,
    #[serde(default)]
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    #[serde(rename = "raceNumber")]
    pub race_number: Option<String>,

    #[serde(default)]
    #[serde(rename = "exchangeId")]
    pub exchange_id: Option<Value>,
    #[serde(default)]
    #[serde(rename = "marketStartTime")]
    pub market_start_time: Option<String>,
    #[serde(default)]
    #[serde(rename = "marketType")]
    pub market_type: Option<String>,
    #[serde(default)]
    #[serde(rename = "numberOfWinners")]
    pub number_of_winners: Option<Value>,
}

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

        let resp = self.call_api(session_token, &url, "GET").await?;

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
    ) -> Result<reqwest::Response> {
        

        let builder = self.build_request(session_token, url, method)?;

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

        let resp = self.call_api(session_token, &url, "GET").await?;

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

        let resp = self.call_api(session_token, &url, "GET").await?;

        Ok(resp)

    }


    pub async fn list_market_books(
        &self,
        session_token: &str,
        locale: &str,
        domain: BetfairDomain,
    ) -> Result<reqwest::Response> {
    
        let url = format!(
            "https://{}/exchange/betting/rest/v1.0/listMarketBooks/",
            domain.host()
        );

        let resp = self.call_api(session_token, &url, "GET").await?;

        Ok(resp)

    }

}
