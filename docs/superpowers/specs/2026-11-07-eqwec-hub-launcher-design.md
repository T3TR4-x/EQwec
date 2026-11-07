# EQwec Hub Launcher — Design

**Date:** 2026-11-07
**Revision:** 2, resolves the builder review of revision 1
**Status:** draft for approval
**Owner:** EQwec engineering
**Related:** ADR-0002, ADR-0007, ADR-0011, ADR-0012, ADR-0013 (new),
`docs/product-specs/eqwec-hub-schemas.md`

## 1. Intent

Give EQwec its first runnable, user-facing application: a native desktop
launcher that opens on macOS, shows what is installed, and can install, update,
roll back, uninstall, and launch a product from a locally supplied signed
release bundle.

The launcher is a real product surface, not a mockup. It uses the existing Hub
verification, staging, activation, registry, journal, and layout engines. It
does not invent a second installation model.

This is a **developer preview**. It is not a public release, has no production
signing identity, and does not download packages from the network.

## 2. Goals

- One Rust codebase (`eframe`/`egui`) that opens a native window.
- Window and all actions verified on macOS on Apple silicon.
- Windows and Linux build, pass unit/integration tests, pass `--self-check`,
  and produce installable artifacts in CI. Opening the GUI window on those
  platforms is `BLOCKED` until a desktop run is recorded; CI headless evidence
  is not treated as window proof.
- Read the real Hub registry and show installed products, versions, channels,
  active trees, and installation times.
- Install, update, roll back, uninstall, and launch from a local signed bundle.
- Recover an interrupted operation on startup before the window becomes
  interactive.
- Hold a crash-safe profile lock for the lifetime of the process.
- Preflight disk space before staging.
- Run a bounded health check on the verified staged binary before activation.
- Provide a headless `--self-check` mode for CI and packaging gates.
- Produce development artifacts: macOS DMG, Windows MSI, Linux `.deb` and
  `tar.zst`.

## 3. Non-goals

- Network download transport, update channels, mirrors, delta updates.
- Production code signing, notarization, Authenticode, or a release channel.
- Repair, dependency resolution, offline bundles, background updates.
- Cancellation of an in-flight staging operation. The operation runs to
  completion or the process is killed; journal recovery handles the latter.
- Plugin, browser, media, or creative-product functionality.
- Accessibility certification. egui/AccessKit is used; the audit stays open.
- Final brand identity or icon set.
- Automatic elevation. Everything the launcher installs is per-user.

## 4. Anti-goals

- A webview or Electron shell for application chrome.
- A second package format or signature scheme.
- Executing a package before signature, size, and SHA-256 verification.
- Deleting arbitrary paths during uninstall.
- Claiming a platform is supported because it compiles.
- A demo that bypasses verification and activation.

## 5. Constraints

- Rust 1.99.1, edition 2024, `#![forbid(unsafe_code)]`.
- UI stack `egui`/`eframe` per ADR-0002, wrapped so raw egui types stay inside
  `app.rs`.
- Manifest v1, trust store v1, package archive v1, registry v1, journal v1,
  layout v1. Uninstall extends the journal per ADR-0013.
- The launcher owns the profile lock, local bundle selection, launch policy,
  and UI. `eqwec-updater` owns verification, staging, activation, recovery,
  uninstall, the lock primitive, and disk preflight.
- No network access in v0.1.
- The developer-preview trust store contains a public key only. The matching
  seed is public test material in `tests/fixtures/eqwec-hub` and is never used
  for release signing.
- `eframe` default features only; no file-dialog dependency. A bundle folder is
  selected by drag and drop or by pasting an absolute path.

## 6. Approaches considered

### A. Rust + `egui`/`eframe`, one codebase (chosen)

One native window, one state model, one packaging pipeline. `eframe` supports
macOS, Windows, Linux, and AccessKit. The cost is a large dependency tree and
system libraries on Linux. It matches ADR-0002 and keeps the launcher in the
same language as the engine.

### B. Native UI per platform

Best platform integration, three codebases and three security reviews. Direct
conflict with the Rust-first architecture. Rejected for v0.1.

### C. CLI first, GUI later

Fastest path to install/update/rollback, but does not satisfy "the application
opens as a desktop app". Rejected as the primary deliverable; a `--self-check`
mode is still included for CI.

## 7. Product scope for v0.1

### 7.1 Window

Single window titled **EQwec Hub**:

- header: product name, launcher version `0.1.0-alpha.1`, platform,
  architecture, and a persistent banner
  `Developer preview — test signatures only`;
- status strip: data directory, profile-lock state, registry state
  (`not initialized`, `ok`, `error`), recovery result, and **Refresh**;
- product list: one card per registry record with product id, version,
  channel, active tree, installed-at time, and action buttons;
- empty state: short explanation, drag-and-drop target for a bundle folder,
  and a path field;
- error state: typed error text and **Reload**, never a panic;
- read-only mode: if the profile lock cannot be acquired, or recovery fails,
  every mutating action is disabled and the banner names the blocking error;
- action feedback: indeterminate progress while a worker runs, then success or
  a typed failure. The previous active version stays intact on failure.

### 7.2 Actions

- **Install** — bundle for a product that is not installed.
- **Update** — bundle with a newer version for an installed product.
- **Roll back** — choose a retained version on the product card.
- **Uninstall** — confirmation modal, then the journaled uninstall.
- **Launch** — see §7.5.
- **Show data folder** — opens the profile base with the platform opener.

Each mutation runs on one worker thread with an `mpsc` result channel. The UI
stays responsive, shows an indeterminate spinner, and disables conflicting
actions. Closing the window during an operation is allowed after a warning;
the journal and recovery make the next start safe. There is no cancel button.

Install, update, and rollback reuse `parse_and_verify`, `stage_package_from_path`,
`activate`, and `rollback`. Before activation the launcher runs the health check
in §8.2. The launcher generates an operation id and idempotency key locally.

### 7.3 Release bundle

A directory containing exactly `manifest.json` and `package.zip`.

The manifest is bounded by `MAX_MANIFEST_BYTES`; the package by
`MAX_PACKAGE_BYTES`. The launcher verifies the manifest against the embedded
developer-preview trust store and the current platform, architecture, channel,
and time before reading the package. The package is verified and staged by the
existing engine.

A demo bundle is produced by the non-published fixture crate:

```bash
cargo run -p eqwec-hub-fixture --bin make-demo-bundle -- target/demo-bundle
```

The generator uses the host target and the current time, and signs with the
public fixture seed. It is test infrastructure, not a release tool.

### 7.4 Developer-preview trust store

The launcher embeds `assets/dev-trust-store.json`, containing the public key of
the fixture seed, key id `dev-fixture-2026`, epoch 1, channel `stable`, and a
validity window covering the developer preview. The launcher labels every
build `Developer preview — test signatures only` and refuses to call the trust
store production. Production key custody is a separate release gate.

### 7.5 Launch convention

The entry point is `bin/<product_id>` on Unix and `bin/<product_id>.exe` on
Windows, declared as mode `0755` in the package. The fixture binary is renamed
from `bin/fixture` to `bin/eqwec-fixture` to exercise the convention.

Launch authorization is **pointer-first**:

1. Read `active.json` with `ActivePointer::load`. It is the only activation
   pointer.
2. Require a registry record whose `active_tree` and `package_sha256` match the
   pointer. A mismatch, a missing pointer, or an invalid pointer is a
   **repair required** state: the card disables Launch and shows the reason.
3. Resolve the tree under the product root with `InstallLayout::tree_path` and
   require it to stay inside the root.
4. Resolve the entry point, reject symlinks, require a regular file.
5. Spawn with `std::process::Command`, working directory = tree root, no shell,
   no interpolated arguments, inherited stdio.

## 8. Architecture

```text
apps/eqwec-hub/
  Cargo.toml
  src/
    main.rs          argument dispatch and eframe entry point
    app.rs           egui rendering and input handling
    state.rs         pure launcher state and view models (no egui)
    actions.rs       install/update/rollback/uninstall orchestration
    launch.rs        pointer-first resolution, health check, process spawn
    paths.rs         platform data directory resolution
    trust.rs         embedded developer-preview trust store
    worker.rs        one background job thread and result channel
  assets/dev-trust-store.json
  tests/launcher_actions.rs
```

- `state.rs` has no egui dependency and is testable with temp directories.
- `actions.rs` calls public `eqwec-updater` APIs only.
- `app.rs` is the only module importing `eframe`/`egui`.
- `paths.rs` resolves:
  - macOS: `$HOME/Library/Application Support/EQwec`
  - Windows: `%LOCALAPPDATA%\EQwec`
  - Linux: `$XDG_DATA_HOME/eqwec` or `$HOME/.local/share/eqwec`
  - `EQWEC_DATA_DIR` overrides the default for tests and portable runs; it must
    be absolute and normalized.
- Startup order: resolve data dir → acquire profile lock → run recovery until
  `NothingToRecover` → load registry → show window. The launcher does not create
  directories on startup. `ensure_base_dirs` runs on the first mutation.

### 8.1 New updater capabilities

**Profile lock** (`crates/eqwec-updater/src/lock.rs`):

```rust
pub struct ProfileLock { /* holds an exclusively locked file */ }
impl ProfileLock {
    pub fn acquire(path: &Path) -> Result<Self, UpdaterError>;
}
```

Uses `fs4` advisory locking on `<base>/profile.lock`. The OS releases the lock
when the process exits, so there is no stale-PID heuristic. A second process
fails with `UpdaterError::ProfileLocked`. The lock file contains the launcher
version and PID for diagnostics only.

**Disk preflight** (`crates/eqwec-updater/src/disk.rs`):

```rust
pub fn ensure_free_space(path: &Path, required_bytes: u64) -> Result<u64, UpdaterError>;
```

Uses `fs4::available_space`. The launcher requires
`package_size * 2 + 64 MiB` (staging plus the retained previous tree plus
margin). The arithmetic is unit-tested with injected values; the syscall path
is integration-tested.

**Journaled uninstall** (`crates/eqwec-updater/src/uninstall.rs`, ADR-0013):

```rust
pub fn uninstall(
    layout: &InstallLayout,
    registry: &mut Registry,
    product_id: &ProductId,
    operation_id: OperationId,
    idempotency_key: IdempotencyKey,
    updated_at: Timestamp,
) -> Result<UninstallOutcome, UpdaterError>;
```

Journal phases added: `uninstall-intent` and `uninstalled` (terminal).
Order, matching schema §4.2:

1. Verify the record, `install_root == layout.product_root(product_id)`, and
   strict containment under the layout base.
2. Append `uninstall-intent` with the current channel, version, and package
   hash.
3. Remove the product root recursively, refusing symlinks. Deletion is
   idempotent.
4. Remove the registry record and write the registry atomically.
5. Append `uninstalled`.

Recovery maps `uninstall-intent` to the same idempotent sequence. A failure at
step 3 appends `failed` and keeps the record, so the UI shows a retryable
error. A failure at step 4 leaves a record whose tree is gone; recovery retries
step 4. The launcher is the only writer and holds the profile lock, so no reader
observes the intermediate state.

### 8.2 Health check

Before activation, after staging:

- resolve the staged entry point with the same containment and symlink rules;
- spawn it with a 10-second timeout, working directory = staged tree root;
- require exit code 0; on timeout kill the child and fail;
- a failed health check aborts before `activate`, leaves the active tree
  unchanged, and cleans staging through the existing failure path.

The staged tree is already signature- and hash-verified, so this does not
execute unverified bytes. The fixture prints its generation and reads its
payload, so a broken tree fails the check.

## 9. Packaging

| Platform | Artifact | Tooling | Signing in v0.1 |
|---|---|---|---|
| macOS | `EQwec Hub.app` in `EQwec-Hub-<version>-macos-<arch>.dmg` | `cargo build --release`, `codesign`, `hdiutil` | ad-hoc `codesign -s -` |
| Windows | `EQwec-Hub-<version>-windows-x86_64.msi` | WiX v4+ as a `dotnet tool`, hand-written `.wxs` | unsigned |
| Linux | `eqwec-hub_<version>_amd64.deb`, `eqwec-hub-<version>-linux-x86_64.tar.zst` | staged tree + `dpkg-deb --build`; `tar` + `zstd` | unsigned; minisign later |

- macOS: `Contents/Info.plist`, `Contents/MacOS/eqwec-hub`, placeholder icon.
  The app is run from `/Applications` or a temp copy, never from the DMG.
- Windows: per-user MSI installed to `%LOCALAPPDATA%\Programs\EQwec Hub`,
  **outside** the data base `%LOCALAPPDATA%\EQwec`. Start-menu shortcut, no
  elevation, standard WiX upgrade and uninstall entries.
- Linux `.deb`: system-wide `/usr/bin/eqwec-hub` plus
  `/usr/share/applications/eqwec-hub.desktop`, installed by the OS package
  manager with its normal root consent. The application itself never elevates.
  The `tar.zst` is the per-user portable alternative. The data directory stays
  per-user in both cases.
- A GitHub Actions workflow `package.yml` runs on `workflow_dispatch` and `v*`
  tags. It builds release binaries, runs `--self-check`, packages, unpacks or
  installs the artifact, runs `--self-check` again, and uploads it with 14-day
  retention. Packaging is not part of the per-push gate.

## 10. Security and integrity

- No network access in v0.1.
- Manifest signature, size, hash, channel, platform, architecture, time, epoch,
  revocation, and rollback sequence are verified before staging.
- Registry reads are bounded by `MAX_REGISTRY_BYTES`; corrupt input is an error,
  never silently repaired.
- The profile lock is OS-level and released on process death.
- Disk space is preflighted before staging.
- The staged binary is health-checked before activation.
- Uninstall validates containment and refuses symlinked roots; the registry is
  the source of truth and is updated only after file removal.
- Launch is pointer-first and validates containment, symlinks, and regular-file
  type. No shell, no argument interpolation.
- The developer-preview seed is public and is not a release key.
- No secrets, signed URLs, or full package contents in logs or diagnostics.
- `#![forbid(unsafe_code)]` on the launcher; `fs4` is the only new dependency
  with platform locking code.

## 11. Testing

- Unit: data-directory resolution per platform with injected environment,
  registry state mapping, product view models, launch path resolution,
  containment and symlink rejection, error rendering, disk math, bundle layout
  validation, self-check exit codes.
- Updater unit/integration: profile lock mutual exclusion and release on drop,
  disk preflight, uninstall ordering, containment, idempotent retry, and
  fault-injected recovery at `uninstall-intent` and before the registry write.
- Launcher integration (no egui): install a fixture bundle into a temp layout,
  list it, health-check, update to v2, roll back to v1, launch both, uninstall,
  verify registry and filesystem state; tampered manifest, tampered package,
  wrong target, missing pointer, and missing executable each fail closed.
- `--self-check` on all three CI platforms; nonzero on a corrupt registry and on
  a committed record with a missing or inconsistent pointer.
- Manual GUI smoke test on macOS: empty state, install, card appears, Launch,
  Update, Rollback, Uninstall.
- Packaging smoke test in CI: unpack/install the artifact and run
  `--self-check`.
- Existing gates: fmt, clippy, build, test, audit, deny, docs, skills.

## 12. Acceptance criteria

1. `cargo run -p eqwec-hub` opens a window on macOS.
2. `cargo run -p eqwec-hub -- --self-check` prints version, platform, data
   directory, lock state, and registry state, and exits 0 on a healthy or empty
   profile; exits 1 on a corrupt or inconsistent profile.
3. Install, update, rollback, launch, and uninstall work against the fixture
   bundle in an integration test.
4. The macOS DMG opens and the app launches (manual, recorded).
5. Windows MSI and Linux `.deb`/`tar.zst` are produced by CI and pass
   `--self-check` on their native runners. GUI window verification on Windows
   and Linux stays `BLOCKED` until a desktop run is recorded.
6. Corrupt registry, tampered manifest, tampered package, wrong target, missing
   pointer, and missing executable produce typed errors and leave the previous
   installation intact.
7. No network, no unsafe, no secrets, no production signing claim.
8. All repository gates pass and documentation/skill status is reconciled.

## 13. Implementation order

1. Updater additions: `fs4` admission, profile lock, disk preflight, uninstall
   with ADR-0013 and recovery tests.
2. Launcher crate: paths, state, `--self-check`, worker, window shell with
   empty/error/read-only states.
3. Actions: install/update/rollback with health check and launch.
4. Fixture entry-point rename and `make-demo-bundle`.
5. Packaging scripts and `package.yml`.
6. Documentation, dependency map, legal review, gate reports.
7. Manual macOS smoke test and CI packaging verification.

## 14. Risks and open questions

- **Linux GUI build dependencies.** `eframe` needs X11/Wayland/GL development
  libraries. CI installs them explicitly; if the GUI build is unstable there,
  the Linux package job is marked `BLOCKED` rather than silently skipped.
- **WiX v4 tool availability.** Installed as a `dotnet tool` in the workflow.
  If it fails, the MSI job is `BLOCKED`; a zip fallback is not an installer and
  does not satisfy acceptance.
- **Unsigned artifacts.** Gatekeeper and SmartScreen warn. Expected for the
  developer preview and documented in the artifact README.
- **No download transport.** "Install" means a local bundle; the UI says so and
  shows no fake update check.
- **Uninstall failure.** Registry-first after file removal means a failed
  cleanup keeps a retryable record; a failed registry write is repaired on next
  startup.
- **Accessibility and IME.** egui/AccessKit is used; a dedicated audit is
  required before a public release.
- **App identity.** `EQwec Hub` and `com.eqwec.hub` are provisional.
