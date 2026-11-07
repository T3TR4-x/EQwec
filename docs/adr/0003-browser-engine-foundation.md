# ADR-0003: Browser engine foundation

- **Status:** ACCEPTED (with a mandatory prototype and legal gate)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0001, `docs/research/BROWSER-ENGINE-FEASIBILITY.md`, `docs/security/THREAT-MODEL.md`, ADR-0004

## 1. Context

EQwec Browser must render the modern web, isolate untrusted content, support profiles and sessions, and receive security patches on a credible schedule. EQwec does not intend to rewrite Chromium in Rust. The question is which engine foundation can be responsibly shipped and maintained by this project, starting from a bootstrap phase with no browser prototype.

The choice is expensive to reverse. It determines the process model, sandbox, extension surface, codec/DRM path, packaging size, update pipeline, and the security operating model.

## 2. Requirements

- Modern web compatibility: HTML, CSS, JavaScript, media, authentication, and common web applications.
- Cross-platform behavior on Windows, macOS, and Linux, with x86_64 and ARM64 where feasible.
- Per-profile isolation and Chromium-class site isolation and sandboxing.
- A Rust-owned application layer for windows, tabs, sessions, profiles, downloads, permissions, and policy.
- A narrow, auditable FFI boundary with no engine pointers in persisted state.
- A security-update path with a defined patch SLA, rebuild procedure, rollback, and staged rollout.
- A credible codec and DRM decision, or an explicit product limitation.
- A packaging and update story that does not require disabling platform security controls.
- A fallback that preserves the Rust application contract if the primary engine fails a gate.
- No Chrome Web Store parity claim unless the engine and product actually support it.

## 3. Alternatives considered

### A. Chromium Embedded Framework (CEF)

Mature BSD-licensed embedding framework with a stable C++ API, binary distributions, release branches tied to Chromium milestones, multi-process architecture, sandbox support, and an active Rust binding. Tracks even Chromium milestones, creates a branch roughly every four weeks, and publishes stable, extended, and LTS lines. Stable security fixes arrive about two weeks after Chromium stable; extended about eight weeks; LTS for roughly nine months. Branch updates are manual, so a CEF build can lag the corresponding Chromium fix.

Weaknesses: not a browser product; no built-in tabs, sessions, downloads, or permissions; large binary; extension surface is not the public Chrome extension API; proprietary codecs disabled by default; DRM requires build and licensing work; accessibility needs platform validation.

### B. Maintained Chromium fork

Maximum control, including below the embedding API, proprietary codecs, DRM, and a custom update channel. Brave demonstrates the model: MPL-2.0, roughly two-week major releases, and a public rebase process. The cost is a permanent browser-platform organization: rebase engineers, release engineers, crash triage, QA, and security response on every Chromium milestone. Arc is a cautionary reference; its proprietary model is being sunset on 2026-12-22. Rejected for the initial phase.

### C. Platform webviews

WebView2 on Windows and WKWebView on macOS inherit OS/Edge updates and reduce installer size. WebView2 is Chromium-based and supports a fixed-version runtime; WKWebView is WebKit-based and inherits OS updates. They fragment behavior across platforms, reduce control over engine revision, codecs, DRM, extensions, and process policy, and require a separate Linux decision. Viable fallback, not a cross-platform primary.

### D. Servo

MPL-2.0 Rust-native engine hosted by the Linux Foundation Europe. The `servo` crate is at 0.7.0 (published 2026-10-05). The Servo Book describes webview consumability as ongoing. Promising long-term, but not ready for a general-purpose browser with media, DRM, extensions, and accessibility requirements. Watch item, not the initial foundation.

### E. Electron

MIT, Chromium-based, but its own security documentation states that Electron is not a web browser and is unsuitable for arbitrary untrusted content. A browser built on Electron would have to reimplement Chromium's browser-level isolation, navigation policy, permission model, and update path. Rejected.

## 4. Evidence

- CEF is BSD-licensed, based on Chromium, and provides release branches, binary distributions, and a multi-process architecture. Stable branch 156 (October 2026); LTS branch 150. Branch updates are manual.
- Chromium moved to a two-week release cycle; CEF tracks even milestones and backports security fixes to stable, extended, and LTS lines.
- The `cef` crate (tauri-apps/cef-rs) is an actively maintained FFI wrapper, versioned against CEF branches; the current line at research time was 156.3.1+156.0.15 under Apache-2.0/MIT.
- WebView2 is Windows-only; Evergreen updates with Edge; Fixed Version is documented as larger than 250 MB; PlayReady is supported and Widevine is not.
- WKWebView is Apple-only; WebExtensions are supported through `WKWebExtensionController` on macOS 15.4+ and iOS 18.4+; FairPlay is Apple's DRM.
- Servo is MPL-2.0; the `servo` crate is 0.7.0; the Servo Book points embedders toward `tauri-runtime-verso` or `servo-gtk` and describes webview consumability as ongoing.
- Electron's security documentation explicitly warns against arbitrary untrusted content.
- Arc's official sunset date is 2026-12-22.
- Brave is MPL-2.0 and rebases on Chromium; its release process shows the maintenance cost of a fork.

## 5. Decision

### 5.1 Primary engine

Adopt CEF as the primary engine foundation, behind a narrow Rust boundary in `crates/eqwec-browser-engine`.

- **Initial pin:** CEF 156.x stable line and the matching `cef` crate line. Record the exact CEF build, crate version, and build flags at prototype start; do not treat this ADR's research-time values as permanently pinned.
- **Process model:** the Rust application process owns windows, native UI, profiles, tabs, sessions, downloads, permissions, and engine supervision. CEF owns browser, renderer, GPU, network, utility, and extension processes.
- **Profile isolation:** one CEF user-data directory and engine process tree per active profile; explicit teardown and crash recovery. Do not share an engine user-data directory across profiles.
- **FFI boundary:** opaque handle newtypes, C-compatible ABI, typed errors, versioned capability handshake, panic containment before the boundary, documented thread affinity. No engine pointer or internal struct in session/profile data.
- **Sandbox and site isolation:** preserve Chromium's sandbox and site isolation. Verify effective flags at runtime; an accidental disablement is a release blocker.
- **Tabs and sessions:** tabs are Rust-owned records; session restoration rebuilds metadata first and instantiates engine views lazily. Tab sleeping discards the view but keeps the record.
- **Downloads:** Rust-owned records; validate destination, filename, size, and type; never auto-open a download.
- **Accessibility:** test the platform assistive-technology matrix; do not claim parity until measured.
- **Extensions:** do not claim Chrome Web Store parity. CEF does not expose the public Chrome extension API. If extensions are supported, define a curated capability model and validate it in the prototype.

### 5.2 Fallback

If CEF fails a gate or the product scope narrows, use WebView2 on Windows and WKWebView on macOS behind the same Rust engine contract. Accept the losses: no single cross-platform engine behavior, no Chrome extension parity, platform-specific codec/DRM outcomes, less control over process policy and engine revision, and a separate Linux decision (likely WebKitGTK, not evaluated in this ADR).

### 5.3 Rejected for the initial phase

- Chromium fork: requires a permanent browser-platform organization.
- Servo: embedding, media, DRM, and accessibility readiness are not sufficient.
- Electron: wrong security model for a browser.

## 6. Conditions and gates

### 6.1 Prototype gate

Before expanding browser features, a prototype must prove on Windows, macOS, and Linux:

1. Real website rendering and navigation, including JavaScript-heavy sites and media playback.
2. Multiple tabs and at least two isolated profiles.
3. Process isolation: separate browser, renderer, GPU, network, and utility processes; one renderer per site as configured.
4. Sandbox and site-isolation flags verified at runtime.
5. Rust-to-engine integration with typed errors and panic containment.
6. Session persistence and restore after a clean shutdown and after a crash.
7. Download handling with path validation and no auto-open.
8. Accessibility smoke test with platform assistive technology.
9. Installer packaging, signing, and a measured process-tree memory baseline.

### 6.2 Legal gate

Codec and DRM licensing must be resolved before enabling proprietary media. CEF's default binary distributions do not include H.264/AAC. Enabling them requires build flags and patent/distribution review. Widevine requires agreements and provisioning; Linux lacks persistent-license support in CEF (issue #3404). Do not ship DRM in the first release. See `docs/research/LEGAL-LICENSE-REVIEW.md` and ADR-0004.

### 6.3 Security operating model

Before the first public release, name:

- the engine owner and security-response owner;
- the pinned CEF branch and update cadence;
- the patch SLA and emergency rebuild procedure;
- the rollback and staged-rollout procedure;
- the SBOM, license inventory, and binary provenance process;
- the crash-reporting consent model and data-redaction rules;
- the site-isolation and sandbox verification checklist.

The patch SLA is `UNVERIFIED` until measured. Do not publish an hour/day target as fact before the first rebuild exercise.

## 7. Consequences

**Positive**

- Chromium-class web compatibility without rewriting the engine.
- Rust remains the application language; the FFI surface is narrow and owned.
- CEF's process model and sandbox align with the browser threat model.
- A fallback contract preserves tabs, sessions, downloads, and profile policy if the engine changes.

**Negative**

- EQwec owns Chromium security rebuilds and releases; this is a permanent operational burden.
- Installer size is large; updates are EQwec-owned.
- CEF is not a browser product; tabs, sessions, downloads, permissions, and update UX are all EQwec work.
- Extension parity is not available through a public Chrome extension API.
- Proprietary codecs and DRM require legal and build work.
- Accessibility parity is unverified until the prototype runs the platform matrix.

**Security**

- Preserve sandbox and site isolation; never disable them for memory or startup.
- Treat renderer IPC as untrusted; validate version, type, size, origin, and authorization.
- Keep engine handles out of persisted state.
- Define a patch SLA and incident response before release.
- CEF/Chromium third-party notices and codec build flags are release artifacts.

**Performance**

- Measure the whole process tree, not a single process.
- No memory number is claimed here. The 20-30% exploratory target is a hypothesis, not evidence.
- Tab sleeping, lazy restoration, and cache sizing are safe only if session state and isolation are preserved.

**Licensing**

- CEF is BSD-3-Clause; Chromium and bundled components have their own licenses.
- Proprietary codecs, DRM, and platform runtimes require separate review.
- The `cef` crate is permissive; pin and audit it.

**Maintenance**

- CEF branch updates are manual; EQwec must track Chromium advisories and CEF releases.
- A fork would be more control but a much larger permanent organization.
- The fallback is cheaper to maintain but less capable and less uniform.

## 8. Reconsideration criteria

Revisit this ADR if:

- Codec or DRM licensing cannot be secured, collapsing CEF's compatibility advantage.
- Full Chrome extension parity becomes mandatory.
- The security patch lag exceeds the accepted SLA.
- The project cannot fund browser-platform build, signing, sandbox, crash, and update work.
- WebView2 plus WKWebView meets the product bar and Linux is out of scope.
- The prototype fails accessibility, site isolation, DRM, or memory budgets.
- Servo reaches general-purpose embedding readiness with the required media, DRM, and accessibility support.
- A Chromium fork becomes justified by a funded browser-platform team and a product requirement CEF cannot meet.

## 9. Links

- `docs/research/BROWSER-ENGINE-FEASIBILITY.md`
- `docs/security/THREAT-MODEL.md`
- `docs/security/SECURITY-BASELINE.md`
- `docs/architecture/TECHNOLOGY-SELECTION.md`
- ADR-0004 (media and codecs)
- ADR-0010 (plugin and extension trust)
