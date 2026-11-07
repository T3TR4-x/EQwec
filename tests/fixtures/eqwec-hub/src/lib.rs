//! Non-production fixture bundles for EQwec Hub tests and demos.
//!
//! The fixture seed is public test material. It must never be used for a
//! release. The helpers build real ZIP packages, sign manifests with the
//! fixture key, and compile tiny probe executables with the active Rust
//! toolchain.

use std::error::Error;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use ed25519_dalek::{Signer, SigningKey};
use eqwec_core::{Architecture, Platform};
use eqwec_security::Sha256Digest;
use eqwec_security::hex::encode_lower_hex;
use eqwec_updater::Manifest;
use serde_json::{Value, json};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, System, ZipWriter};

/// Public non-production signing seed.
pub const PUBLIC_FIXTURE_SEED: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];

/// Product identifier used by the fixture.
pub const PRODUCT_ID: &str = "eqwec-fixture";

/// Key identifier embedded in the launcher's developer-preview trust store.
pub const DEV_KEY_ID: &str = "dev-fixture-2026";

/// Result type for fixture helpers.
pub type FixtureResult<T> = Result<T, Box<dyn Error>>;

/// One manifest to sign.
#[derive(Clone, Copy, Debug)]
pub struct BundleSpec<'a> {
    /// Product version.
    pub version: &'a str,
    /// Monotonic metadata sequence.
    pub sequence: u64,
    /// Signing key identifier.
    pub key_id: &'a str,
    /// Manifest issue time, `YYYY-MM-DDTHH:MM:SSZ`.
    pub issued_at: &'a str,
    /// Manifest expiry time, `YYYY-MM-DDTHH:MM:SSZ`.
    pub expires_at: &'a str,
}

/// Builds a ZIP package containing `bin/eqwec-fixture` and a payload.
#[must_use]
pub fn build_package(binary: &[u8], version: &str) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    write_entry(&mut writer, &entry_path(), binary, 0o100_755);
    write_entry(
        &mut writer,
        "share/fixture.txt",
        format!("payload {version}\n").as_bytes(),
        0o100_644,
    );
    writer
        .finish()
        .expect("finish fixture package")
        .into_inner()
}

/// Writes `manifest.json` and `package.zip` into `dir`.
///
/// # Errors
///
/// Returns an error when the directory cannot be created, the package cannot
/// be serialized, or the manifest cannot be signed.
pub fn write_bundle(dir: &Path, binary: &[u8], spec: &BundleSpec<'_>) -> FixtureResult<()> {
    fs::create_dir_all(dir)?;
    let package = build_package(binary, spec.version);
    let manifest = manifest_bytes(&package, spec)?;
    fs::write(dir.join("package.zip"), package)?;
    fs::write(dir.join("manifest.json"), manifest)?;
    Ok(())
}

/// Compiles a fixture probe that prints its generation and payload.
///
/// The probe reads `share/fixture.txt` relative to its own tree root, so a
/// missing payload exits nonzero.
///
/// # Errors
///
/// Returns an error when the source cannot be written or `rustc` fails.
pub fn compile_fixture_probe(dir: &Path, generation: u8) -> FixtureResult<PathBuf> {
    let source = format!(
        "use std::path::Path;\n\
         fn main() {{\n\
         let executable = std::env::current_exe().expect(\"probe path\");\n\
         let root = executable.parent().and_then(Path::parent).expect(\"tree root\");\n\
         let payload = std::fs::read_to_string(root.join(\"share/fixture.txt\"))\n\
         .expect(\"read share/fixture.txt\");\n\
         println!(\"eqwec-fixture v{generation}\");\n\
         print!(\"{{payload}}\");\n\
         }}\n"
    );
    compile_probe(dir, &format!("probe-{generation}"), &source)
}

/// Compiles a probe that sleeps for sixty seconds.
///
/// # Errors
///
/// Returns an error when the source cannot be written or `rustc` fails.
pub fn compile_sleeping_probe(dir: &Path) -> FixtureResult<PathBuf> {
    let source = "fn main() { std::thread::sleep(std::time::Duration::from_secs(60)); }\n";
    compile_probe(dir, "sleeping-probe", source)
}

/// Compiles a probe that exits with status 1.
///
/// # Errors
///
/// Returns an error when the source cannot be written or `rustc` fails.
pub fn compile_failing_probe(dir: &Path) -> FixtureResult<PathBuf> {
    let source = "fn main() { std::process::exit(1); }\n";
    compile_probe(dir, "failing-probe", source)
}

fn compile_probe(dir: &Path, name: &str, source: &str) -> FixtureResult<PathBuf> {
    fs::create_dir_all(dir)?;
    let source_path = dir.join(format!("{name}.rs"));
    fs::write(&source_path, source)?;
    let executable = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc)
        .arg("--edition=2024")
        .arg("-O")
        .arg("-o")
        .arg(&executable)
        .arg(&source_path)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "rustc failed for {}: {}",
            source_path.display(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(executable)
}

fn manifest_bytes(package: &[u8], spec: &BundleSpec<'_>) -> FixtureResult<Vec<u8>> {
    let placeholder = "00".repeat(64);
    let unsigned = serde_json::to_vec(&manifest_value(package, spec, &placeholder))?;
    let parsed = Manifest::parse(&unsigned)?;
    let signature = signing_key().sign(&parsed.canonical_payload());
    let signed = manifest_value(package, spec, &encode_lower_hex(&signature.to_bytes()));
    Ok(serde_json::to_vec(&signed)?)
}

fn manifest_value(package: &[u8], spec: &BundleSpec<'_>, signature: &str) -> Value {
    json!({
        "schema_version": 1,
        "product_id": PRODUCT_ID,
        "product_version": spec.version,
        "channel": "stable",
        "target": {
            "os": platform().as_str(),
            "architecture": architecture().as_str(),
            "minimum_platform": "1.0"
        },
        "package": {
            "source": format!(
                "https://packages.eqwec.example/fixture/{version}/eqwec-fixture-{version}.zip",
                version = spec.version
            ),
            "size": package.len(),
            "hashes": [{ "algorithm": "sha256", "value": Sha256Digest::hash(package).to_hex() }]
        },
        "dependencies": [],
        "metadata_sequence": spec.sequence,
        "issued_at": spec.issued_at,
        "expires_at": spec.expires_at,
        "release_notes": format!("Fixture release {}.", spec.version),
        "signature": {
            "key_id": spec.key_id,
            "algorithm": "ed25519",
            "signature": signature,
            "signing_time": spec.issued_at,
            "key_epoch": 1
        }
    })
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&PUBLIC_FIXTURE_SEED)
}

fn platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else {
        Platform::Linux
    }
}

fn architecture() -> Architecture {
    if cfg!(target_arch = "aarch64") {
        Architecture::Aarch64
    } else {
        Architecture::X86_64
    }
}

fn entry_path() -> String {
    if cfg!(windows) {
        format!("bin/{PRODUCT_ID}.exe")
    } else {
        format!("bin/{PRODUCT_ID}")
    }
}

fn write_entry(writer: &mut ZipWriter<Cursor<Vec<u8>>>, name: &str, data: &[u8], mode: u32) {
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default())
        .external_attributes(mode << 16)
        .system(System::Unix)
        .name(name.as_bytes());
    writer
        .start_file_with_options(options)
        .expect("start fixture entry");
    writer.write_all(data).expect("write fixture entry");
}
