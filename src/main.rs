use anyhow::Result;
mod betfair;
mod environment;
use betfair::BetfairClient;
use environment::Environment;
use std::env;

use chrono::{DateTime, Local};

use crate::betfair::BetfairDomain;
use crate::betfair::BetfairAccountClient;
use crate::betfair::BetfairBettingClient;
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

            let account_client = BetfairAccountClient::new(&client);

            let account_details = account_client
                .get_account_details(&ka.token, "en", BetfairDomain::Com)
                .await?;
            println!("Account details: {:#?}", account_details.text().await?);

            let account_funds = account_client
                .get_account_funds(&ka.token, "en", BetfairDomain::Com)
                .await?;
            println!("Account funds: {:#?}", account_funds.text().await?);
        }
        Some("market") => {
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

            let _account_client = BetfairAccountClient::new(&client);

            let betting_client = BetfairBettingClient::new(&client);

            let list_market_books = betting_client
                .list_market_books(&ka.token, "en", BetfairDomain::Com)
                .await?;
            println!("List market books: {:#?}", list_market_books.json::<serde_json::Value>().await?);

            let list_market_catalogue = betting_client
                .list_market_catalogue(&ka.token, "en", BetfairDomain::Com)
                .await?;
            println!("List market catalogue: {:#?}", list_market_catalogue.json::<serde_json::Value>().await?);


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

                            //print_menu_names(grandchild, 0, 3, 100);
                            
                            let json = serde_json::to_string_pretty(&grandchild)?;
                            std::fs::write("gb_today.json", json)?;
                            println!("Wrote gb_today.json");
                            
                            process_meetings(&grandchild);

                            
                        } else if grandchild.name == "IRE" {
                            println!("IRE Found Today!");

                            //print_menu_names(grandchild, 0, 3, 100);
                            
                            let json = serde_json::to_string_pretty(&grandchild)?;
                            std::fs::write("ie_today.json", json)?;
                            println!("Wrote ie_today.json");
                            
                            process_meetings(&grandchild);

                            
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

fn process_meetings(grandchild: &NavigationNode) -> Result<(), Box<dyn std::error::Error>> {
    
    for child in grandchild.children.iter() {
        println!("{}", child.name);

        process_races(child)?;
    }


    Ok(())
}

fn process_races(meeting: &NavigationNode) -> Result<(), Box<dyn std::error::Error>> {
    let today = Local::now().date_naive();
    
    for race in meeting.children.iter() {
        if race.market_type == Some("WIN".to_string()) {

            let start_time = match race.market_start_time.as_deref() {
                Some(s) => s,
                None => continue,
            };

            let utc: DateTime<_> = DateTime::parse_from_rfc3339(start_time)?;
            let local = utc.with_timezone(&Local);

            if local.date_naive() != today {
                continue;
            }

            println!("{} : {}", local.format("%H:%M"), race.id.0);
        }
    }
    
    Ok(())
}
