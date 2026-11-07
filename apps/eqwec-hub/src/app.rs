//! Native window rendering.
//!
//! This is the only module that imports `eframe`/`egui`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use eframe::egui;

use crate::actions;
use crate::error::LauncherError;
use crate::launch;
use crate::paths::HostOs;
use crate::state::{LaunchState, LauncherState, LockState, RecoveryState, RegistryState};
use crate::worker::Worker;

const PREVIEW_AMBER: egui::Color32 = egui::Color32::from_rgb(200, 150, 40);
const SUCCESS_GREEN: egui::Color32 = egui::Color32::from_rgb(80, 170, 90);

struct Feedback {
    success: bool,
    message: String,
}

/// The launcher window.
pub struct HubApp {
    state: LauncherState,
    trust_status: String,
    bundle_path: String,
    feedback: Option<Feedback>,
    worker: Worker,
    confirm_uninstall: Option<String>,
    rollback_choice: BTreeMap<String, String>,
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
            bundle_path: String::new(),
            feedback: None,
            worker: Worker::new(),
            confirm_uninstall: None,
            rollback_choice: BTreeMap::new(),
        }
    }

    fn poll_worker(&mut self, ctx: &egui::Context) {
        if let Some(outcome) = self.worker.poll() {
            self.feedback = Some(Feedback {
                success: outcome.result.is_ok(),
                message: match outcome.result {
                    Ok(message) => message,
                    Err(error) => format!("{}: {error}", outcome.label),
                },
            });
            self.state.refresh();
            ctx.request_repaint();
        }
    }

    fn accept_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        let Some(path) = dropped.last() else {
            return;
        };
        let bundle = if path.is_dir() {
            path.clone()
        } else {
            path.parent()
                .map_or_else(|| path.clone(), Path::to_path_buf)
        };
        self.bundle_path = bundle.display().to_string();
        self.feedback = Some(Feedback {
            success: true,
            message: format!("Bundle selected: {}", bundle.display()),
        });
    }

    fn start_job<F>(&mut self, label: String, job: F)
    where
        F: FnOnce() -> Result<String, LauncherError> + Send + 'static,
    {
        self.feedback = None;
        self.worker.start(label, job);
    }

    fn install_clicked(&mut self) {
        let bundle = PathBuf::from(self.bundle_path.trim());
        if bundle.as_os_str().is_empty() {
            self.feedback = Some(Feedback {
                success: false,
                message: "Choose an absolute bundle folder first.".to_owned(),
            });
            return;
        }
        let data_dir = self.state.data_dir.clone();
        self.start_job("Install".to_owned(), move || {
            actions::install_bundle(&data_dir, &bundle)
        });
    }

    fn launch_clicked(&mut self, product_id: &str) {
        match actions::launch_product(&self.state.data_dir, product_id) {
            Ok(pid) => {
                self.feedback = Some(Feedback {
                    success: true,
                    message: format!("Launched {product_id} (pid {pid})"),
                });
            }
            Err(error) => {
                self.feedback = Some(Feedback {
                    success: false,
                    message: format!("Launch failed: {error}"),
                });
            }
        }
    }

    fn rollback_clicked(&mut self, product_id: &str, target: &str) {
        let data_dir = self.state.data_dir.clone();
        let product = product_id.to_owned();
        let version = target.to_owned();
        self.start_job(format!("Roll back {product} to {version}"), move || {
            actions::rollback(&data_dir, &product, &version)
        });
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("EQwec Hub").size(24.0).strong());
                ui.label(
                    egui::RichText::new(format!(
                        "{} · {}/{}",
                        env!("CARGO_PKG_VERSION"),
                        HostOs::current().as_str(),
                        std::env::consts::ARCH
                    ))
                    .weak(),
                );
            });
            ui.add_space(4.0);
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgba_unmultiplied(200, 150, 40, 24))
                .show(ui, |ui| {
                    ui.colored_label(PREVIEW_AMBER, "Developer preview — test signatures only");
                });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.weak("trust");
                ui.label(&self.trust_status);
                ui.separator();
                ui.weak("profile");
                match &self.state.lock {
                    LockState::Acquired => {
                        ui.label("locked by this process");
                    }
                    LockState::Unavailable(reason) => {
                        ui.colored_label(egui::Color32::LIGHT_RED, reason);
                    }
                };
                ui.separator();
                ui.weak("recovery");
                match &self.state.recovery {
                    RecoveryState::Clean => {
                        ui.label("clean");
                    }
                    RecoveryState::Recovered(message) => {
                        ui.colored_label(PREVIEW_AMBER, message);
                    }
                    RecoveryState::Failed(message) => {
                        ui.colored_label(egui::Color32::LIGHT_RED, message);
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.monospace(egui::RichText::new(self.state.data_dir.display().to_string()).weak());
                if ui.small_button("Show data folder").clicked()
                    && let Err(error) = launch::open_folder(&self.state.data_dir)
                {
                    self.feedback = Some(Feedback {
                        success: false,
                        message: error.to_string(),
                    });
                }
                if ui.small_button("Refresh").clicked() {
                    self.state.refresh();
                }
            });
            if !self.state.can_mutate() {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    "Read-only: install, update, rollback, and uninstall are disabled",
                );
            }
            ui.add_space(8.0);
        });
    }

    fn show_feedback(&mut self, ui: &mut egui::Ui) {
        let mut dismiss = false;
        if let Some(feedback) = &self.feedback {
            let color = if feedback.success {
                SUCCESS_GREEN
            } else {
                egui::Color32::LIGHT_RED
            };
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(if feedback.success { "Done" } else { "Failed" })
                            .strong()
                            .color(color),
                    );
                    ui.colored_label(color, &feedback.message);
                    if ui.small_button("Dismiss").clicked() {
                        dismiss = true;
                    }
                });
            });
            ui.add_space(4.0);
        }
        if dismiss {
            self.feedback = None;
        }
    }

    fn show_install_row(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Install or update").size(18.0).strong());
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Release bundle");
                ui.add(
                    egui::TextEdit::singleline(&mut self.bundle_path)
                        .hint_text("/absolute/path/to/bundle")
                        .desired_width(420.0),
                );
                let enabled = self.state.can_mutate()
                    && !self.worker.is_busy()
                    && !self.bundle_path.trim().is_empty();
                if ui
                    .add_enabled(enabled, egui::Button::new("Install / Update"))
                    .clicked()
                {
                    self.install_clicked();
                }
            });
            ui.label(
                egui::RichText::new(
                    "Exactly manifest.json and package.zip. Drag a bundle folder onto the window. \
                     Local bundles only; v0.1 has no download transport.",
                )
                .weak(),
            );
            if let Some(label) = self.worker.label() {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label(format!("{label}…"));
                    ui.label(
                        egui::RichText::new(
                            "Closing the window is safe: the journal recovers on the next start.",
                        )
                        .weak(),
                    );
                });
            }
        });
        ui.add_space(8.0);
    }

    fn show_products(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Installed products");
        });
        ui.separator();
        let registry = self.state.registry.clone();
        match registry {
            RegistryState::NotInitialized => {
                show_empty_state(ui);
            }
            RegistryState::Error(error) => {
                ui.colored_label(egui::Color32::LIGHT_RED, "The registry cannot be read.");
                ui.monospace(error);
                ui.label("Reload after fixing the profile. Mutating actions are disabled.");
            }
            RegistryState::Ready(views) if views.is_empty() => {
                show_empty_state(ui);
            }
            RegistryState::Ready(views) => {
                for view in views {
                    self.show_product_card(ui, &view);
                    ui.add_space(4.0);
                }
            }
        }
    }

    fn show_product_card(&mut self, ui: &mut egui::Ui, view: &crate::state::ProductView) {
        egui::Frame::group(ui.style())
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&view.product_id).size(18.0).strong());
                    ui.label(
                        egui::RichText::new(format!("{} · {}", view.version, view.channel)).weak(),
                    );
                });
                ui.add_space(4.0);
                ui.monospace(egui::RichText::new(format!("tree      {}", view.active_tree)).weak());
                ui.monospace(
                    egui::RichText::new(format!("installed {}", view.installed_at)).weak(),
                );
                if !view.retained.is_empty() {
                    ui.monospace(
                        egui::RichText::new(format!("retained  {}", view.retained.join(", ")))
                            .weak(),
                    );
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    match &view.launch {
                        LaunchState::Ready => {
                            ui.label(egui::RichText::new("Ready to launch").color(SUCCESS_GREEN));
                        }
                        LaunchState::RepairRequired(reason) => {
                            ui.colored_label(
                                egui::Color32::LIGHT_RED,
                                format!("Repair required: {reason}"),
                            );
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let enabled = self.state.can_mutate() && !self.worker.is_busy();
                        if ui
                            .add_enabled(enabled, egui::Button::new("Uninstall"))
                            .clicked()
                        {
                            self.confirm_uninstall = Some(view.product_id.clone());
                        }
                        if !view.retained.is_empty() {
                            let mut choice = self
                                .rollback_choice
                                .get(&view.product_id)
                                .cloned()
                                .unwrap_or_else(|| view.retained[0].clone());
                            egui::ComboBox::from_id_salt(format!("rollback-{}", view.product_id))
                                .selected_text(choice.clone())
                                .show_ui(ui, |ui| {
                                    for version in &view.retained {
                                        ui.selectable_value(&mut choice, version.clone(), version);
                                    }
                                });
                            self.rollback_choice
                                .insert(view.product_id.clone(), choice.clone());
                            if ui
                                .add_enabled(enabled, egui::Button::new("Roll back"))
                                .clicked()
                            {
                                self.rollback_clicked(&view.product_id, &choice);
                            }
                        }
                        let launch_enabled =
                            matches!(view.launch, LaunchState::Ready) && !self.worker.is_busy();
                        let accent = ui.visuals().selection.bg_fill;
                        if ui
                            .add_enabled(
                                launch_enabled,
                                egui::Button::new(egui::RichText::new("Launch").strong())
                                    .fill(accent),
                            )
                            .clicked()
                        {
                            self.launch_clicked(&view.product_id);
                        }
                    });
                });
            });
    }

    fn show_uninstall_modal(&mut self, ctx: &egui::Context) {
        let Some(product) = self.confirm_uninstall.clone() else {
            return;
        };
        let mut action = None;
        let response = egui::Modal::new(egui::Id::unique("uninstall-modal")).show(ctx, |ui| {
            ui.heading("Uninstall product?");
            ui.label(format!(
                "This removes {product} and its files from the profile."
            ));
            ui.label(
                "The operation is journaled; an interrupted uninstall resumes on the next start.",
            );
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    action = Some(false);
                }
                if ui.button("Uninstall").clicked() {
                    action = Some(true);
                }
            });
        });
        if response.should_close() {
            action = Some(false);
        }
        match action {
            Some(true) => {
                self.confirm_uninstall = None;
                let data_dir = self.state.data_dir.clone();
                let product_id = product;
                self.start_job(format!("Uninstall {product_id}"), move || {
                    actions::uninstall_product(&data_dir, &product_id)
                });
            }
            Some(false) => {
                self.confirm_uninstall = None;
            }
            None => {}
        }
    }
}

fn show_empty_state(ui: &mut egui::Ui) {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Nothing installed yet")
                .size(18.0)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Drop a release bundle folder onto this window, or paste its absolute path above.",
            )
            .weak(),
        );
        ui.label(
            egui::RichText::new("A bundle contains exactly manifest.json and package.zip.").weak(),
        );
    });
    ui.add_space(24.0);
}

impl eframe::App for HubApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_worker(ui.ctx());
        self.accept_dropped_files(ui.ctx());
        if self.worker.is_busy() {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
        self.show_header(ui);
        egui::CentralPanel::default_margins().show(ui, |ui| {
            self.show_feedback(ui);
            self.show_install_row(ui);
            ui.separator();
            self.show_products(ui);
        });
        self.show_uninstall_modal(ui.ctx());
    }
}
