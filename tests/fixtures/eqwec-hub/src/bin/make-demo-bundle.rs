//! Writes a developer-preview release bundle for the EQwec Hub launcher.
//!
//! Usage:
//! `cargo run -p eqwec-hub-fixture --bin make-demo-bundle -- <outdir> [version] [sequence]`
//!
//! The bundle is signed with the public fixture seed, so only the launcher's
//! developer-preview trust store accepts it. The product executable is
//! compiled with `rustc`, which means this generator requires the Rust
//! toolchain that built the workspace.

#![forbid(unsafe_code)]

use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use eqwec_hub_fixture::{BundleSpec, DEV_KEY_ID, PRODUCT_ID, compile_fixture_probe, write_bundle};
use eqwec_updater::Timestamp;

const USAGE: &str = "usage: make-demo-bundle <outdir> [version] [sequence]";
const DEFAULT_VERSION: &str = "1.0.0";
const DEFAULT_SEQUENCE: u64 = 1;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("make-demo-bundle: {error}");
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let outdir = arguments.next().ok_or(USAGE)?;
    let version = match arguments.next() {
        Some(value) => value
            .into_string()
            .map_err(|_| "version must be valid UTF-8")?,
        None => DEFAULT_VERSION.to_owned(),
    };
    let sequence = match arguments.next() {
        Some(value) => value
            .to_str()
            .ok_or("sequence must be valid UTF-8")?
            .parse::<u64>()?,
        None => DEFAULT_SEQUENCE,
    };
    if sequence == 0 {
        return Err("sequence must be at least 1".into());
    }
    if arguments.next().is_some() {
        return Err(USAGE.into());
    }

    let outdir = std::path::absolute(PathBuf::from(outdir))?;
    prepare_outdir(&outdir)?;

    let build_dir = std::env::temp_dir().join(format!(
        "eqwec-demo-build-{}-{sequence}",
        std::process::id()
    ));
    let generation = u8::try_from(sequence).unwrap_or(1);
    let probe = compile_fixture_probe(&build_dir, generation)?;
    let binary = fs::read(&probe)?;
    let _ = fs::remove_dir_all(&build_dir);

    let now = Timestamp::now_utc()?;
    let issued = now
        .checked_add_seconds(-3600)
        .ok_or("cannot compute the issue time")?
        .to_string();
    let expires = now
        .checked_add_seconds(7 * 24 * 60 * 60)
        .ok_or("cannot compute the expiry time")?
        .to_string();
    let spec = BundleSpec {
        version: &version,
        sequence,
        key_id: DEV_KEY_ID,
        issued_at: &issued,
        expires_at: &expires,
    };
    write_bundle(&outdir, &binary, &spec)?;

    println!("Wrote {PRODUCT_ID} {version} (metadata sequence {sequence})");
    println!("  {}", outdir.join("manifest.json").display());
    println!("  {}", outdir.join("package.zip").display());
    println!();
    println!("Install it with: cargo run -p eqwec-hub");
    Ok(())
}

/// Rejects a directory that already holds anything but bundle files.
///
/// Re-running the generator into the same directory overwrites the two
/// bundle files. Any other entry would make the launcher reject the bundle,
/// so the generator fails early and names the offending entry.
fn prepare_outdir(outdir: &Path) -> Result<(), Box<dyn Error>> {
    match fs::read_dir(outdir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name != "manifest.json" && name != "package.zip" {
                    return Err(format!(
                        "{} contains {name}; a bundle may contain only manifest.json and package.zip",
                        outdir.display()
                    )
                    .into());
                }
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}
