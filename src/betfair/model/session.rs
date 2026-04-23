use serde::Deserialize;

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
