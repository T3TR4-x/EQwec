# EQwec

**Engineer Quality with Empathy to Customer**

EQwec is an independent ecosystem of professional desktop applications and a unified software launcher. The project is built around a Rust-first, native-first architecture with measurable standards for correctness, security, privacy, performance, accessibility, and long-term maintainability.

## Status

**Phase 3 — Hub MVP.** The project is at version `0.0.0`. No product has a usable release yet, and nothing in this repository should be treated as production-ready.

The Cargo workspace, pinned toolchain, shared crates, and CI workflow exist on `integration`. Hub verification, durable state, activation, rollback, recovery, bounded package staging, journaled uninstall, a native launcher UI, local signed-bundle install/update/rollback/uninstall/launch, and packaging scripts for macOS, Windows, and Linux are implemented on `dev-hub`. The mandatory skills, research, architecture, and security gates have passed for the planning scope. GitHub Actions CI run `42398364288` passed the workspace tests on macOS, Ubuntu, and Windows; packaging run `42398364283` built and smoke-tested the macOS DMG, Windows MSI, and Linux packages. Download transport, repair, dependency resolution, release signing, notarization, and Windows/Linux desktop acceptance remain `BLOCKED`.

## Products

| Product | Purpose | Status |
|---|---|---|
| EQwec Hub | Launcher, package manager, installer, updater, and installation health manager | MVP in progress: verification, activation, rollback, recovery, package staging, journaled uninstall, native launcher UI, local signed-bundle workflows, and packaging scripts implemented; download transport and release signing are not |
| EQwec Browser | Chromium-based browser with Rust-owned application logic and a native UI | Research and feasibility |
| PhotoCraft | Layer-based, non-destructive image editing and photography | Planned |
| VectorCraft | Vector graphics, Bézier path editing, and SVG interoperability | Planned |
| FilmCraft | Non-linear video editing, playback, and rendering | Planned |
| EffectCraft | Compositing, animation, and visual effects | Planned |
| CraftAnything | Cross-application creative workspace and asset exchange | Scope definition |
| AudioCraft | Digital audio workstation with real-time-safe audio processing | Planned |

Product names are provisional where noted by the planning documents. Each product will have its own version history, release cadence, and installation package.

## Architecture

The approved default is a Rust-first modular monorepo built with a Cargo workspace:

- Application entry points live under `apps/`.
- Reusable, independently testable libraries live under `crates/`.
- Product binaries are built, packaged, versioned, and released independently.
- Shared infrastructure is versioned and tested as part of the workspace.
- Native platform integration and a lightweight UI stack are preferred over bundling a browser runtime for application chrome.
- Security boundaries, tenant ownership, and untrusted input handling are explicit parts of the design.

The browser will use a maintained Chromium-based foundation. EQwec does not claim to rewrite the Chromium rendering engine in Rust; the Rust layer owns application logic, orchestration, persistence, and the native user interface.

## Platforms

Planned targets are Windows, macOS, and Linux on x86_64 and ARM64 where technically feasible. The current development environment is macOS on Apple silicon. No platform is supported for public use until its release gates pass.

## Run the Hub developer preview

The launcher is a developer preview. It accepts only locally supplied bundles signed with the public fixture key embedded in `apps/eqwec-hub/assets/dev-trust-store.json`; that key is test material and must never sign a release.

```bash
# 1. Open the launcher window.
cargo run -p eqwec-hub

# 2. In another terminal, generate a signed demo bundle for this host.
cargo run -p eqwec-hub-fixture --bin make-demo-bundle -- /tmp/eqwec-demo/bundle

# 3. Paste the printed absolute path into the launcher, then Install / Update and Launch.
#    The installed fixture prints its generation and payload.

# 4. Generate a newer bundle into the same folder, install it, and use Roll back.
cargo run -p eqwec-hub-fixture --bin make-demo-bundle -- /tmp/eqwec-demo/bundle 1.1.0 2
```

`--self-check` reports profile state without opening a window:

```bash
cargo run -p eqwec-hub -- --self-check
```

Packaging scripts build local artifacts into `dist/`:

| Platform | Command | Artifact |
|---|---|---|
| macOS | `./packaging/eqwec-hub/macos/build-dmg.sh` | Ad-hoc-signed `.app` in a DMG; the script mounts it and runs `--self-check` |
| Windows | `./packaging/eqwec-hub/windows/build-msi.ps1` | Per-user MSI for `%LOCALAPPDATA%\Programs\EQwec Hub`; requires the .NET SDK and pins the WiX 4.0.6 tool |
| Linux | `./packaging/eqwec-hub/linux/build-deb.sh` | `.deb` and `tar.zst`; requires `dpkg-deb` and `zstd` |

These artifacts are unsigned developer builds. Developer ID signing, notarization, Authenticode, and a release channel do not exist yet.

## Repository Branches

- `main` is the informational branch. It contains only this README.
- `integration` contains the current workspace, shared infrastructure, policies, skills, and architecture documents.
- `dev-<product>` branches contain product-specific work.
- `production` contains reviewed release source and release metadata.

Development documentation, agent instructions, and project skills are maintained on `integration` and product branches, not on `main`.

## Licensing and Ownership

No public license has been selected yet. Until a license is published, no permission is granted to redistribute the source or binaries. Third-party dependencies and assets will be tracked with their licenses before any release.

## Releases

No releases exist. Release tags will be namespaced per product, for example `eqwec-hub/v0.1.0`, and will only be created after the documented quality, security, packaging, and signing gates pass.
