use anyhow::Result;
mod betfair;
mod environment;
use betfair::BetfairClient;
use environment::Environment;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let env = Environment::from_env()?;

    let client = BetfairClient::new(&env.app_key, &env.cert_path, &env.key_path, env.insecure)?;
    let (status, session) = client.cert_login(&env.username, &env.password).await?;

    println!("HTTP {status}");
    println!("loginStatus={}", session.login_status);
    println!("sessionToken={}", session.session_token);

    Ok(())
}
