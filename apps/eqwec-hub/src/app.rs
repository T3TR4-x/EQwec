//! Native window rendering.
//!
//! This is the only module that imports `eframe`/`egui`.

use eframe::egui;

use crate::paths::HostOs;
use crate::state::{LaunchState, LauncherState, LockState, RegistryState};

/// The launcher window.
pub struct HubApp {
    state: LauncherState,
    trust_status: String,
}

impl HubApp {
    /// Creates the window from loaded state.
    #[must_use]
    pub fn new(state: LauncherState) -> Self {
        let trust_status = match crate::trust::trust_store() {
            Ok(_) => "developer-preview key loaded".to_owned(),
            Err(error) => format!("unavailable: {error}"),
        };
        Self {
            state,
            trust_status,
        }
    }
}

impl eframe::App for HubApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading("EQwec Hub");
                ui.label(format!(
                    "{} · {}",
                    env!("CARGO_PKG_VERSION"),
                    HostOs::current().as_str()
                ));
            });
            ui.colored_label(
                egui::Color32::from_rgb(200, 150, 40),
                "Developer preview — test signatures only",
            );
            ui.label(format!("trust: {}", self.trust_status));
            if !self.state.can_mutate() {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    "Read-only: mutating actions are disabled",
                );
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.monospace(self.state.data_dir.display().to_string());
                ui.separator();
                match &self.state.lock {
                    LockState::Acquired => {
                        ui.label("profile: locked by this process");
                    }
                    LockState::Unavailable(reason) => {
                        ui.colored_label(egui::Color32::LIGHT_RED, format!("profile: {reason}"));
                    }
                };
            });
            ui.add_space(6.0);
        });

        egui::CentralPanel::default_margins().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Installed products");
                if ui.button("Refresh").clicked() {
                    self.state.refresh();
                }
            });
            ui.separator();
            match &self.state.registry {
                RegistryState::NotInitialized => {
                    ui.label("No profile is initialized yet.");
                    ui.label("Install a signed release bundle to create one.");
                }
                RegistryState::Error(error) => {
                    ui.colored_label(egui::Color32::LIGHT_RED, "The registry cannot be read.");
                    ui.monospace(error);
                    ui.label("Reload after fixing the profile. Mutating actions are disabled.");
                }
                RegistryState::Ready(views) if views.is_empty() => {
                    ui.label("No products are installed.");
                }
                RegistryState::Ready(views) => {
                    for view in views {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong(&view.product_id);
                                ui.label(format!("{} · {}", view.version, view.channel));
                            });
                            ui.monospace(format!("tree: {}", view.active_tree));
                            ui.monospace(format!("installed: {}", view.installed_at));
                            if !view.retained.is_empty() {
                                ui.monospace(format!("retained: {}", view.retained.join(", ")));
                            }
                            match &view.launch {
                                LaunchState::Ready => {
                                    ui.label("Ready to launch.");
                                }
                                LaunchState::RepairRequired(reason) => {
                                    ui.colored_label(
                                        egui::Color32::LIGHT_RED,
                                        format!("Repair required: {reason}"),
                                    );
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }
                }
            }
        });
    }
}
