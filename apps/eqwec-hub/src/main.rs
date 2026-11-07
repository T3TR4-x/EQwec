//! EQwec Hub developer-preview launcher.
//!
//! `--self-check` validates the profile without opening a window. Any other
//! invocation recovers an interrupted transaction and opens the launcher.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::Path;
use std::process::ExitCode;

use eqwec_hub::error::LauncherError;
use eqwec_hub::paths::{self, HostOs};
use eqwec_hub::state::{LaunchState, LauncherState, LockState, RecoveryState, RegistryState};
use eqwec_hub::{app, trust};
use eqwec_updater::{
    InstallLayout, ProfileLock, RecoveryOutcome, Registry, Timestamp, read_journal, recover,
};

fn main() -> ExitCode {
    if std::env::args()
        .skip(1)
        .any(|argument| argument == "--self-check")
    {
        return self_check();
    }
    match run_gui() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("eqwec-hub: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_gui() -> Result<(), LauncherError> {
    let data_dir = paths::data_dir()?;
    std::fs::create_dir_all(&data_dir).map_err(|error| LauncherError::Io {
        detail: format!("create data directory {}: {error}", data_dir.display()),
    })?;

    let lock_path = data_dir.join("profile.lock");
    let _lock = match ProfileLock::acquire(&lock_path).map_err(LauncherError::from) {
        Ok(lock) => Some(lock),
        Err(LauncherError::ProfileLocked) => None,
        Err(error) => return Err(error),
    };
    let lock_state = if _lock.is_some() {
        LockState::Acquired
    } else {
        LockState::Unavailable("another EQwec process holds the profile lock".to_owned())
    };
    let recovery = if _lock.is_some() {
        recover_profile(&data_dir)
    } else {
        RecoveryState::Clean
    };
    let mut state = LauncherState::load(data_dir, lock_state);
    state.recovery = recovery;

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 620.0])
            .with_min_inner_size([640.0, 420.0])
            .with_title("EQwec Hub"),
        ..Default::default()
    };
    eframe::run_native(
        "EQwec Hub",
        options,
        Box::new(|_cc| Ok(Box::new(app::HubApp::new(state)))),
    )
    .map_err(|error| LauncherError::Launch {
        detail: error.to_string(),
    })
}

fn recover_profile(data_dir: &Path) -> RecoveryState {
    let layout = match InstallLayout::new(data_dir) {
        Ok(layout) => layout,
        Err(error) => return RecoveryState::Failed(error.to_string()),
    };
    let mut registry = match Registry::load(&layout.registry_path()) {
        Ok(Some(registry)) => registry,
        Ok(None) => return RecoveryState::Clean,
        Err(error) => return RecoveryState::Failed(error.to_string()),
    };
    if let Err(error) = registry.validate_against_layout(&layout) {
        return RecoveryState::Failed(error.to_string());
    }
    let now = match Timestamp::now_utc() {
        Ok(now) => now,
        Err(error) => return RecoveryState::Failed(error.to_string()),
    };
    match recover(&layout, &mut registry, now) {
        Ok(RecoveryOutcome::NothingToRecover) => RecoveryState::Clean,
        Ok(RecoveryOutcome::Discarded { operation_id }) => {
            RecoveryState::Recovered(format!("discarded interrupted operation {operation_id}"))
        }
        Ok(RecoveryOutcome::Committed { operation_id }) => {
            RecoveryState::Recovered(format!("completed interrupted activation {operation_id}"))
        }
        Ok(RecoveryOutcome::RolledBack { operation_id }) => {
            RecoveryState::Recovered(format!("completed interrupted rollback {operation_id}"))
        }
        Ok(RecoveryOutcome::Uninstalled { operation_id }) => {
            RecoveryState::Recovered(format!("completed interrupted uninstall {operation_id}"))
        }
        Err(error) => RecoveryState::Failed(error.to_string()),
    }
}

fn self_check() -> ExitCode {
    match self_check_report() {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("self-check failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn self_check_report() -> Result<String, LauncherError> {
    let data_dir = paths::data_dir()?;
    let mut report = format!(
        "eqwec-hub {}\nplatform: {}/{}\ndata directory: {}\n",
        env!("CARGO_PKG_VERSION"),
        HostOs::current().as_str(),
        std::env::consts::ARCH,
        data_dir.display()
    );
    if !data_dir.exists() {
        report
            .push_str("lock: not acquired (profile not initialized)\nregistry: not initialized\n");
        return Ok(report);
    }

    let _lock =
        match ProfileLock::acquire(&data_dir.join("profile.lock")).map_err(LauncherError::from) {
            Ok(lock) => {
                report.push_str("lock: acquired\n");
                lock
            }
            Err(LauncherError::ProfileLocked) => {
                report.push_str("lock: held by another process\n");
                return Err(LauncherError::ProfileLocked);
            }
            Err(error) => return Err(error),
        };

    let state = LauncherState::load(data_dir.clone(), LockState::Acquired);
    match &state.registry {
        RegistryState::NotInitialized => report.push_str("registry: not initialized\n"),
        RegistryState::Ready(views) => {
            report.push_str(&format!("registry: ok ({} installations)\n", views.len()));
            for view in views {
                if let LaunchState::RepairRequired(reason) = &view.launch {
                    report.push_str(&format!(
                        "repair required: {} — {reason}\n",
                        view.product_id
                    ));
                }
            }
        }
        RegistryState::Error(error) => {
            return Err(LauncherError::Registry(error.clone()));
        }
    }

    let layout = InstallLayout::new(&data_dir)?;
    if let Some(unfinished) = read_journal(&layout.journal_path())?.unfinished_operation() {
        report.push_str(&format!(
            "journal: unfinished operation {} at {}\n",
            unfinished.operation_id().as_str(),
            unfinished.phase().as_str()
        ));
        return Err(LauncherError::Registry(
            "an interrupted operation requires recovery".to_owned(),
        ));
    }
    report.push_str("journal: clean\n");
    match trust::trust_store() {
        Ok(_) => report.push_str("trust: developer-preview key loaded\n"),
        Err(error) => report.push_str(&format!("trust: {error}\n")),
    }
    Ok(report)
}
