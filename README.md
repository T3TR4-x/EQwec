# EQwec

**Engineer Quality with Empathy to Customer**

EQwec is an independent ecosystem of professional desktop applications and a unified software launcher. The project is built around a Rust-first, native-first architecture with measurable standards for correctness, security, privacy, performance, accessibility, and long-term maintainability.

## Status

**Phase 3 — Hub MVP.** The project is at version `0.0.0`. No product has a usable release yet, and nothing in this repository should be treated as production-ready.

The Cargo workspace, pinned toolchain, `eqwec-core` shared crate, and CI workflow exist on `integration`. Hub verification, durable state, activation, rollback, recovery, bounded package staging, and a non-production v1/v2 fixture workflow are implemented on `dev-hub`. The mandatory skills, research, architecture, and security gates have passed for the planning scope. Download transport, uninstall, the launcher UI, installer packaging, platform fixture runs, and every other product remain `BLOCKED`. The GitHub remote is connected and the first CI run is pending; record its result before calling CI verified.

## Products

| Product | Purpose | Status |
|---|---|---|
| EQwec Hub | Launcher, package manager, installer, updater, and installation health manager | MVP in progress: verification, activation, rollback, recovery, package staging, and an in-process v1/v2 fixture workflow implemented; no installer or launcher UI yet |
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
