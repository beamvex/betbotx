use anyhow::Result;
mod betfair;
mod environment;
use betfair::BetfairClient;
use environment::Environment;
use std::env;

use crate::betfair::BetfairDomain;
use crate::betfair::NavigationNode;

fn print_menu_names(node: &NavigationNode, depth: usize, max_depth: usize, max_children: usize) {
    if depth > max_depth {
        return;
    }

    let indent = "  ".repeat(depth);
    println!("{indent}{}", node.name);
    if depth == max_depth {
        return;
    }
    for child in node.children.iter().take(max_children) {
        print_menu_names(child, depth + 1, max_depth, max_children);
    }
}

fn usage() {
    eprintln!(
        "Usage:\n  betbotapix fetch [output.json]\n  betbotapix load [output.json]"
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let mut args = env::args().skip(1);
    let mode = args.next();
    let path = args.next().unwrap_or_else(|| "output.json".to_string());

    match mode.as_deref() {
        Some("fetch") | None => {
            let env = Environment::from_env()?;

            let client =
                BetfairClient::new(&env.app_key, &env.cert_path, &env.key_path, env.insecure)?;
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
            print_menu_names(&menu.1, 0, 4, 10);

            let json = serde_json::to_string_pretty(&menu.1)?;
            std::fs::write(&path, json)?;
            println!("Wrote {path}");
        }
        Some("account") => {
            let env = Environment::from_env()?;

            let client =
                BetfairClient::new(&env.app_key, &env.cert_path, &env.key_path, env.insecure)?;
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

            let account_details = client
                .get_account_details(&ka.token, "en", BetfairDomain::Com)
                .await?;
            println!("Account details: {:#?}", account_details.text().await?);
        }
        Some("load") => {
            let bytes = std::fs::read(&path)?;
            let menu: NavigationNode = serde_json::from_slice(&bytes)?;

            let mut horse_racing_node: Option<&NavigationNode> = None;

            for child in menu.children.iter().take(100) {
                //println!("{}", child.name);
                if child.name == "Horse Racing" {
                    println!("Found Horse Racing!");
                    horse_racing_node = Some(child);

                    for grandchild in child.children.iter().take(100) {
                        if grandchild.name == "GB" {
                            println!("GB Found Today!");

                            print_menu_names(grandchild, 0, 3, 100);
                            
                            let json = serde_json::to_string_pretty(&grandchild)?;
                            std::fs::write("gb_today.json", json)?;
                            println!("Wrote gb_today.json");
                            
                            break;
                        }
                    }



                    break;
                }
            }

            
        }
        Some("-h") | Some("--help") => {
            usage();
        }
        Some(other) => {
            eprintln!("Unknown mode: {other}\n");
            usage();
        }
    }

    Ok(())
}
