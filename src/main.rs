use anyhow::Result;
mod betfair;
mod environment;
use betfair::BetfairClient;
use environment::Environment;

use crate::betfair::BetfairDomain;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let env = Environment::from_env()?;

    let client = BetfairClient::new(&env.app_key, &env.cert_path, &env.key_path, env.insecure)?;
    let (status, session) = client.cert_login(&env.username, &env.password).await?;

    println!("HTTP {status}");
    println!("loginStatus={}", session.login_status);
    println!("sessionToken={}", session.session_token);

    let (ka_status, ka) = client.keep_alive(&session.session_token).await?;
    println!("keepAlive HTTP {ka_status}");
    println!("keepAlive status={}", ka.status);
    println!("keepAlive product={}", ka.product);
    println!("keepAlive token={}", ka.token);
    if let Some(err) = ka.error {
        println!("keepAlive error={err}");
    }

    let menu = client
        .navigation_menu(&ka.token, "en", BetfairDomain::Com)
        .await?;
    println!("{menu:#?}");

    Ok(())
}
