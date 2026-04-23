use anyhow::{Context, Result};

use super::client::BetfairClient;
use super::model::{BetfairDomain, NavigationNode};

pub struct BetfairNavigationClient<'a> {
    client: &'a BetfairClient,
}

impl<'a> BetfairNavigationClient<'a> {
    pub fn new(client: &'a BetfairClient) -> Self {
        Self { client }
    }

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

        let resp = self.client.call_api(session_token, &url, "GET", None).await?;

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
}
