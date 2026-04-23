use anyhow::Result;

use super::client::BetfairClient;
use super::model::{
    BetfairDomain, MarketBookRequest, MarketCatalogueRequest, MarketFilter, PriceProjection,
};

pub struct BetfairBettingClient<'a> {
    client: &'a BetfairClient,
}

impl<'a> BetfairBettingClient<'a> {
    pub fn new(client: &'a BetfairClient) -> Self {
        Self { client }
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

        let resp = self
            .client
            .call_api(session_token, &url, "POST", Some(body))
            .await?;

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
            market_projection: vec![
                "MARKET_START_TIME".to_string(),
                "RUNNER_DESCRIPTION".to_string(),
                "EVENT".to_string(),
                "COMPETITION".to_string(),
                "EVENT_TYPE".to_string(),
                "MARKET_DESCRIPTION".to_string(),
            ],
            max_results: 100,
        };

        let body = serde_json::to_string(&request)?;

        let resp = self
            .client
            .call_api(session_token, &url, "POST", Some(body))
            .await?;

        Ok(resp)
    }
}
