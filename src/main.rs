use anyhow::{Context, Result};
use eframe::egui;
use std::fs;
use std::sync::mpsc;
use std::thread;

mod betfair;
mod environment;

use crate::betfair::{BetfairClient, BetfairDomain, NavigationNode};
use crate::environment::Environment;

struct MenuApp {
    rx: Option<mpsc::Receiver<Result<NavigationNode>>>,
    menu: Option<NavigationNode>,
    error: Option<String>,
}

impl MenuApp {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = (|| -> Result<NavigationNode> {
                dotenvy::dotenv().ok();
                let env = Environment::from_env()?;

                let rt = tokio::runtime::Runtime::new().context("creating tokio runtime")?;
                rt.block_on(async move {
                    let client =
                        BetfairClient::new(&env.app_key, &env.cert_path, &env.key_path, env.insecure)?;
                    let (_status, session) = client.cert_login(&env.username, &env.password).await?;
                    let (_ka_status, ka) = client.keep_alive(&session.session_token).await?;
                    let (_menu_status, menu) =
                        client.navigation_menu(&ka.token, "en", BetfairDomain::Com).await?;

                    let menu_json = serde_json::to_string_pretty(&menu)?;
                    fs::write("output.json", menu_json)?;

                    Ok(menu)
                })
            })();

            let _ = tx.send(result);
        });

        Self {
            rx: Some(rx),
            menu: None,
            error: None,
        }
    }
}

fn menu_ui(ui: &mut egui::Ui, node: &NavigationNode) {
    if node.children.is_empty() {
        ui.label(&node.name);
        return;
    }

    egui::CollapsingHeader::new(format!("{} ({})", node.name, node.id.0))
        .default_open(false)
        .show(ui, |ui| {
            for child in &node.children {
                menu_ui(ui, child);
            }
        });
}

impl eframe::App for MenuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.menu.is_none() && self.error.is_none() {
            if let Some(rx) = &self.rx {
                match rx.try_recv() {
                    Ok(Ok(menu)) => {
                        self.menu = Some(menu);
                        self.rx = None;
                    }
                    Ok(Err(e)) => {
                        self.error = Some(format!("{e:#}"));
                        self.rx = None;
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.error = Some("background fetch thread disconnected".to_string());
                        self.rx = None;
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.heading("Betfair Navigation Menu");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::Resize::default()
                    .resizable(true)
                    .default_width(360.0)
                    .min_width(180.0)
                    .show(ui, |ui| {
                        ui.set_min_height(ui.available_height());

                        if let Some(err) = &self.error {
                            ui.colored_label(egui::Color32::RED, err);
                            ui.separator();
                            ui.label("Fix .env values (BETFAIR_USERNAME/BETFAIR_PASSWORD/BETFAIR_APP_KEY, etc) then restart.");
                            return;
                        }

                        if let Some(menu) = &self.menu {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                menu_ui(ui, menu);
                            });
                        } else {
                            ui.label("Fetching menu from Betfair...");
                        }
                    });

                ui.separator();

                ui.vertical(|ui| {
                    ui.set_min_height(ui.available_height());
                    if self.menu.is_some() {
                        ui.label("Saved to output.json");
                    } else if self.error.is_none() {
                        ui.label("Loading...");
                    }
                });
            });
        });

        ctx.request_repaint();
    }
}

fn main() -> Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Betfair Menu",
        options,
        Box::new(|_cc| Box::new(MenuApp::new())),
    )
    .map_err(|e| anyhow::anyhow!("starting eframe: {e}"))?;

    Ok(())
}
