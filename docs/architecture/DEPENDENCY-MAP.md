# Dependency Map

**Status:** planning baseline with an admitted verification core. The seven dependencies in §9 are admitted for manifest, trust, package verification, and the non-production fixture tests. GUI, media, audio, browser, and packaging technologies remain candidates under research and legal review.

## 1. Internal dependency direction

```text
L4 apps
  |  depends on product crates and shared crates
  v
L3 product domain crates
  |  depends on L0-L2 and its own product crates
  v
L2 domain infrastructure: eqwec-graphics, eqwec-media, eqwec-ui
  |  depends on L0-L1
  v
L1 platform services: eqwec-config, eqwec-storage, eqwec-logging,
                      eqwec-security, eqwec-ipc, eqwec-updater
  |  depends on L0
  v
L0 primitives: eqwec-core
```

Rules:

- Dependencies point downward only.
- A shared crate must not depend on an app or a product-specific crate.
- A product crate must not depend on another product's internal crate.
- Cross-product behavior uses a shared crate with an owner or a versioned interchange contract.
- A shared crate is created only when it has a real owner, API, tests, and a second consumer or a documented architectural reason.

## 2. Shared crate ownership

| Crate | Owner | Consumers | Public contract |
|---|---|---|---|
| `eqwec-core` | Core team | All | Typed IDs, errors, scope, units, common traits |
| `eqwec-config` | Platform team | All | Versioned configuration schema, validation, migration |
| `eqwec-storage` | Platform team | All | Atomic persistence, journals, tenant/profile scoping |
| `eqwec-logging` | Platform team | All | Structured, redacted diagnostics |
| `eqwec-security` | Security team | Hub, Browser, all products | Crypto verification, path safety, permissions |
| `eqwec-ipc` | Platform team | Hub, Browser, CraftAnything | Versioned local IPC with authorization |
| `eqwec-updater` | Hub team | Hub (generic API for others if justified) | Download, verify, stage, activate, roll back |
| `eqwec-graphics` | Graphics team | Photo, Vector, Film, Effect, CraftAnything | GPU device/resource management, rendering abstractions |
| `eqwec-media` | Media team | Film, Effect, Audio, Photo | Bounded decode/encode and media primitives |
| `eqwec-ui` | Design-system team | All | Tokens and reusable UI primitives, not product workflows |

Ownership means API review, compatibility expectations, tests for every consumer, and a named maintainer.

## 3. External dependency categories (candidates only)

The following are candidates for evaluation. None is approved, versioned, or licensed for distribution yet. The legal review and technology ADRs must approve each before it enters `Cargo.toml`.

| Category | Candidates | Notes |
|---|---|---|
| GUI | Slint, Iced, egui, platform-native | Pending ADR-0002; license and accessibility differ |
| GPU/rendering | wgpu, Vello, Skia (skia-safe) | Pending ADR-0002; CPU fallback required |
| Text/fonts | HarfBuzz, FreeType, fontdb, cosmic-text | License and shaping quality must be verified |
| Accessibility | AccessKit and platform bridges | Must be verified per framework/platform |
| Browser engine | CEF, Chromium fork, WebView2, WKWebView, Servo | Pending ADR-0003 |
| Media decode/encode | FFmpeg, GStreamer, Symphonia, platform codecs | Pending ADR-0004; LGPL/GPL and patent issues |
| Image/RAW | image-rs, LibRaw, rawspeed, OpenColorIO, LittleCMS | Pending ADR-0004; camera and color constraints |
| Vector | lyon, kurbo, usvg/resvg, custom geometry | Pending ADR; boolean robustness must be proven |
| Audio I/O | CPAL, platform audio APIs | Pending ADR-0005; real-time behavior must be verified |
| Audio plugins | CLAP, VST3, LV2 | Pending ADR-0005/0010; license and sandbox constraints |
| Serialization | serde, CBOR/MessagePack/Protobuf, custom canonical format | Pending ADR-0006 |
| Storage | SQLite (rusqlite), sled, redb, filesystem journals | Pending ADR; crash safety and migration |
| Crypto | RustCrypto, ring, aws-lc-rs, platform stores | Pending security review; no homegrown crypto |
| IPC | Unix sockets/named pipes, shared memory | Pending ADR; authentication and bounds |
| Logging | tracing, log | Redaction and bounded output are project requirements |
| Testing | cargo-nextest, proptest, cargo-fuzz, criterion | Not yet adopted |
| Security tooling | cargo-audit, cargo-deny, secret scanners | `cargo-audit` and `cargo-deny` run locally and in CI; secret scanning not adopted |
| Packaging | WiX/MSIX, Apple notarization, deb/rpm/AppImage/Flatpak/Snap | Pending ADR-0007 |

## 4. Dependency admission policy

A dependency is admitted only when:

1. It is necessary; the standard library or an existing dependency cannot solve the problem.
2. Its license is compatible with the project's distribution model and is recorded in the license inventory.
3. It is maintained; security response and release cadence are acceptable.
4. It is widely used or has a credible audit/testing story for the risk it carries.
5. It can be isolated or replaced without rewriting product semantics.
6. It does not introduce an unacceptable binary-size, memory, startup, or platform burden.
7. It passes advisory and license checks in CI.
8. Its use is documented in the dependency map and, for a major choice, an ADR.

Do not add a dependency for a few lines of trivial code. Do not vendor a fork without an owner, update plan, and security patch process.

## 5. Forbidden or restricted dependencies

- No Electron or bundled browser runtime for native app chrome unless an ADR proves it is the only viable option and the cost is accepted.
- No GPL/AGPL dependency in a product distributed under a permissive license unless the legal review explicitly approves the combined distribution and obligations.
- No unmaintained crypto, parser, or codec library for untrusted input.
- No dependency that disables sandboxing, certificate validation, or signature checks.
- No telemetry/analytics SDK enabled by default.
- No proprietary codec or DRM bundle without a license and distribution decision.
- No product-to-product internal dependency.

## 6. Versioning and compatibility

- Pin the toolchain and use `Cargo.lock` for the application workspace.
- Shared crates follow SemVer and document compatibility.
- A breaking shared-crate change requires an impact assessment and cross-product tests.
- Third-party versions are pinned or bounded; updates are reviewed and tested.
- Record engine versions (Chromium, codecs, audio backend) in release metadata where applicable.

## 7. Supply-chain controls

- Advisory and license scanning in CI.
- SBOM and third-party notices before release.
- Secret scanning and least-privilege CI tokens.
- Reproducible builds where feasible.
- Review build scripts and proc macros as code.
- Do not auto-merge major dependency updates without compatibility tests.

## 8. Current state

Last verified: 2026-11-07.

| Item | State |
|---|---|
| Cargo workspace | Exists: resolver 3, edition 2024, Rust 1.99.1, members `crates/*` and `tests/fixtures/eqwec-hub` |
| Workspace crates | `eqwec-core`, `eqwec-security`, `eqwec-updater`; non-published `eqwec-hub-fixture` test crate; seven direct third-party dependencies |
| Approved dependencies | The seven direct dependencies in §9 passed advisory, license, ban, and source checks on 2026-11-07 |
| Lockfile | `Cargo.lock` committed; 64 locked packages |
| SBOM | None |
| License inventory | Planning review complete: `docs/research/LEGAL-LICENSE-REVIEW.md`; codec/DRM items `BLOCKED` pending counsel |
| Technology selection | Planning baseline: `docs/architecture/TECHNOLOGY-SELECTION.md` |
| ADR set | ADR-0001 through ADR-0012 recorded; ADR-0004 and ADR-0008 are `PROPOSED` |
| Advisory/license CI | PASS in GitHub Actions run `42389844319` on commit `c066d67`; local commands also pass |

## 9. Admitted dependencies

These are the only third-party direct dependencies admitted so far. Admission covers the exact locked version for local verification; a version update requires a fresh advisory and license check. Workspace crates still have no `license` field because ADR-0008 is `PROPOSED`; `deny.toml` exempts private workspace crates only.

| Crate | Declared | Locked | License | Purpose | Used by |
|---|---|---|---|---|---|
| `ed25519-dalek` | 3.0.0 | 3.0.0 | BSD-3-Clause | Strict Ed25519 verification; signing in the non-production fixture tests | `eqwec-security`, `eqwec-hub-fixture` (dev) |
| `sha2` | 0.11.1 | 0.11.1 | MIT OR Apache-2.0 | SHA-256 digests | `eqwec-security`, `eqwec-updater` |
| `serde` | 1.0.228 | 1.0.229 | MIT OR Apache-2.0 | Derive deserialization for wire structs | `eqwec-updater` |
| `serde_json` | 1.0.150 | 1.0.154 | MIT OR Apache-2.0 | Bounded JSON parsing; fixture manifest construction | `eqwec-updater`, `eqwec-hub-fixture` (dev) |
| `semver` | 1.0.28 | 1.0.28 | MIT OR Apache-2.0 | Version and constraint validation | `eqwec-updater`, `eqwec-hub-fixture` (dev) |
| `thiserror` | 2.0.18 | 2.0.21 | MIT OR Apache-2.0 | Typed error definitions | `eqwec-security`, `eqwec-updater` |
| `zip` | 9.4.1 | 9.4.1 | MIT | Bounded ZIP/ZIP64 package validation and extraction; fixture package construction | `eqwec-updater`, `eqwec-hub-fixture` (dev) |

`eqwec-hub-fixture` is non-published test infrastructure. Its dev-dependencies reuse admitted crates; it introduces no new third-party dependency. `zip` pulls `flate2` (MIT OR Apache-2.0) and `zlib-rs` (Zlib) for the pure-Rust deflate backend. They are transitive and pinned by `Cargo.lock`; a direct use requires a new admission entry.

`cargo deny check` reported `advisories ok, bans ok, licenses ok, sources ok` on 2026-11-07. The duplicate `syn` 2.x/3.x warning is accepted because the two versions are isolated to proc-macro expansion.

This map is updated whenever a direct dependency is added, removed, or upgraded.
