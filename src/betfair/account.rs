use anyhow::Result;

use super::client::BetfairClient;
use super::model::BetfairDomain;

pub struct BetfairAccountClient<'a> {
    client: &'a BetfairClient,
}

impl<'a> BetfairAccountClient<'a> {
    pub fn new(client: &'a BetfairClient) -> Self {
        Self { client }
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

        let resp = self.client.call_api(session_token, &url, "GET", None).await?;

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

        let resp = self.client.call_api(session_token, &url, "GET", None).await?;

        Ok(resp)
    }
}
