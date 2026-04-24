use anyhow::{Context, Result};

use super::client::BetfairClient;
use super::model::BetfairSession;

pub struct BetfairSsoClient<'a> {
    client: &'a BetfairClient,
}

impl<'a> BetfairSsoClient<'a> {
    pub fn new(client: &'a BetfairClient) -> Self {
        Self { client }
    }

    pub async fn cert_login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(reqwest::StatusCode, BetfairSession)> {
        let resp = self
            .client
            .mtls_client()
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
}
