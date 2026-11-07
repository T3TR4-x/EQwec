# Technology Selection

| Field | Value |
|---|---|
| Status | Planning baseline. Decisions are recorded in ADRs; a selection here is not a dependency approval. |
| Date | 2026-11-07 |
| Scope | Shared technology stack for all EQwec products |
| Evidence standard | Verified facts cite an official source; planning judgments are labeled as assumptions; unverified items are marked `UNVERIFIED` or `BLOCKED`. |
| Related | `docs/adr/`, `docs/architecture/DEPENDENCY-MAP.md`, `docs/research/LEGAL-LICENSE-REVIEW.md` |

## 1. Selection principles

1. Rust-first, native-first, and local-first.
2. Prefer a small number of shared stacks over one stack per product.
3. A technology must have a credible security-update path and an acceptable license.
4. Accessibility, internationalization, and text input are release criteria, not later polish.
5. Do not adopt a framework because it is popular; adopt it because it satisfies a documented requirement.
6. Do not use a web runtime for native application chrome without an ADR that proves the tradeoff.
7. Pin versions, record provenance, and keep every FFI boundary narrow and owned.
8. A selection that depends on an unresolved legal question is conditional, not approved.

## 2. Summary

| Area | Selected direction | ADR | State |
|---|---|---|---|
| Application UI | `egui`/`eframe` with a project-owned design layer; native menus/dialogs through platform integration | ADR-0002 | Accepted with prototype gate |
| GPU abstraction | `wgpu`; custom canvases render directly; no second GPU API in product code | ADR-0002 | Accepted |
| 2D geometry and text | `kurbo`, `lyon`, `cosmic-text`; CPU reference renderer; Vello evaluation | ADR-0002 | Accepted with evaluation gate |
| Browser engine | CEF behind a narrow Rust boundary; platform webviews as fallback | ADR-0003 | Accepted with prototype gate |
| Media decode/encode | FFmpeg LGPL-only build plus platform hardware codecs; royalty-free formats first | ADR-0004 | Proposed; legal gate |
| Image and RAW | `image`, `exr`, LibRaw, LittleCMS; OpenColorIO evaluation | ADR-0004 | Proposed; legal gate |
| Audio I/O | CPAL first; platform backends behind a real-time-safe interface | ADR-0005 | Accepted |
| Audio plugins | CLAP primary, VST3 secondary, LV2 later; out-of-process hosting | ADR-0005, ADR-0010 | Accepted with prototype gate |
| Project format | Versioned bundle with JSON manifest and content-addressed assets; ZIP single-file package | ADR-0006 | Accepted |
| Catalogs and caches | SQLite; never the canonical project state | ADR-0006 | Accepted |
| Cryptography | RustCrypto primitives; Ed25519 signatures; no homegrown cryptography | ADR-0007 | Accepted |
| IPC | Local sockets/named pipes with peer credentials, versioned messages, bounded payloads | ADR-0010 | Accepted |
| Diagnostics | Local structured logs and local crash records; opt-in export only | ADR-0009 | Accepted |
| Packaging | Per-platform native packages; signed manifests and artifacts | ADR-0007 | Accepted with release gate |
| Project license | MIT OR Apache-2.0 recommended; owner approval required | ADR-0008 | Proposed |

## 3. Application UI and rendering (ADR-0002)

### 3.1 Requirements

- Windows, macOS, and Linux on x86_64 and ARM64.
- Dense professional layouts: docking, panels, inspectors, timelines, node graphs, canvas surfaces.
- Custom visual identity, dark/light/high-contrast themes, scalable UI, reduced motion.
- Keyboard-first navigation, command palette, focus visibility, screen-reader access.
- Text input and IME for English, Ukrainian, and Slovak, including composed characters.
- High-DPI rendering and smooth pan/zoom on large canvases.
- Bounded memory and startup; no bundled browser runtime for app chrome.
- A permissive license compatible with an undecided project license.
- A credible fallback if accessibility or text input fails.

### 3.2 Candidates

| Candidate | License | Strengths | Weaknesses | Verdict |
|---|---|---|---|---|
| `egui`/`eframe` 0.35+ | MIT OR Apache-2.0 | Pure Rust; immediate mode; `wgpu` backend; AccessKit since 0.34; strong custom-widget and canvas story; large ecosystem | Immediate-mode redraw cost; text editing/IME less mature than native toolkits; native menus/dialogs need separate integration; frequent breaking releases | **Primary** |
| Slint 1.18 | GPLv3; royalty-free desktop/mobile/web license; commercial | Declarative DSL; good performance; native-looking widgets; accessibility support; embedded-to-desktop range | License couples the project to GPLv3 or a commercial/royalty-free agreement; DSL and toolchain; custom canvas integration less direct; open accessibility bugs | Fallback if license is resolved |
| Iced 0.14 | MIT | Pure Rust; Elm architecture; `wgpu`; good app-chrome model | Stable 0.14 has no AccessKit integration; text/IME and ecosystem less mature; custom editor widgets require more work | Rejected for v1 |
| Qt 6 via `cxx-qt` | LGPLv3 or GPLv3 or commercial | Mature accessibility, native integration, widgets, deployment tooling | C++ toolchain and bindings; large footprint; LGPL/commercial obligations; not Rust-first | Fallback only if accessibility cannot be met otherwise |
| Platform-native (WinUI 3, AppKit, GTK 4) | Platform-specific | Best integration and accessibility | Three separate UIs and skill sets; slowest path to one coherent product | Rejected for v1 |
| Tauri / web UI | MIT/Apache for Tauri; web stack | Fast UI iteration; web ecosystem | Bundled web runtime for app chrome; memory and accessibility costs; conflicts with native-first policy | Rejected |
| Electron | MIT | Familiar web stack | Explicitly not a browser; large runtime; wrong security model for EQwec | Rejected |

**Verified facts used above:**

- egui 0.35.0 (published 2026-06-25) requires AccessKit 0.24.1 as a core dependency since 0.34.
- Iced 0.14.0 has no AccessKit dependency; the accessibility issue has been open since 2020.
- Slint is available under GPLv3, a royalty-free desktop/mobile/web license, and a commercial license; the royalty-free license has distribution conditions and embedded royalties.
- Qt is available under LGPLv3, GPLv3 (some modules only), and commercial terms; mixing commercial and community editions in one project is not allowed.
- Electron's own security documentation states that Electron is not a web browser and is unsuitable for arbitrary untrusted content.

### 3.3 Decision

Use `egui`/`eframe` as the primary application UI framework, wrapped in a project-owned `eqwec-ui` layer that provides:

- design tokens, typography, spacing, color, elevation, motion, and focus rules;
- reusable primitives (buttons, fields, menus, dialogs, tabs, tables, inspectors, notifications);
- docking and panel layout;
- accessibility annotations and keyboard navigation conventions;
- internationalized string lookup and pluralization;
- platform integration adapters for menus, file dialogs, clipboard, drag-and-drop, notifications, and IME.

`egui` is a rendering and interaction library, not the EQwec design system. Product code must not depend on raw `egui` types outside the UI layer unless the dependency is reviewed.

Use `wgpu` as the single GPU abstraction. `egui`'s `wgpu` backend and custom canvas renderers share one device manager. Do not introduce a second GPU API (OpenGL, Metal, Direct3D, Vulkan) in product code without an ADR.

Use `kurbo` for Bézier and geometric math, `lyon` for tessellation, and `cosmic-text` for shaping and layout. Use a CPU reference renderer for correctness tests. Evaluate `vello_cpu` and `vello_gpu` for high-quality 2D rendering; do not make Vello a release dependency until its 0.x API and performance are verified on all target platforms.

Do not use Skia initially. Skia is mature and BSD-3-Clause, but it adds a large C++ build, binary size, and a second rendering model. Reconsider if the Rust rendering path cannot meet documented quality or performance budgets.

### 3.4 Prototype gate

Before product UI work, a time-boxed prototype must prove on Windows, macOS, and Linux:

1. A representative editor shell: docked panels, inspector, timeline or node graph, canvas, menus, dialogs, and command palette.
2. Text input and IME for English, Ukrainian, and Slovak, including dead keys and composed characters.
3. Screen-reader access through AccessKit and the platform bridge: names, roles, values, states, focus order, and live updates.
4. Keyboard-only navigation, visible focus, high-contrast mode, and reduced motion.
5. High-DPI rendering and 60 fps interaction on the mid-range hardware profile.
6. Native file dialogs, menus, clipboard, drag-and-drop, and multi-window behavior.
7. A measured idle and active memory baseline for the shell.

If the prototype fails accessibility or text input, the fallback order is Slint (if the license is resolved) and then Qt (if a commercial or LGPLv3 distribution is approved). A failed gate is a decision trigger, not a reason to lower the requirement.

## 4. Browser engine (ADR-0003)

Use CEF as the primary engine foundation, behind a narrow Rust boundary in `crates/eqwec-browser-engine`. Use platform webviews (WebView2 on Windows, WKWebView on macOS) as the fallback behind the same engine contract. Reject a Chromium fork, Servo, and Electron for the initial phase.

The full evidence, candidate matrix, codec/DRM constraints, and decision triggers are in `docs/research/BROWSER-ENGINE-FEASIBILITY.md`. The ADR records the decision and the prototype gate.

Key conditions:

- Pin a CEF stable branch and the `cef` crate version at prototype start; record the exact values.
- Keep all engine FFI in one crate; catch panics before the boundary; never persist engine handles.
- Preserve Chromium sandbox and site isolation; verify effective flags at runtime.
- Define the security patch SLA, rebuild path, rollback, and staged rollout before the first public release.
- Complete codec and DRM licensing before enabling proprietary media.
- Do not claim Chrome Web Store extension parity.

## 5. Media, image, and codecs (ADR-0004)

### 5.1 Media

Use FFmpeg libraries under an LGPL-2.1-or-later build for broad decode support. Build with GPL-only components disabled. Do not ship `libx264` or `libx265` in the default distribution. Use platform hardware encoders (VideoToolbox, Media Foundation, VA-API) or a separately reviewed encoder for H.264/HEVC output.

For the first FilmCraft implementation, prefer FFmpeg as a subprocess for export and batch rendering. This isolates the LGPL boundary and simplifies notices and relinking. Use linked libraries only where real-time playback requires them, and record the compliance checklist.

Use Symphonia (MPL-2.0) for audio decode where it covers the format, and FFmpeg for containers and codecs it does not. Use `cpal` for audio output and `rubato` for resampling.

### 5.2 Image and RAW

Use `image` for common raster formats, `exr` for OpenEXR, and `png`/`jpeg`/`webp` crates where format-specific control is needed. Use LibRaw for camera RAW; it is LGPL-2.1 or CDDL, and the LGPL option is preferred. Use LittleCMS (`lcms2`) for ICC profiles. Evaluate OpenColorIO for ACES and complex color pipelines; do not require it for the first release.

The photo pipeline uses a linear working space, bounded tiled processing, and an explicit display transform. The first release may limit supported color spaces and bit depths, but the architecture must not assume 8-bit sRGB everywhere.

### 5.3 Codec and DRM policy

- Default to royalty-free formats: PNG, JPEG, WebP, AVIF, Opus, FLAC, VP9, AV1.
- Treat H.264, AAC, HEVC, ProRes, DNxHR, and DRM as legal-gated features, not defaults.
- Do not ship DRM in the first release. Widevine, FairPlay, and PlayReady require agreements, provisioning, and privacy review.
- Record exact build flags, codec sets, and third-party notices per release.
- Mark codec/patent review `BLOCKED` until counsel or an authorized licensing decision exists.

## 6. Audio and plugins (ADR-0005, ADR-0010)

- Use CPAL for the first cross-platform audio I/O implementation, with platform backends (CoreAudio, WASAPI, ALSA/PipeWire/JACK) behind a project-owned trait.
- The real-time callback performs no allocation, blocking lock, file I/O, network, logging, or unbounded work.
- Use bounded lock-free queues between the audio thread and UI/disk workers.
- Host CLAP first, VST3 second, and LV2 later. CLAP is MIT; VST3 is MIT since SDK 3.8; LV2 is ISC. Do not support AAX initially.
- Host plugins out of process where feasible. A plugin crash must not take down the DAW. Scanning runs in a separate process with timeouts and crash isolation.
- Do not claim a security sandbox until the platform-specific isolation is implemented and tested.
- Use `midir` for MIDI I/O and `wmidi` for messages, subject to admission review.

## 7. Project format and storage (ADR-0006)

Use a versioned **project bundle**:

```text
project.eqwec/
  manifest.json        # schema, product, IDs, timestamps, asset index, hashes
  document.json        # document tree and properties
  assets/<sha256>      # content-addressed binary payloads
  preview.png          # optional preview
```

A single-file package is a ZIP container with the same layout. Parsing is bounded: entry count, uncompressed size, compression ratio, path rules, and duplicate names. Reject absolute paths, `..`, symlinks, and unknown critical fields. Writes are atomic: temp file in the target directory, flush, rename, retain a recoverable previous version.

Use SQLite for catalogs, indexes, and caches, not as the canonical project state. SQLite is public domain; `rusqlite` is MIT. Use `serde_json` for manifest and document metadata; use binary assets for pixel, audio, and video payloads. If profiling later proves JSON metadata is a bottleneck, evaluate CBOR with a canonical encoding, but do not add it preemptively.

Use SHA-256 for content addressing and package integrity. Use Ed25519 for release signatures. Do not invent a cryptographic format.

## 8. Cryptography, IPC, and diagnostics

- Cryptography: RustCrypto primitives (`sha2`, `ed25519-dalek` or an approved equivalent), reviewed and pinned. No homegrown crypto. Prefer platform keystores for secrets.
- IPC: local sockets or named pipes with peer credentials, versioned messages, explicit authorization, and bounded payloads. Shared memory only for audio/video buffers with a documented synchronization protocol.
- Logging: `tracing` with structured fields, rotation, size bounds, and redaction. No secrets, browsing history, project contents, or personal paths by default.
- Crash diagnostics: local crash record plus redacted log bundle; export is user-initiated. No automatic upload and no third-party analytics SDK by default.
- Telemetry: none by default. Any future telemetry requires a separate ADR, explicit consent, a documented data list, retention, and deletion.

## 9. Testing and tooling

| Area | Candidate | License | State |
|---|---|---|---|
| Test runner | `cargo-nextest` | MIT OR Apache-2.0 | Candidate; verify on CI |
| Property testing | `proptest` | MIT OR Apache-2.0 | Candidate |
| Fuzzing | `cargo-fuzz` / libFuzzer | MIT OR Apache-2.0 | Candidate |
| Benchmarks | `criterion` | MIT OR Apache-2.0 | Candidate |
| Snapshot tests | `insta` | Apache-2.0 | Candidate |
| Coverage | `cargo-llvm-cov` | MIT OR Apache-2.0 | Candidate |
| Advisories | `cargo-audit` | MIT OR Apache-2.0 | Candidate |
| License/dependency policy | `cargo-deny` | MIT OR Apache-2.0 | Candidate |
| License notices | `cargo-about` | MIT OR Apache-2.0 | Candidate |
| SBOM | `cargo-cyclonedx` | Apache-2.0 | Candidate |
| Desktop UI automation | Platform-specific; TBD | TBD | `BLOCKED` until prototype |
| Browser automation | Playwright | Apache-2.0 | Candidate; CLI execution `BLOCKED` until verified |

No tool is approved until it is installed, run, and recorded with an exit code in the workspace.

## 10. Packaging and signing (ADR-0007)

| Platform | Initial package | Signing | Notes |
|---|---|---|---|
| Windows | Per-user MSI (WiX Toolset) | Authenticode | Optional MSIX for Store distribution later; do not require elevation |
| macOS | `.app` in signed DMG or ZIP | Developer ID, hardened runtime, notarization, stapling | Direct distribution first; Mac App Store later if sandbox constraints allow |
| Linux | Portable `tar.zst` with `.desktop` entry; `.deb`/`.rpm` later; Flatpak for sandboxed distribution | minisign/Ed25519 over artifacts and manifests | Flatpak restricts Hub's ability to install arbitrary apps; document the limitation |

Release manifests are signed over the exact payload bytes, not a re-serialization. Use an envelope with key ID, signature, product, version, channel, platform, architecture, size, SHA-256, minimum OS, dependencies, publication time, and expiry. The client stores the last accepted version per channel and rejects downgrades unless a signed downgrade policy allows them.

Key custody: offline root key, online release key signed by the root, rotation and revocation procedure, no keys in the repository or CI logs. No release is possible until this exists.

## 11. Rejected or deferred technologies

| Technology | Decision | Reason |
|---|---|---|
| Electron | Rejected | Wrong security model for a browser; large runtime for app chrome |
| Tauri/web UI for native apps | Rejected for v1 | Bundled web runtime; accessibility and memory costs; native-first policy |
| Chromium fork | Deferred | Requires a permanent browser-platform team; CEF is the lower-risk start |
| Servo | Watch item | Embedding API and media/DRM/accessibility support not ready |
| Skia | Deferred | C++ build and binary size; Rust path must be proven insufficient first |
| Qt | Fallback | Mature but heavy; LGPL/commercial obligations; not Rust-first |
| Slint | Fallback | License coupling and accessibility gaps; reconsider if resolved |
| Iced | Rejected for v1 | No AccessKit in stable 0.14 |
| GStreamer | Deferred | Strong pipeline model, but plugin/codec packaging and LGPL complexity; FFmpeg is the first path |
| AAX | Rejected | Proprietary, paid, and not necessary for the first release |
| Widevine/FairPlay/PlayReady | Deferred | Legal agreements, provisioning, and privacy review required |

## 12. Open items and blockers

| Item | State | Unblocked by |
|---|---|---|
| egui accessibility and IME parity | `UNVERIFIED` | Cross-platform prototype with screen readers and IME |
| CEF patch lag and rebuild time | `UNVERIFIED` | Measured rebuild and backport exercise |
| Codec patent and distribution licensing | `BLOCKED` | Legal review and licensing decision |
| DRM agreements | `BLOCKED` | Commercial agreements and provisioning |
| Project license | `PROPOSED` | Owner approval (ADR-0008) |
| Audio plugin sandbox level | `UNVERIFIED` | Platform prototype and threat review |
| Packaging tool licenses | `UNVERIFIED` | Verify WiX, AppImage, Flatpak, and signing tool licenses |
| CI provider and remote | `BLOCKED` | Remote authorization and CI configuration |
| Windows/Linux builds | `BLOCKED` | Workspace and platform CI |

## 13. Sources

All sources were accessed on 2026-11-07 unless stated otherwise.

1. egui changelog and AccessKit dependency. https://github.com/emilk/egui/blob/master/CHANGELOG.md
2. egui and eframe on crates.io. https://crates.io/crates/egui
3. Iced 0.14 accessibility issue. https://github.com/iced-rs/iced/issues/552
4. Slint licensing. https://slint.dev/pricing
5. Slint software license. https://slint.dev/agreements/slint-software-license.pdf
6. Qt licensing. https://doc.qt.io/qt-6/licensing.html
7. Qt open-source obligations. https://www.qt.io/development/open-source-lgpl-obligations
8. Electron security. https://www.electronjs.org/docs/latest/tutorial/security
9. wgpu on crates.io. https://crates.io/crates/wgpu
10. Vello repository and license. https://github.com/linebender/vello
11. Vello GPU crate. https://crates.io/crates/vello_gpu
12. CEF documentation. https://chromiumembedded.github.io/cef/
13. FFmpeg legal page. https://www.ffmpeg.org/legal.html
14. VST 3 licensing. https://steinbergmedia.github.io/vst3_dev_portal/pages/VST%2B3%2BLicensing/VST3%2BLicense.html
15. CLAP repository. https://github.com/free-audio/clap
16. LV2 specification. https://lv2plug.in/
17. SQLite copyright. https://www.sqlite.org/copyright.html
18. RustCrypto repositories. https://github.com/RustCrypto
19. WiX Toolset. https://wixtoolset.org/
20. Apple notarization. https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
21. Flatpak documentation. https://docs.flatpak.org/
22. AppImage documentation. https://appimage.org/
23. The Update Framework specification. https://theupdateframework.github.io/specification/latest/
24. minisign. https://jedisct1.github.io/minisign/
25. AccessKit. https://accesskit.dev/
