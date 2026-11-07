# Feasibility Synthesis

| Field | Value |
|---|---|
| Status | Planning synthesis. Feasibility verdicts are conditional judgments, not implementation approval. |
| Date | 2026-11-07 |
| Scope | All eight EQwec products and the shared platform |
| Method | Synthesize `BROWSER-ENGINE-FEASIBILITY.md`, `COMPETITOR-ANALYSIS.md`, `LEGAL-LICENSE-REVIEW.md`, `TECHNOLOGY-SELECTION.md`, and the accepted ADRs against the stated requirements. |
| Evidence standard | `OBSERVED` = stated by an official source; `ASSUMPTION` = planning judgment; `UNVERIFIED` = not confirmed; `BLOCKED` = cannot proceed without a missing prerequisite. |
| Decision owner | Project owner and architecture review |

## 1. Executive summary

The ecosystem is **technically feasible but not yet de-risked**. Every product has a credible implementation path using the selected Rust-first stack, and no product requires an unsolved research breakthrough. The binding constraints are organizational and legal, not algorithmic:

1. **Breadth is the largest risk.** Eight product families, including a browser and a package manager, are planned with only a Phase 2 workspace, one passing three-OS CI run, no installer, and no release artifact. The sequence must be depth-first: Hub, then one creative product, then the rest.
2. **The browser is the highest-risk product.** CEF provides Chromium-class compatibility without a fork, but EQwec becomes responsible for engine rebuilds, security response, codecs, DRM, and a large installer. This is a permanent operating cost, not a one-time integration.
3. **Codec and DRM licensing is unresolved.** H.264, AAC, HEVC, ProRes, and DNxHR cannot ship by default until counsel or an authorized licensing decision exists. AV1, VP9, Opus, FLAC, PNG, JPEG, and WebP provide a royalty-free first-release path.
4. **Audio real-time correctness is unproven.** CPAL and out-of-process CLAP/VST3 hosting are credible, but no EQwec measurement exists. The audio callback contract and plugin crash isolation must be proven before AudioCraft work.
5. **The project license is not selected.** Until ADR-0008 is accepted, redistribution is unauthorized and contribution terms do not exist.
6. **Accessibility and text input are the main GUI unknowns.** `egui` has an AccessKit path, but IME and screen-reader parity across Windows, macOS, and Linux are `UNVERIFIED` until the prototype runs.

**Overall verdict:** feasible with conditions. Proceed to the workspace and Hub, and keep Browser, FilmCraft, EffectCraft, and AudioCraft behind their stated prototype and legal gates.

## 2. Method and limits

This document answers four questions for each product:

- Is there a credible implementation path with the selected stack?
- What must be proven before product work starts?
- What can block or invalidate the path?
- What is the smallest first proof?

Limits:

- No EQwec product exists. No performance, compatibility, or reliability claim in this document is measured.
- Competitor behavior is taken from official documentation only, as recorded in `COMPETITOR-ANALYSIS.md`.
- Legal conclusions are `BLOCKED` pending counsel; this document is not legal advice.
- Local verification was performed on macOS on Apple silicon. GitHub Actions also runs the workspace tests on Ubuntu and Windows, but no product runtime has been accepted on those platforms.
- Numeric targets in `PERFORMANCE-STRATEGY.md` are hypotheses until a benchmark harness exists.

## 3. Cross-cutting findings

### 3.1 Credible common foundation

`OBSERVED`/`ASSUMPTION`: The accepted ADRs define one coherent stack: Rust, `egui`/`eframe`, `wgpu`, `kurbo`/`lyon`/`cosmic-text`, FFmpeg under LGPL-only, CPAL, CLAP-first plugin hosting, a versioned project bundle, and signed native packages. Each choice has a documented fallback. This reduces integration risk compared with per-product stacks.

The common foundation is now a workspace with three crates and a non-published fixture. The next evidence must be the Hub trust chain: signed manifest, verified download, transactional install/update/rollback, and fault injection. Workspace CI now runs on all three platforms; product-runtime acceptance on Windows and Linux remains to be built.

### 3.2 Trust and update chain

`OBSERVED`: Competitor launchers and stores compete on convenience; several rely on platform stores or vendor accounts. EQwec's differentiator is a local-first, signed, transactional install/update chain with offline support and rollback.

`ASSUMPTION`: This is feasible and is the right first product because it is smaller than a browser or editor and its security model is well bounded. It also creates the signing and release infrastructure every other product needs.

### 3.3 Data ownership and interchange

`OBSERVED`: Professional creative tools rely on proprietary project formats and vendor ecosystems. Users face lock-in, subscription pressure, and uncertain long-term access.

`ASSUMPTION`: A versioned, documented, inspectable project bundle with content-addressed assets and a guaranteed local export path is a credible differentiator. Cross-product exchange must use explicit packages with provenance and fidelity declarations, not live mutation of foreign documents.

### 3.4 Legal exposure

`OBSERVED`: Copyright licenses do not grant patent licenses. H.264, AAC, HEVC, ProRes, and DNxHR carry patent or vendor terms; DRM requires agreements. GPL components such as `libx264` and `libx265` cannot ship in a proprietary build.

`BLOCKED`: Codec, DRM, and some format-interoperability decisions require counsel. The default release must stay on royalty-free formats.

### 3.5 Accessibility and localization

`OBSERVED`: `egui` depends on AccessKit, but platform screen-reader parity and IME behavior are not established by the dependency alone.

`UNVERIFIED`: Cross-platform accessibility and Ukrainian/Slovak text input. These are release criteria and must be proven in the GUI prototype, not deferred.

### 3.6 Maintenance capacity

`ASSUMPTION`: The largest non-technical risk is sustaining eight products plus shared infrastructure. A browser alone implies a permanent security-response function. The roadmap must assume a small team and sequence work so each product reaches a defensible vertical slice before the next begins.

## 4. Per-product feasibility

### 4.1 EQwec Hub

**Verdict:** feasible; recommended first product.

**Credible path:** native `egui` shell, signed manifest, local installation registry, transactional staging and activation, rollback, repair, uninstall, offline bundles, redacted diagnostics. Uses ADR-0006 storage discipline and ADR-0007 signing.

**Key risks:**

- Signing key custody and revocation are organizational, not code.
- Platform installers differ (MSI, `.app`/DMG, `tar.zst`/Flatpak) and each needs signing and testing.
- A launcher that installs other applications conflicts with store sandbox models; document the limitation.
- Interrupted-update recovery must be proven with fault injection.

**Blockers:** signing keys, certificates, notarization account, and release channel. All release steps `BLOCKED`.

**First proof:** install, verify, launch, update, kill mid-update, recover, repair, roll back, and uninstall a fixture application on macOS, then Windows and Linux.

### 4.2 EQwec Browser

**Verdict:** feasible with CEF, but the highest-risk and highest-cost product.

**Credible path:** CEF behind a narrow Rust boundary, Chromium sandbox and site isolation preserved, Rust-owned tabs/profiles/sessions/downloads/permissions, per-profile engine process trees, signed updates, platform-webview fallback behind the same contract.

**Key risks:**

- Permanent engine rebuild and security-response burden; CEF branch updates are manual.
- Installer size and update bandwidth.
- Proprietary codecs disabled by default; DRM requires agreements and provisioning.
- No public Chrome extension API; no Web Store parity claim.
- Accessibility parity and memory behavior are `UNVERIFIED`.
- Linux DRM support in CEF is limited.

**Blockers:** browser prototype gate and codec/DRM legal gate. The prototype does not exist.

**First proof:** render real sites, two isolated profiles, multiple tabs, verified sandbox/site isolation, session restore after crash, download path validation, signed installer, measured process-tree memory.

**Reconsideration:** if codec/DRM cannot be secured and extension parity is mandatory, narrow the product or switch to platform webviews.

### 4.3 PhotoCraft

**Verdict:** feasible; large scope, moderate technical risk.

**Credible path:** linear working space, bounded tiled processing, layer/mask/adjustment model, `image`/`exr`/LibRaw decode, LittleCMS ICC, GPU compositing through `wgpu`, versioned project bundle, non-destructive history.

**Key risks:**

- RAW camera coverage and color accuracy are deep, long-tail problems.
- GPU/CPU parity and color management are easy to get subtly wrong.
- Large-document memory and tile cache behavior need measurement.
- PSD and other proprietary interchange fidelity is `BLOCKED` without legal and technical review.

**Blockers:** GUI prototype, media/color legal review (ADR-0004), and a document model not yet designed.

**First proof:** open a representative RAW and layered document, apply non-destructive adjustments, save, reopen, and export with color round-trip within a defined tolerance.

### 4.4 VectorCraft

**Verdict:** feasible; best candidate for the first complete creative application.

**Credible path:** `kurbo` geometry, `lyon` tessellation, `cosmic-text` shaping, `wgpu` rendering, SVG import/export through a bounded parser, path editing and boolean operations with a CPU reference implementation for correctness.

**Key risks:**

- Boolean robustness and numerical precision are hard.
- SVG feature coverage and text-on-path fidelity vary.
- Large-document hit-testing and snapping performance need measurement.
- Proprietary format compatibility (AI, CDR) is not claimed.

**Blockers:** GUI prototype and document model. No vector engine exists.

**First proof:** create paths, edit nodes, apply fills/strokes/gradients, perform booleans, import and export SVG, save/reopen without loss.

**Why first:** narrower than photo or video, testable with deterministic geometry, and it exercises the shared document, undo, rendering, and export foundations.

### 4.5 FilmCraft

**Verdict:** feasible but expensive; depends on media and render infrastructure.

**Credible path:** FFmpeg LGPL-only decode, subprocess export first, platform hardware codecs, proxy/cache pipeline, timeline model, A/V sync against a shared transport clock, OpenTimelineIO interchange, versioned project bundle with external media references.

**Key risks:**

- Frame-accurate seek, scrub, and A/V sync across codecs and containers.
- Proxy/cache invalidation and disk footprint.
- Export determinism and hardware/software parity.
- Codec licensing and patent exposure.
- Long-tail camera/media formats.

**Blockers:** media/color legal gate, render graph design, benchmark harness. No media pipeline exists.

**First proof:** import representative media, scrub frame-accurately, cut and play a short timeline with sync, export through the subprocess path, reopen the project.

### 4.6 EffectCraft

**Verdict:** feasible; depends on FilmCraft interchange and a compositing graph.

**Credible path:** node/layer graph, keyframes, GPU compositing through `wgpu`, deterministic cache invalidation, FilmCraft interchange, bounded expression or script model if any.

**Key risks:**

- Graph evaluation, caching, and invalidation correctness.
- GPU/CPU rendering parity and color management.
- Effect plugin trust and sandboxing (ADR-0010).
- Scope creep into a general VFX platform.
- Deterministic render and memory behavior on large compositions.

**Blockers:** FilmCraft contracts, compositing graph design, GUI prototype. No compositor exists.

**First proof:** a small node graph with keyframed animation, cached playback, deterministic render, and round-trip with FilmCraft.

### 4.7 CraftAnything

**Verdict:** feasible; should follow stable product contracts, not precede them.

**Credible path:** workspace manifest, content-addressed asset references, canonical document ownership by each product, provenance records, explicit open/return workflows over versioned IPC.

**Key risks:**

- Becoming a universal editor that duplicates product internals.
- Cross-product version skew and broken references.
- Provenance and C2PA-style claims require careful legal and technical validation.
- Asset deduplication and cache coherence across products.

**Blockers:** stable product document contracts and IPC. No product contract exists.

**First proof:** open a document from two products, reference a shared asset, edit in the owning product, return it, and detect a broken link without data loss.

### 4.8 AudioCraft

**Verdict:** feasible; real-time correctness is the gating risk.

**Credible path:** CPAL first, platform backends behind a project-owned trait, allocation-free callback, bounded lock-free queues, CLAP-first and VST3-second out-of-process hosting, deterministic offline render, versioned project format.

**Key risks:**

- Xruns and latency on real hardware and varied buffer sizes.
- Plugin crash isolation and scan safety.
- Out-of-process IPC latency in the audio path.
- Sample-rate conversion, device hot-plug, and channel layouts.
- Plugin state compatibility and missing-plugin recovery.

**Blockers:** real-time prototype gate, GUI prototype, plugin trust implementation. No audio engine exists.

**First proof:** 16 audio + 4 MIDI tracks at a declared buffer size with zero xruns in a defined stress test, no allocation in the callback, CLAP/VST3 load and crash isolation, deterministic offline render.

## 5. Risk matrix

| Risk | Likelihood | Impact | Products | Mitigation | State |
|---|---|---|---|---|---|
| Breadth before depth | High | High | All | Depth-first sequence; one vertical slice at a time | `ASSUMPTION` |
| Browser engine maintenance | High | High | Browser | CEF pin, patch SLA, rebuild exercise, fallback contract | `UNVERIFIED` |
| Codec/patent licensing | High | High | Browser, FilmCraft, EffectCraft, AudioCraft | Royalty-free default; legal gate; counsel | `BLOCKED` |
| DRM agreements | Medium | High | Browser | No DRM in first release; separate ADR | `BLOCKED` |
| Real-time audio failure | Medium | High | AudioCraft, FilmCraft | Prototype gate; instrumentation; platform backends | `UNVERIFIED` |
| Accessibility/IME gap | Medium | High | All GUI products | GUI prototype with screen readers and IME | `UNVERIFIED` |
| Project license absent | Certain | High | All | ADR-0008 owner approval | `PROPOSED` |
| Signing/key custody absent | Certain | High | All | Root/release keys, revocation, release gate | `BLOCKED` |
| Format interoperability claims | Medium | Medium | Photo, Vector, Film, Effect | Documented fidelity; no parity claims; fixtures | `UNVERIFIED` |
| Plugin security | Medium | High | AudioCraft, Browser, CraftAnything | Out-of-process hosting, consent, revocation | `UNVERIFIED` |
| Scope/feature parity trap | High | High | All | Vertical slices; explicit non-goals | `ASSUMPTION` |
| Team capacity | High | High | All | Sequence, reuse, delete non-essential scope | `ASSUMPTION` |
| Platform divergence | Medium | Medium | All | CI on Windows/macOS/Linux; per-platform owners | `BLOCKED` |
| Performance regression | Medium | Medium | All | Benchmark harness; budgets before optimization | `BLOCKED` |

## 6. Recommended sequence

1. **Phase 2 — Workspace baseline (in progress).** Cargo workspace, pinned toolchain, `eqwec-core`, CI configuration, local advisory/license checks. `dev-hub` next; `production` only after a verified release. No product feature.
2. **Phase 3 — Hub MVP.** Signed manifest, registry, transactional install/update/rollback/repair/uninstall, offline bundles, diagnostics. First real trust chain.
3. **Phase 4 — Browser prototype.** CEF integration, isolation, session restore, packaging, measurements. Do not expand features before the gate.
4. **Phase 5 — Creative foundation.** Document lifecycle, undo, assets, project serialization, migrations, design tokens.
5. **Phase 6 — VectorCraft first complete creative product.** Smallest credible vertical slice with deterministic tests.
6. **Phase 7 — PhotoCraft, then FilmCraft, AudioCraft, EffectCraft, CraftAnything** in dependency order, each with its own gate.

This order is a recommendation, not a commitment. It can change only with recorded evidence and owner approval.

## 7. Conditions for each gate

| Gate | Condition | Current state |
|---|---|---|
| Skills | 10 project skills valid, discoverable, trigger-tested | `PASS` for bootstrap scope |
| Documentation | Required research, architecture, security, performance, and development docs exist and pass structural checks | `PASS`; `check-docs.py --gate` returned 0 failures on 2026-11-07 |
| Architecture | ADRs for GUI, browser, media, audio, storage, packaging, license, diagnostics, and plugin trust are recorded | `PASS` with open gates inside ADRs |
| Security | Threat model and baseline exist; product-specific boundaries reviewed | `PASS` for planning baseline; runtime `BLOCKED` |
| Workspace | Cargo workspace builds, tests, lints, and checks advisories/licenses | `PASS` locally on macOS and in GitHub Actions run `42389844319` on macOS, Ubuntu, and Windows |
| Hub release | Install/update/rollback/repair/uninstall verified on claimed platforms | `BLOCKED` — no implementation |
| Browser prototype | Sandbox, isolation, session restore, packaging, measurements | `BLOCKED` — no prototype |
| Media legal | Codec/patent and DRM decisions recorded | `BLOCKED` — counsel required |
| GUI prototype | Accessibility, IME, performance on three platforms | `BLOCKED` — no prototype |
| Audio prototype | Real-time, plugin isolation, deterministic render | `BLOCKED` — no prototype |

## 8. Plan status

| Area | State | Evidence |
|---|---|---|
| Environment audit | `PASS` | macOS 26.7.2 arm64; Rust 1.99.1; Cargo 1.99.1; Node 26.10.0; OpenCode 2.2.5 |
| Project skills | `PASS` | `scripts/verify-skills.py` 10/10; schema validation 10/10; trigger evidence in `SKILLS-AUDIT.md` |
| Research | `PASS` | Browser, competitor, legal, and this synthesis exist |
| Architecture | `PASS` with open decisions | ADR-0001 through ADR-0010 recorded; 0004 and 0008 remain `PROPOSED` |
| Security baseline | `PASS` for planning | Threat model and baseline exist; runtime verification `BLOCKED` |
| Performance | `PASS` for methodology | Strategy exists; no benchmark `BLOCKED` |
| Workspace | `PASS` locally and in CI | Cargo workspace; 163 tests; fmt, clippy, build, audit, and deny pass; GitHub Actions run `42389844319` passed on macOS, Ubuntu, and Windows |
| Product implementation | `BLOCKED` | Gates above must pass first |
| Windows/Linux verification | `PASS` for workspace tests; product acceptance `BLOCKED` | CI runs the workspace tests; no Hub application or installer exists to accept |
| Release | `BLOCKED` | No keys, signing, installer, or channel |

## 9. Open decisions

| Decision | Owner | Blocking |
|---|---|---|
| Accept ADR-0008 project license | Project owner | Distribution and contribution |
| Accept or amend ADR-0004 codec policy | Owner and counsel | Browser and media features |
| Choose FFmpeg linkage model for first release | Engineering and counsel | FilmCraft/EffectCraft export |
| Approve signing key custody | Owner | Any release |
| Select GUI prototype platforms and hardware profiles | Engineering | GUI gate |
| Define audio latency and xrun budgets | Engineering | Audio gate |
| Decide browser extension scope | Product and architecture | Browser feature set |

## 10. Related documents

- `docs/research/BROWSER-ENGINE-FEASIBILITY.md`
- `docs/research/COMPETITOR-ANALYSIS.md`
- `docs/research/LEGAL-LICENSE-REVIEW.md`
- `docs/architecture/TECHNOLOGY-SELECTION.md`
- `docs/architecture/SYSTEM-ARCHITECTURE.md`
- `docs/architecture/DEPENDENCY-MAP.md`
- `docs/security/THREAT-MODEL.md`
- `docs/security/SECURITY-BASELINE.md`
- `docs/performance/PERFORMANCE-STRATEGY.md`
- `docs/development/ROADMAP.md`
- `docs/development/QUALITY-GATES.md`
- `docs/product-specs/eqwec-hub.md`
- `docs/adr/README.md`

## 11. Sources

All sources were accessed on 2026-11-07. This document synthesizes primary-source research recorded in the related documents; it does not introduce new vendor claims.

1. CEF documentation and release branches. https://chromiumembedded.github.io/cef/
2. Chromium release schedule. https://www.chromium.org/developers/calendar/
3. FFmpeg legal and license page. https://www.ffmpeg.org/legal.html
4. WebView2 documentation. https://learn.microsoft.com/microsoft-edge/webview2/
5. WKWebView documentation. https://developer.apple.com/documentation/webkit/wkwebview
6. egui repository and changelog. https://github.com/emilk/egui
7. AccessKit. https://accesskit.dev/
8. wgpu. https://wgpu.rs/
9. CPAL. https://github.com/RustAudio/cpal
10. CLAP. https://github.com/free-audio/clap
11. VST 3 licensing. https://steinbergmedia.github.io/vst3_dev_portal/pages/VST%2B3%2BLicensing/VST3%2BLicense.html
12. SQLite copyright. https://www.sqlite.org/copyright.html
13. The Update Framework. https://theupdateframework.github.io/specification/latest/
14. Apple notarization. https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
15. WiX Toolset. https://wixtoolset.org/
16. Flatpak documentation. https://docs.flatpak.org/
