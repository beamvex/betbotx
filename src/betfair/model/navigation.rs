use serde::{Deserialize, Serialize};
use serde_json::Value;

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
