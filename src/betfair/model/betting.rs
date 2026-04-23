use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MarketBookRequest {
    #[serde(rename = "marketIds")]
    pub market_ids: Vec<String>,
    #[serde(rename = "priceProjection")]
    pub price_projection: PriceProjection,
}

#[derive(Debug, Serialize)]
pub struct PriceProjection {
    #[serde(rename = "priceData")]
    pub price_data: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MarketFilter {
    #[serde(rename = "marketIds")]
    pub market_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MarketCatalogueRequest {
    #[serde(rename = "filter")]
    pub filter: MarketFilter,
    #[serde(rename = "marketProjection")]
    pub market_projection: Vec<String>,
    #[serde(rename = "maxResults")]
    pub max_results: i32,
}
