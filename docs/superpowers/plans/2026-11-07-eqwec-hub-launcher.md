# EQwec Hub Launcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a runnable EQwec Hub desktop launcher that installs, updates, rolls back, uninstalls, and launches a signed fixture bundle on macOS, and builds/packages for Windows and Linux.

**Architecture:** New `apps/eqwec-hub` crate on `eframe`/`egui`; all installation logic stays in `eqwec-updater`. Three updater additions (profile lock, disk preflight, journaled uninstall) land first with tests, then the launcher, then packaging.

**Tech Stack:** Rust 1.99.1, edition 2024, eframe 0.37, fs4 1.1, existing `eqwec-updater`/`eqwec-security`/`eqwec-core`.

**Spec:** `docs/superpowers/specs/2026-11-07-eqwec-hub-launcher-design.md`

## Global Constraints

- `#![forbid(unsafe_code)]`; no `unwrap()`/`expect()` on recoverable production paths.
- Ukrainian imperative Conventional Commit subjects; type/scope in Latin.
- Version `0.1.0-alpha.1` for the launcher crate only.
- No network, no production keys, no secrets in the repository.
- `main` stays README-only; work lands on `dev-hub`, then `integration`.
- Every gate result recorded as `PASS`/`FAIL`/`BLOCKED`/`NOT_APPLICABLE` with the real command.

## Review Focus

- Corrupt/oversized registry: window shows a typed error, never panics.
- Committed registry record with missing or mismatched `active.json`: Launch disabled, "repair required".
- Wrong platform/architecture bundle: rejected before staging, previous install untouched.
- Uninstall interrupted between tree removal and registry write: recovery finishes it; retry is idempotent.
- Window closed during staging: next start recovers to a complete version.

---

### Task 1: Profile lock

**Files:**
- Modify: `Cargo.toml`, `crates/eqwec-updater/Cargo.toml`
- Create: `crates/eqwec-updater/src/lock.rs`
- Modify: `crates/eqwec-updater/src/lib.rs`, `crates/eqwec-updater/src/error.rs`
- Test: `crates/eqwec-updater/tests/lock.rs`

**Interfaces:**
- Produces: `eqwec_updater::ProfileLock::acquire(path: &Path) -> Result<ProfileLock, UpdaterError>`, `UpdaterError::ProfileLocked`.

- [ ] Add `fs4 = "1.1.0"` to workspace and updater dependencies; admit it in `DEPENDENCY-MAP.md`.
- [ ] Test: second `acquire` on the same path returns `ProfileLocked`; dropping the first allows re-acquire; a different path succeeds.
- [ ] Implement with `fs4::fs_std::FileExt::try_lock_exclusive` on a `File` opened `create(true).read(true).write(true)`; write version/PID diagnostics after locking.
- [ ] Run `cargo test -p eqwec-updater --test lock`, then commit `feat(hub): додати блокування профілю`.

### Task 2: Disk preflight

**Files:**
- Create: `crates/eqwec-updater/src/disk.rs`
- Modify: `crates/eqwec-updater/src/lib.rs`, `src/error.rs`
- Test: `crates/eqwec-updater/src/disk.rs` unit tests

**Interfaces:**
- Produces: `eqwec_updater::ensure_free_space(path: &Path, required_bytes: u64) -> Result<u64, UpdaterError>`, `UpdaterError::InsufficientDiskSpace { available, required }`, `eqwec_updater::required_staging_bytes(package_size: u64) -> Option<u64>`.

- [ ] Test the arithmetic: `required_staging_bytes(0) == Some(64 MiB)`, `1 GiB → 2 GiB + 64 MiB`, overflow → `None`.
- [ ] Test `ensure_free_space` against `std::env::temp_dir()` with `required_bytes = 0` (passes) and `u64::MAX` (fails with `InsufficientDiskSpace`).
- [ ] Implement with `fs4::available_space`; reject `required_bytes` overflow.
- [ ] Run tests; commit `feat(hub): перевіряти вільне місце перед розпакуванням`.

### Task 3: Journaled uninstall

**Files:**
- Create: `crates/eqwec-updater/src/uninstall.rs`, `docs/adr/0013-hub-uninstall-transaction.md`
- Modify: `src/journal.rs` (`Phase::UninstallIntent`, `Phase::Uninstalled`, terminal set), `src/activation.rs` (recovery), `src/lib.rs`, `src/error.rs`
- Test: `crates/eqwec-updater/tests/uninstall.rs`

**Interfaces:**
- Produces: `uninstall(layout, registry, product_id, operation_id, idempotency_key, updated_at) -> Result<UninstallOutcome, UpdaterError>`; `UninstallOutcome::{Removed, FilesRemovedRegistryPending}`.
- Consumes: `InstallLayout`, `Registry`, `OperationId`, `IdempotencyKey`, `Timestamp`.

- [ ] Test: removes tree and record; refuses a record whose `install_root` is outside the layout; missing product is a typed error; second call is `NotFound`.
- [ ] Test fault injection: fail after tree removal, before registry write; `recover` removes the record and appends `uninstalled`; retry is idempotent.
- [ ] Implement phases and recovery mapping per spec §8.1; registry write is last.
- [ ] Write ADR-0013 and update the schema document's phase table.
- [ ] Run `cargo test -p eqwec-updater`; commit `feat(hub): додати журнальоване видалення продукту`.

### Task 4: Launcher skeleton, paths, state, self-check

**Files:**
- Create: `apps/eqwec-hub/Cargo.toml`, `src/main.rs`, `src/paths.rs`, `src/state.rs`, `src/trust.rs`, `src/app.rs`, `assets/dev-trust-store.json`
- Modify: root `Cargo.toml` members
- Test: `src/paths.rs`, `src/state.rs` unit tests

**Interfaces:**
- Produces: `paths::data_dir_from(env) -> Result<PathBuf, LauncherError>`, `state::LauncherState::load(layout) -> LauncherState`, `state::ProductView`, `main` dispatch for `--self-check`.

- [ ] Test data-dir resolution for macOS/Windows/Linux using injected env maps; reject relative `EQWEC_DATA_DIR`.
- [ ] Test state mapping: missing registry → `NotInitialized`; corrupt → `Error`; record with mismatched pointer → `RepairRequired`.
- [ ] Implement `--self-check` before any eframe call; exit 0 healthy/empty, 1 corrupt/inconsistent.
- [ ] Implement the window: header, banner, status strip, list, empty/error/read-only states, Refresh.
- [ ] Run `cargo run -p eqwec-hub -- --self-check` and `cargo test -p eqwec-hub`; commit `feat(hub): додати каркас лаунчера EQwec Hub`.

### Task 5: Actions, health check, launch

**Files:**
- Create: `apps/eqwec-hub/src/actions.rs`, `src/launch.rs`, `src/worker.rs`
- Modify: `src/app.rs`, `src/state.rs`
- Test: `apps/eqwec-hub/tests/launcher_actions.rs`

**Interfaces:**
- Produces: `actions::install_bundle(data_dir, bundle_dir)` (install and update), `actions::rollback(data_dir, product_id, version)`, `actions::uninstall_product(data_dir, product_id)`, `actions::resolve_installed_entry(...) -> Result<(PathBuf, PathBuf), LauncherError>`, `actions::launch_product(...) -> Result<u32, LauncherError>`, `launch::resolve_entry(layout, record, pointer) -> Result<ResolvedEntry, LauncherError>`, `launch::health_check(tree, product_id) -> Result<(), LauncherError>`.

- [x] Test bundle validation: missing `manifest.json`/`package.zip`, extra files, relative path.
- [x] Test `resolve_entry`: missing pointer, pointer/record mismatch, symlink tree, symlink entry, missing file, path escape → typed errors; valid tree → `bin/<id>`.
- [x] Test health check: fixture v1 tree passes; tree with missing payload fails; timeout kills the child.
- [x] Implement install/update/rollback with disk preflight → verify → stage → health check → activate; worker thread + channel.
- [x] Implement uninstall with confirmation modal and launch with `Command`.
- [x] Integration test: install v1 → launch → update v2 → rollback v1 → uninstall against temp layout.
- [x] Run `cargo test -p eqwec-hub`; commit `feat(hub): підключити встановлення, оновлення, відкат і запуск`.

### Task 6: Fixture entry point and demo bundle

**Files:**
- Modify: `tests/fixtures/eqwec-hub/Cargo.toml`, `tests/fixtures/eqwec-hub/tests/hub_end_to_end.rs`
- Create: `tests/fixtures/eqwec-hub/src/bin/make_demo_bundle.rs`

**Interfaces:**
- Produces: `cargo run -p eqwec-hub-fixture --bin make-demo-bundle -- <outdir>` writing `manifest.json` + `package.zip` for the host target and current time.

- [x] Rename packaged executable to `bin/eqwec-fixture` (`.exe` on Windows); update the end-to-end tests.
- [x] Implement the generator using the public fixture seed, `Timestamp::now()` or the current clock, host `Platform`/`Architecture`.
- [x] Test: generated bundle parses and verifies with the launcher trust store.
- [x] Run fixture tests; commit `test(hub): генерувати демонстраційний пакет для лаунчера`.

### Task 7: Packaging

**Files:**
- Create: `packaging/eqwec-hub/macos/build-dmg.sh`, `packaging/eqwec-hub/windows/eqwec-hub.wxs`, `packaging/eqwec-hub/windows/build-msi.ps1`, `packaging/eqwec-hub/linux/build-deb.sh`, `packaging/eqwec-hub/linux/eqwec-hub.desktop`, `.github/workflows/package.yml`
- Modify: `README.md` (artifact instructions)

- [ ] macOS: build release, assemble `.app`, ad-hoc sign, `hdiutil create`, mount, run `--self-check`.
- [ ] Windows: WiX v4 dotnet tool, per-user MSI to `%LOCALAPPDATA%\Programs\EQwec Hub`, no elevation.
- [ ] Linux: `.deb` + `.desktop` + `tar.zst`; install with `dpkg -i` in CI and run `--self-check`.
- [ ] `package.yml`: `workflow_dispatch` + `v*`, matrix, upload artifacts, 14-day retention.
- [ ] Commit `build(hub): додати пакування для macOS, Windows і Linux`.

### Task 8: Documentation and gates

**Files:**
- Modify: `AGENTS.md`, `README.md`, `CLAUDE.md`, `docs/development/QUALITY-GATES.md`, `TEST-STRATEGY.md`, `SKILL-USAGE.md`, `ROADMAP.md`, `docs/research/FEASIBILITY.md`, `docs/architecture/DEPENDENCY-MAP.md`, `docs/architecture/SYSTEM-ARCHITECTURE.md`, `docs/security/THREAT-MODEL.md`, `docs/product-specs/eqwec-hub-schemas.md`, `docs/adr/0007-packaging-and-signing.md`, `.agents/skills/eqwec-hub/SKILL.md`

- [ ] Run fmt, clippy, build, full tests, audit, deny, skills, docs; record exact results.
- [ ] Run `package.yml` on all three platforms; record artifact and `--self-check` evidence.
- [ ] Reconcile every status claim; keep GUI window on Windows/Linux `BLOCKED` unless a desktop run exists.
- [ ] Commit `docs(hub): зафіксувати стан лаунчера та пакування`.
