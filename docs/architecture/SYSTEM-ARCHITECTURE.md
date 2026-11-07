# System Architecture

**Status:** planning baseline with a Phase 2 workspace. ADR-0001 through ADR-0010 are recorded; ADR-0004 and ADR-0008 remain `PROPOSED`. `crates/eqwec-core` exists; no application or product crate exists.

## 1. Goals

- A Rust-first modular monorepo with independently buildable, testable, versioned, and releasable products.
- Native-first desktop applications with predictable resource use and no default browser runtime for app chrome.
- Explicit security boundaries, bounded parsing, and safe failure.
- Shared infrastructure where it is genuinely shared; product isolation where behavior differs.
- A coherent user experience and design system without forcing one layout onto every tool.
- Offline-first operation and local data ownership.

Non-goals for the first milestones:

- Reimplementing Chromium in Rust.
- A universal editor that duplicates every product's internals.
- Cloud accounts, mandatory sync, or telemetry.
- Microservices for local desktop functionality.
- Releasing every product together.

## 2. Architectural principles

1. **Product boundaries are real.** A product owns its document model, editing semantics, and user workflow. Shared crates provide infrastructure, not product policy.
2. **Dependencies point inward.** Apps depend on product crates and shared crates; shared crates never depend on an app or another product's internal crate.
3. **One canonical owner per document.** CraftAnything references documents; it does not own their bytes.
4. **Contracts are typed and versioned.** IPC, manifests, project schemas, and interchange packages have explicit versions and compatibility tests.
5. **Untrusted input is bounded.** Parsers have size, depth, count, and time limits; failures are typed and safe.
6. **Real-time paths are allocation-free and blocking-free.** Audio callbacks and render-critical loops do not allocate, lock unboundedly, perform I/O, or await.
7. **Resource lifetimes are explicit.** Files, processes, GPU resources, threads, and temp files have owners and cleanup paths.
8. **Measure before optimizing.** Performance claims require a reproducible baseline.
9. **Reversible decisions are preferred.** Expensive-to-reverse choices require an ADR with reconsideration criteria.
10. **Documentation describes reality.** Planned work is labeled planned.

## 3. System context

```text
                         +---------------------------+
                         |        EQwec Hub          |
                         | launcher / installer /    |
                         | updater / registry        |
                         +-------------+-------------+
                                       |
                     signed packages / manifests / IPC
                                       |
     +----------------+----------------+----------------+----------------+
     |                |                |                |                |
+----v----+     +-----v-----+    +-----v-----+    +-----v-----+    +-----v-----+
| Browser |     | PhotoCraft|    | VectorCraft|   | FilmCraft |    | EffectCraft|
+---------+     +-----------+    +-----------+    +-----------+    +-----------+
     |                |                |                |                |
     +----------------+----------------+----------------+----------------+
                                       |
                              +--------v--------+
                              |  CraftAnything  |
                              | workspace /     |
                              | asset exchange  |
                              +-----------------+
                                       |
                              +--------v--------+
                              |    AudioCraft   |
                              | DAW / audio I/O |
                              +-----------------+
```

Hub is the installation and update authority. It is not a runtime dependency of an installed product unless a documented contract requires it. CraftAnything coordinates assets and workflows; it does not replace the owning editors.

## 4. Repository layout

```text
EQwec/
├── Cargo.toml                 # workspace root (planned)
├── Cargo.lock                 # application workspace lock (planned)
├── rust-toolchain.toml        # pinned toolchain (planned)
├── README.md                  # main and development branches
├── AGENTS.md                  # canonical agent entry point
├── CLAUDE.md                  # Claude-specific entry point
├── .agents/skills/            # canonical project skills
├── apps/
│   ├── eqwec-hub/
│   ├── eqwec-browser/
│   ├── photocraft/
│   ├── vectorcraft/
│   ├── filmcraft/
│   ├── effectcraft/
│   ├── craftanything/
│   └── audiocraft/
├── crates/
│   ├── eqwec-core/
│   ├── eqwec-ui/
│   ├── eqwec-config/
│   ├── eqwec-storage/
│   ├── eqwec-logging/
│   ├── eqwec-updater/
│   ├── eqwec-security/
│   ├── eqwec-ipc/
│   ├── eqwec-graphics/
│   └── eqwec-media/
├── tests/
│   ├── integration/
│   ├── regression/
│   ├── security/
│   └── performance/
├── benchmarks/
├── packaging/
├── scripts/
├── docs/
└── .github/workflows/         # planned
```

This is a target, not an instruction to create empty crates. A crate is created when it has a real owner, API, tests, and a second consumer or a documented architectural reason.

## 5. Crate layers

| Layer | Crates | Responsibility | May depend on |
|---|---|---|---|
| L0 — primitives | `eqwec-core` | Typed IDs, errors, result conventions, time/units, tenant/profile scope, common traits | Standard library, minimal utilities |
| L1 — platform services | `eqwec-config`, `eqwec-storage`, `eqwec-logging`, `eqwec-security`, `eqwec-ipc` | Configuration, atomic persistence, redacted diagnostics, crypto/path safety, versioned IPC | L0 |
| L2 — domain infrastructure | `eqwec-graphics`, `eqwec-media`, `eqwec-ui` | GPU resources and rendering abstractions, bounded decode/encode, design tokens and UI primitives | L0, L1 |
| L3 — product domain | `eqwec-hub-*`, `eqwec-browser-*`, `eqwec-photo-*`, `eqwec-vector-*`, `eqwec-film-*`, `eqwec-effect-*`, `eqwec-craft-*`, `eqwec-audio-*` | Product-specific models, commands, persistence, rendering, workflows | L0–L2, own product crates |
| L4 — apps | `apps/*` | Composition root, windowing, command routing, packaging entry point | Product crates and shared crates |

Rules:

- A lower layer must not depend on a higher layer.
- A product crate must not depend on another product's crate.
- `eqwec-ui` contains reusable primitives and tokens, not product workflow logic.
- `eqwec-media` contains bounded, generic media primitives; FilmCraft/EffectCraft/AudioCraft own their domain semantics.
- `eqwec-updater` is Hub-owned but exposes a generic transaction engine; it must not contain Hub UI or product-specific policy.
- Shared crates require an owner, public API, compatibility expectations, and tests that exercise every consumer.

## 6. Process and concurrency model

### General applications

- A UI/main process owns windows, commands, and application state.
- CPU-heavy work (decode, encode, render, indexing) runs on bounded worker pools.
- Long operations are cancellable and report progress.
- State is shared through message passing or immutable snapshots; locks are short and never held across an `await`.
- A crash must not leave a half-written project; persistence uses journals and atomic rename.

### Browser

- Rust owns the application process, native UI, profiles, tabs, sessions, downloads, permissions, and engine supervision.
- Chromium owns renderer, GPU, network, utility, and extension processes.
- The renderer-to-app IPC boundary is untrusted and validated.
- Profiles are isolated; one engine user-data directory per active profile is the planned default.
- A renderer crash is contained and recoverable; it does not crash the browser shell.

### Audio

- A dedicated real-time audio thread/callback owns the sample clock and graph processing.
- No allocation, blocking lock, file I/O, network, logging, or unbounded work in the callback.
- UI and disk operations communicate through bounded lock-free or message-passing queues.
- Plugin hosting is out-of-process where feasible; a plugin crash must not take down the DAW.
- The sample clock is the sole authority for scheduling and rendering.

### Compositing and rendering

- The UI thread and render thread are separated where the framework requires it.
- GPU resources are owned by a device manager; cache keys include content, transform, viewport, device scale, and backend.
- CPU is the correctness reference for effects and vector rendering; GPU output must match within a documented tolerance or fall back.
- Device loss is recoverable; caches are invalidated safely.

## 7. Data ownership and persistence

| Data | Owner | Notes |
|---|---|---|
| Photo project | PhotoCraft | Native format; export for interchange |
| Vector project | VectorCraft | Native format; SVG export/import |
| Film project | FilmCraft | Native format; media references, not media copies by default |
| Effect project | EffectCraft | Native format; FilmCraft interchange contract |
| Audio project | AudioCraft | Native format; rendered audio export |
| Browser profile/session | Browser | Per-profile, atomic, schema-versioned |
| Installation registry | Hub | Authoritative installed-state record |
| Workspace manifest/provenance | CraftAnything | References canonical documents; no duplicate mutable payloads |
| Shared cache | Product or shared crate | Disposable, rebuildable, bounded, scoped |

Rules:

- Every path, query, cache key, and IPC message that belongs to a user or tenant carries an explicit tenant/profile scope.
- Writes are atomic: temp file in the target directory, flush, rename, retain a recoverable previous version.
- Schemas are versioned and migrated with tests for every version boundary.
- Unknown extension data is preserved or explicitly rejected; it is never silently dropped.
- Uninstall preserves user projects by default; purge requires explicit confirmation.

## 8. Shared contracts

- **IDs:** stable, typed, opaque; never derived from display names or paths.
- **Errors:** typed, actionable, redacted; no secrets or raw content in diagnostics.
- **Versions:** independent product versions; versioned schemas and IPC; compatibility tests.
- **IPC:** local-only, authenticated, versioned, bounded, authorized; no arbitrary paths/URLs/commands from unprivileged peers.
- **Project format:** canonical encoding, bounded parsing, explicit schema version, migration path.
- **Interchange:** explicit export/import package with provenance and fidelity declaration; no live foreign project mutation.
- **Design system:** shared tokens and primitives, product-specific layouts.

## 9. Security boundaries

The architecture must preserve:

- Chromium sandbox and site isolation.
- Out-of-process or otherwise isolated risky parsers/plugins where feasible.
- Least privilege and per-user installation by default.
- Signature and integrity verification before execution or installation.
- Bounded parsing of all untrusted input.
- Redacted diagnostics and no default telemetry.
- Key custody outside the repository.

See `docs/security/THREAT-MODEL.md` and `docs/security/SECURITY-BASELINE.md`.

## 10. Build and release independence

- The workspace builds shared crates and product crates together for compatibility.
- Each product has its own binary, package, version, changelog, and release channel.
- A shared-crate change requires an impact assessment and cross-product tests.
- Products may be released on different schedules.
- A release is assembled from `production`, not from an arbitrary `integration` tip.

## 11. Open decisions

| Decision | State | ADR |
|---|---|---|
| GUI framework and rendering stack | ACCEPTED with a cross-platform prototype gate | ADR-0002 |
| Browser engine foundation | ACCEPTED with a prototype and legal gate | ADR-0003 |
| Media decode/encode and codec policy | PROPOSED; not normative; legal gate | ADR-0004 |
| Audio backend and plugin hosting | ACCEPTED with a real-time prototype gate | ADR-0005 |
| Project serialization format | ACCEPTED | ADR-0006 |
| Packaging and signing per platform | ACCEPTED with a release gate; keys and tooling do not exist | ADR-0007 |
| Original project license | PROPOSED; owner approval required | ADR-0008 |
| Crash reporting and diagnostics | ACCEPTED with an implementation gate | ADR-0009 |
| Plugin/extension trust model | ACCEPTED with prototype and isolation gates | ADR-0010 |

Technology selections are recorded in `docs/architecture/TECHNOLOGY-SELECTION.md`. Feasibility verdicts and the recommended sequence are in `docs/research/FEASIBILITY.md`.

## 12. Current state

- `Cargo.toml`, `Cargo.lock`, `crates/eqwec-core/`, and `.github/workflows/ci.yml` exist. There is no `apps/`, product crate, installer, or release artifact.
- This document describes the approved target, not implemented behavior.
- ADR-0001 through ADR-0010 are recorded. ADR-0004 and ADR-0008 remain `PROPOSED`; the other ADRs carry explicit prototype, legal, or release gates.
- The architecture gate is `PASS` for the planning baseline: the target architecture, dependency direction, security boundaries, and ADR set exist and agree. Runtime architecture verification is `BLOCKED` until the workspace and prototypes exist.
