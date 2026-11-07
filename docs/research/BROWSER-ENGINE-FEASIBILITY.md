# Browser Engine Feasibility

| Field | Value |
|---|---|
| Status | Research complete for planning; not an ADR and not an implementation approval |
| Date | 2026-11-07 |
| Scope | Engine foundation for EQwec Browser only |
| Decision owner | Pending architecture review |
| Follow-up | A browser engine ADR is required before any implementation |

## 1. Executive summary

EQwec Browser needs Chromium-class web compatibility, a Rust-owned application layer, per-profile process isolation, a credible security update path, and a packaging story that works on Windows, macOS, and Linux. No candidate satisfies every requirement without trade-offs.

The Chromium Embedded Framework (CEF) is the strongest primary candidate. It is a mature, BSD-licensed embedding framework with a cross-platform C++ API, a multi-process architecture, sandbox support, documented release branches, and an active Rust binding. It keeps browser application logic in Rust while delegating rendering and web-platform behavior to Chromium. Its costs are equally clear: EQwec becomes responsible for rebuilding and shipping Chromium security fixes, the binary is large, arbitrary Chrome extension parity is unsupported, and proprietary codecs and DRM require explicit build and licensing decisions.

Platform webviews are the strongest fallback. WebView2 on Windows and WKWebView on macOS inherit operating-system or Edge update channels and reduce installer size, but they fragment behavior across platforms, weaken control over codecs, DRM, extensions, and process policy, and do not provide one cross-platform engine contract. They are a viable product tier only if EQwec accepts platform-specific capability differences.

The remaining candidates are not suitable as the initial foundation:

- A maintained Chromium fork offers maximum control at the cost of a permanent rebase, toolchain, QA, and security organization. Brave demonstrates that the model works, but it is a browser-company-scale commitment. Arc is a cautionary reference: its proprietary model is being sunset on 2026-12-22.
- Servo is promising and improving, but its own documentation still describes webview embedding as maturing. It cannot yet be recommended for general-purpose browsing with codec, DRM, extension, and accessibility requirements.
- Electron is explicitly documented by its maintainers as not a web browser and as unsuitable for arbitrary untrusted content. Using it for a browser would invert its security guidance.

**Recommendation:** adopt CEF as the primary engine, behind a narrow Rust engine boundary, subject to the conditions in section 12. Keep a platform-webview adapter as the funded fallback. Do not start with a Chromium fork, Servo, or Electron. Revisit the decision if any recommendation-breaking condition in section 11 becomes true.

## 2. Question and evidence standard

The question is not "which engine is best in general." It is "which engine can EQwec responsibly ship and maintain for a security-sensitive browser, given the current bootstrap phase and the absence of a prototype."

This document separates three evidence classes:

- **Verified:** stated by an official project, vendor, or API document cited in section 13.
- **Assumption:** a planning judgment that follows from verified facts but is not itself measured.
- **UNVERIFIED:** a claim that could not be confirmed from a primary source or requires a prototype. Unverified claims are excluded from the recommendation.

No numeric performance, memory, or installer measurements are asserted. The repository has no Cargo workspace and no browser prototype, so such measurements are `BLOCKED`. Qualitative ratings are used deliberately; numeric scores without a workload and a baseline would be false precision.

## 3. Evaluation criteria

| Criterion | Why it matters for EQwec Browser |
|---|---|
| Web compatibility | Users expect modern sites, JavaScript frameworks, media, and authentication flows to work |
| Cross-platform parity | Windows, macOS, and Linux are planned targets; divergent behavior raises support and test cost |
| Security update path | A browser is a security product; engine advisories must become patches on a defined cadence |
| Process isolation | Sites, profiles, and renderers must not share trust or memory boundaries |
| Extension support | Product scope is undecided; Chrome Web Store parity would be a hard constraint |
| Codec and DRM control | Video, conferencing, and streaming require H.264/AAC and sometimes Widevine or FairPlay |
| Accessibility | A browser must work with platform assistive technologies |
| Packaging and footprint | Installer size, update mechanics, and enterprise deployment affect adoption |
| Rust integration | EQwec application logic is Rust-owned; the FFI boundary must be narrow and auditable |
| Licensing and legal risk | Engine, codecs, DRM, and notices must be shippable |
| Operational burden | The team must sustain builds, rebases, signing, rollbacks, and incident response |

## 4. Candidate assessment

### 4.1 Chromium Embedded Framework (CEF)

**Verified facts.** CEF is a BSD-licensed open-source project founded in 2008 and based on Chromium. It provides a stable C++ API, binary distributions, release branches tied to Chromium milestones, and a multi-process architecture. The project reports over 100 million installed instances. CEF tracks even Chromium milestones, creates a new release branch roughly every four weeks, and publishes stable, extended, and LTS support lines. As of this writing the stable branch is 156 (October 2026) and the LTS branch is 150. Stable branches receive security fixes about two weeks after Chromium stable, extended branches about eight weeks, and LTS branches for roughly nine months. Branch updates are manual, so a CEF build can lag the corresponding Chromium fix.

**Architecture fit.** CEF separates browser, renderer, GPU, network, and utility processes. Its default configuration creates a renderer process per unique origin. Sandbox setup is platform-specific and documented; recent milestones changed helper requirements on Windows and macOS. The host application owns windows, profiles, policy, and engine supervision, which matches the planned Rust-owned browser process.

**Rust integration.** The `cef` crate (tauri-apps/cef-rs) is actively maintained and versioned against CEF branches; the current line is 156.3.1+156.0.15 under Apache-2.0/MIT. It is an FFI wrapper, not a Rust rewrite of Chromium. All unsafe code must remain inside the planned `eqwec-browser-engine` crate.

**Weaknesses.** CEF is not a browser product. It does not provide EQwec's tabs, sessions, downloads, permissions, or update UX. Its binary distribution is large, its security cadence depends on manual branch updates, and its extension surface is not a public Chrome extension API. Proprietary codecs are disabled in default builds, and DRM requires build and licensing work. Accessibility behavior must be validated per platform and per rendering mode.

**Evidence confidence:** high for architecture and release process; medium for extension and accessibility parity because those depend on product requirements and prototype verification.

### 4.2 Maintained Chromium fork

A fork such as Brave's Chromium tree provides the deepest control: EQwec could change browser behavior below the embedding API, ship its own update channel, and integrate proprietary codecs and DRM directly. Brave is the useful architectural reference. It is MPL-2.0, ships major releases roughly every two weeks, and aims to deliver Chromium security fixes within about 24 hours. Its public rebase documentation also shows the real cost: manual patch-conflict resolution, toolchain rebuilds, QA, and security review on every Chromium milestone.

That cost is not a one-time integration. It is a permanent browser-platform organization with rebase engineers, release engineers, crash triage, and a security response process. For a bootstrap-phase project with no workspace, no CI, and no browser prototype, starting with a fork would front-load the highest-risk work before product demand is proven.

Arc is a cautionary reference rather than a model. Its official site states that Arc is being sunset on 2026-12-22. The product is proprietary, so its architecture cannot be reused, and its outcome shows that a differentiated browser UX does not by itself guarantee a durable maintenance model.

**Verdict:** rejected for the initial phase. Reconsider only if a product requirement cannot be met by CEF and a funded browser-platform team exists.

### 4.3 WebView2

**Verified facts.** WebView2 is Microsoft's Chromium-based webview for Windows. It offers an Evergreen runtime that updates with Edge and a Fixed Version runtime for controlled deployment; Microsoft documents the Fixed Version as larger than 250 MB. It uses the same process and site-isolation model as Edge, with a process group per user-data folder. Extensions are described as sideload-only, and Microsoft's public documentation is not fully consistent about their availability. WebView2 does not support push notifications, the Web Payment API, or periodic background sync in the same way as a full browser, and Google OAuth is blocked in the embedded context. PlayReady is supported; Widevine is not, and the corresponding request remains open.

**Fit.** WebView2 is an excellent Windows fallback. It reduces installer size with Evergreen, inherits Microsoft's security updates, and has a maintained Rust binding (`webview2-com` 0.39.1, MIT). It is not a cross-platform primary because it does not exist on macOS or Linux. It also gives EQwec less control over engine revision, process policy, codecs, and DRM than CEF.

**Evidence confidence:** high for Windows-only scope and distribution models; medium for extension availability because the documentation conflicts; high for the Widevine gap.

### 4.4 WKWebView

**Verified facts.** WKWebView is Apple's WebKit-based webview on macOS and iOS. WebExtensions are supported through `WKWebExtensionController` on macOS 15.4+ and iOS 18.4+. WebKit's own site-isolation documentation described the effort as still at "step 2" as of January 2025, which means WebKit's isolation model is not equivalent to Chromium's. FairPlay Streaming is Apple's DRM system.

**Fit.** WKWebView is the natural macOS fallback. It inherits OS updates, uses the system WebKit, and integrates with Apple accessibility and platform conventions. It is not a cross-platform primary: there is no Windows or Linux implementation, engine behavior tracks the OS version, and codec/DRM control is Apple's. A Rust binding exists (`objc2-web-kit` 0.3.2), but the application would still need a separate Windows engine and a separate Linux decision.

**Evidence confidence:** high for platform scope and WebExtension availability; medium for site-isolation parity because the public documentation is a status report, not a guarantee.

### 4.5 Servo

**Verified facts.** Servo is an MPL-2.0 browser engine hosted by the Linux Foundation Europe. The `servo` crate is at 0.7.0, published 2026-10-05. The Servo Book states that webview consumability is still ongoing and points embedders toward `tauri-runtime-verso` or `servo-gtk`. Servo publishes Web Platform Tests results, but this document makes no numeric compatibility claim.

**Fit.** Servo is the most credible independent engine for a long-term differentiated browser, and it is Rust-native, which aligns with EQwec's language strategy. It is not ready to be the primary engine for a general-purpose browser that must handle arbitrary sites, proprietary media, DRM, extensions, and platform accessibility. Its embedding API is still maturing, and the ecosystem around it is smaller than Chromium's.

**Verdict:** watch item, not the initial foundation. Re-evaluate when a stable embedding API, broad site compatibility, media/DRM support, and accessibility evidence exist.

### 4.6 Electron

**Verified facts.** Electron's own security documentation states that "Electron is not a web browser" and that displaying arbitrary untrusted content is a severe security risk; it recommends using a real browser for that use case. Electron ships a new major every eight weeks, supports the latest three majors, and aligns with even Chromium versions. It is MIT-licensed.

**Fit.** Electron is designed for applications that control their own content, not for browsing the open web. A browser built on Electron would have to reimplement Chromium's browser-level isolation, navigation policy, permission model, and update path while carrying Electron's own release cadence. That is strictly worse than CEF for this use case.

**Verdict:** rejected.

## 5. Decision matrix

Ratings are qualitative and reflect the criteria in section 3. "Confidence" reflects the strength of the cited evidence, not the quality of the engine.

| Candidate | Web compatibility | Cross-platform parity | Security update path | Process isolation | Extension parity | Codec/DRM control | Packaging footprint | Rust integration | Operational burden | Confidence | Role |
|---|---|---|---|---|---|---|---|---|---|---|---|
| CEF | Strong | Strong | Adequate, EQwec-owned | Strong | Weak | Configurable, legal work | Large | Good via `cef` crate | High | High | Primary |
| Chromium fork | Strong | Strong | Strong only with a browser team | Strong | Strong if built | Full control, full liability | Large | Custom | Very high | Medium | Not initial |
| WebView2 | Strong on Windows | Windows only | OS/Edge-managed | Strong | Weak/conflicting | PlayReady only; no Widevine | Small Evergreen; large fixed | Good via `webview2-com` | Medium | High | Windows fallback |
| WKWebView | Strong on Apple platforms | Apple only | OS-managed | WebKit-specific | Moderate on new OS versions | FairPlay | OS-provided | Good via `objc2-web-kit` | Medium | Medium | macOS fallback |
| Servo | Weak for general web | Limited | EQwec-owned | Improving | Weak | Limited/UNVERIFIED | Small potential | Native Rust | High | Medium | Watch |
| Electron | Strong | Strong | Electron-managed | Chromium-based, app-dependent | Weak | Chromium build-dependent | Large | Good | Medium | High | Rejected |

The matrix does not rank CEF as cheap. It ranks CEF as the only candidate that combines cross-platform Chromium compatibility, a documented embedding API, and a workable Rust boundary. The fallback wins on update cost and loses on parity and control.

## 6. Architecture implications

### 6.1 Process and profile model

The browser application process is Rust-owned. It owns windows, native UI, profile selection, tab and session state, download policy, permission policy, and engine supervision. CEF owns the browser, renderer, GPU, network, and utility processes; EQwec observes lifecycle and enforces policy but does not trust renderer state.

A profile is the unit of isolation. Concurrent profiles must not share a CEF user-data directory or an engine process tree. The planned default is one engine process tree per active profile, with explicit teardown and crash recovery.

### 6.2 Rust boundary

`crates/eqwec-browser-engine` is the only crate allowed to contain engine FFI. It should expose opaque handle newtypes, a C-compatible ABI, typed errors, and a versioned capability handshake. Panics must be caught before crossing the boundary. Thread affinity between the CEF UI thread and I/O threads must be documented and enforced. No engine pointer or internal struct may be persisted in session or profile data.

The `cef` crate reduces wrapper work but does not remove the need to own the boundary. A pinned crate version, a pinned CEF branch, and a build-flag manifest are part of the engine contract.

### 6.3 Sandbox and site isolation

CEF sandbox setup is platform-specific and must be shipped with the application, including helper executables and libraries. Recent CEF milestones changed helper requirements on Windows and macOS, so packaging tests must run per platform and per architecture. Site isolation is inherited from Chromium, and the default configuration creates a renderer per unique origin; however, wrapper projects have historically disabled site isolation with command-line switches. EQwec must verify the effective flags at runtime and treat any accidental disablement as a release blocker.

### 6.4 Tabs, sessions, and downloads

A tab is a Rust-owned record with a stable tab ID, profile ID, URL, title, favicon, loading state, history index, and engine view handle. Session restoration rebuilds the window skeleton and tab metadata first, then instantiates engine views lazily. Tab sleeping discards the engine view but keeps the tab record. Downloads are Rust-owned records; the engine transfers bytes, while Rust validates destination, filename, size, and type and never auto-opens a download.

### 6.5 Accessibility and native integration

Accessibility is a release criterion, not a polish item. CEF supports windowed and windowless rendering, and its API exposes an accessibility handler for windowless browsers. The exact behavior of platform screen readers, high-contrast modes, and keyboard navigation must be tested on Windows, macOS, and Linux. This document does not claim accessibility parity; it is `UNVERIFIED` until a prototype runs the platform assistive-technology matrix.

Native integration also includes file dialogs, drag and drop, notifications, media keys, and protocol handlers. These are Rust-owned surfaces that call into the engine; they must not be implemented as page JavaScript with elevated privileges.

## 7. Security maintenance

### 7.1 CEF cadence and patch lag

Chromium now ships milestones every two weeks, with weekly security releases between milestones. CEF tracks even milestones, creates a branch roughly every four weeks, and backports security fixes to stable, extended, and LTS lines. The published cadence is approximately two weeks for stable, eight weeks for extended, and nine months for LTS. Branch updates are manual, so the real lag depends on CEF maintainers and on EQwec's rebuild pipeline.

This is the central security risk of the CEF recommendation. EQwec must define a patch SLA, monitor Chromium advisories and CEF releases, maintain an emergency rebuild path, and test rollback. The SLA must be measured after the first release; until then, any specific hour or day target is an assumption, not evidence.

### 7.2 Platform webviews

WebView2 Evergreen inherits Edge updates, but the host application must release the WebView2 environment during update and may need a restart. Fixed Version shifts update responsibility to EQwec and adds a large runtime to the installer. WKWebView inherits OS updates, but the supported OS range becomes a product constraint and security fixes arrive on Apple's schedule, not EQwec's.

### 7.3 Fork and Electron options

A fork gives EQwec control only if EQwec funds the rebase and security organization. Brave's public process shows the scale: every Chromium milestone brings patch conflicts, toolchain changes, QA, and security review. Electron's eight-week major cadence and three-major support window are manageable for an app, but they do not solve the browser-level trust problem.

### 7.4 Required security operating model

Before shipping, the browser ADR must name:

- the engine owner and the security-response owner;
- the pinned CEF branch and the update cadence;
- the patch SLA and the emergency rebuild procedure;
- the rollback and staged-rollout procedure;
- the SBOM, license inventory, and binary provenance process;
- the crash-reporting consent model and data-redaction rules;
- the site-isolation and sandbox verification checklist.

## 8. Codec and DRM

### 8.1 Codecs

Chromium's proprietary codec support is limited to Chrome-branded builds. The GN defaults are `proprietary_codecs=false` and `ffmpeg_branding="Chromium"`, and CEF's default binary distributions do not include H.264 or AAC. Enabling them requires `proprietary_codecs=true` and `ffmpeg_branding="Chrome"`, plus the corresponding patent and distribution licensing. CEF issue #3559 remains open, which confirms that this is not a solved default.

For EQwec, codec support is a product decision with a legal gate. A browser that cannot play common H.264/AAC media will fail basic compatibility tests. A browser that ships those codecs must account for licensing and notices. The build must record the exact flags and the resulting codec set.

### 8.2 DRM

CEF builds can enable Widevine. Streaming licenses work, but persistent licenses require VMP signature files on Windows and macOS, and Linux lacks persistent-license support (CEF issue #3404). WebView2 supports PlayReady but not Widevine (issue #4828). WKWebView uses FairPlay. Servo's DRM story is `UNVERIFIED` and cannot be assumed.

DRM is not a checkbox. It affects build flags, signing, device provisioning, certificate custody, privacy disclosures, and support. A streaming-service requirement may force Widevine or FairPlay and therefore constrain the engine choice.

### 8.3 Legal gate

Codec and DRM licensing, third-party notices, and CEF's BSD obligations require legal review. This review is `BLOCKED` in the current phase. No engine recommendation is final until the license inventory and DRM agreements are approved.

## 9. Packaging, size, and memory

CEF ships the engine with the application, including helper executables, locales, snapshots, and platform sandbox libraries. The installer is therefore large, and the application owns updates, signing, notarization, and rollback. The exact compressed and installed sizes are `BLOCKED` until a prototype build exists.

WebView2 Evergreen keeps the application small but depends on a runtime that Microsoft updates; Fixed Version is documented as larger than 250 MB. WKWebView is OS-provided, so installer size is lower but the minimum OS version becomes a hard constraint.

Memory must be measured as a process tree, not as a single process. The tree includes the Rust app, the CEF browser process, renderers, GPU, network, utility, and any extension hosts. No memory number is claimed here. The earlier exploratory target of a 20-30% reduction is not evidence and must not appear in product or release claims until a reproducible baseline exists.

## 10. Open questions and BLOCKED items

| Item | Status | Needed to unblock |
|---|---|---|
| Launch platforms and architectures | `BLOCKED` | Product decision and support matrix |
| Chrome extension parity requirement | `BLOCKED` | Product scope decision; CEF has no public extension API |
| Codec licensing (H.264/AAC) | `BLOCKED` | Legal review and build-flag decision |
| Widevine/FairPlay licensing and provisioning | `BLOCKED` | Legal and streaming-partner agreements |
| CEF patch lag and rebuild time | `UNVERIFIED` | Measured rebuild and backport exercise |
| Site-isolation behavior in the packaged app | `UNVERIFIED` | Runtime flag and process-tree verification |
| Accessibility parity | `UNVERIFIED` | Platform screen-reader test matrix |
| Installer size and process-tree memory | `BLOCKED` | Prototype build and recorded benchmark |
| WebView2 extension availability | `UNVERIFIED` | Prototype against the current runtime and docs |
| Linux fallback engine | `BLOCKED` | Separate WebKitGTK evaluation if Linux is in scope |
| Servo embedding readiness | `UNVERIFIED` | Stable API and compatibility evidence |
| Update and rollback UX | `BLOCKED` | Architecture decision and release design |

## 11. What would make this recommendation wrong

The CEF recommendation should be reversed or narrowed if any of the following becomes true:

1. **Codec or DRM licensing cannot be secured.** If EQwec cannot legally ship H.264/AAC and the required DRM, CEF's compatibility advantage collapses. Platform webviews or a commercial engine arrangement may be the only viable path.
2. **Full Chrome extension parity is mandatory.** CEF does not expose a public Chrome extension API. If the product requires Web Store parity, CEF is the wrong foundation; a fork or a different product scope is required.
3. **The security patch lag exceeds the accepted SLA.** If EQwec cannot rebuild and ship CEF fixes within the agreed window, the browser is a liability. The fallback is platform webviews with OS-managed updates.
4. **The team cannot fund the browser-platform work.** CEF still requires build, signing, sandbox, crash, and update engineering. If that capacity does not exist, a smaller platform-webview product is more honest than a browser that cannot be patched.
5. **WebView2 plus WKWebView meets the product bar and Linux is out of scope.** In that case the fallback is cheaper, safer, and faster to ship. CEF is justified only by cross-platform parity or engine control that the product actually needs.
6. **The prototype fails accessibility, site isolation, DRM, or memory budgets.** A failed gate is a decision trigger, not a reason to weaken the gate.
7. **Servo reaches general-purpose embedding readiness with the required media, DRM, and accessibility support.** That would reopen the engine decision on strategic grounds, not on schedule pressure.

## 12. Recommendation and conditions

### 12.1 Primary: CEF behind a narrow Rust boundary

Adopt CEF as the primary engine foundation for EQwec Browser, with the following conditions:

1. **ADR first.** Write a browser engine ADR naming the pinned CEF branch, the `cef` crate version, the owner, the update cadence, the patch SLA, the rollback plan, and the rejected options.
2. **Legal gate.** Complete codec, DRM, third-party notice, and distribution licensing review before enabling proprietary media.
3. **Prototype gate.** Build a Windows, macOS, and Linux prototype that proves navigation, process isolation, sandboxing, profile separation, crash recovery, accessibility, codec playback, DRM playback (where required), installer packaging, and process-tree memory measurement.
4. **Security operating model.** Staff the patch, rebuild, signing, and incident-response work described in section 7.4.
5. **FFI ownership.** Keep all engine FFI in `eqwec-browser-engine`; pin and audit the `cef` crate; enforce the capability handshake; never persist engine handles.
6. **Fallback contract.** Define the engine trait so a platform-webview adapter can replace CEF without rewriting tabs, sessions, downloads, or profile policy.

### 12.2 Fallback: platform webviews behind the same contract

If CEF fails a gate or the product scope narrows, use WebView2 on Windows and WKWebView on macOS behind the same Rust engine contract. Accept the following losses:

- no single cross-platform engine behavior;
- no Chrome extension parity;
- platform-specific codec and DRM outcomes;
- less control over process policy and engine revision;
- a separate Linux decision, likely WebKitGTK, which is not evaluated here.

The fallback is not a failure. It is the correct product if EQwec cannot sustain a Chromium-based browser platform.

### 12.3 Decision triggers

Revisit the engine decision at the prototype gate, at the first security-patch exercise, and at any major product-scope change involving extensions, DRM, or Linux support. Record the outcome in the ADR and update the threat model, SBOM, and release notes.

## 13. Sources

All sources were accessed on 2026-11-07.

1. CEF, README and project overview. https://github.com/chromiumembedded/cef/blob/master/README.md
2. CEF, Branches and Building. https://chromiumembedded.github.io/cef/branches_and_building
3. CEF, General Usage. https://chromiumembedded.github.io/cef/general_usage
4. CEF, Sandbox Setup. https://chromiumembedded.github.io/cef/sandbox_setup
5. CEF issue #3559, proprietary codec support. https://github.com/chromiumembedded/cef/issues/3559
6. CEF issue #3404, Widevine persistent licenses on Linux. https://github.com/chromiumembedded/cef/issues/3404
7. Chromium, Audio/Video. https://www.chromium.org/audio-video/
8. Chrome for Developers, "Chromium's two-week release cycle". https://developer.chrome.com/blog/chrome-two-week-release
9. Brave, Release Schedule. https://github.com/brave/brave-browser/wiki/Brave-Release-Schedule
10. Brave, Chromium rebases. https://github.com/brave/brave-browser/wiki/Chromium-rebases
11. Brave, latest release notes. https://brave.com/latest/
12. Arc, official site and sunset notice. https://arc.net/
13. Servo, project site. https://servo.org/
14. The Servo Book. https://book.servo.org/
15. Microsoft Edge WebView2, Distribution. https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution
16. Microsoft Edge WebView2, Process model. https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model
17. Microsoft Edge WebView2, Browser features. https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/browser-features
18. Microsoft Edge WebView2, Overview of features and APIs. https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/overview-features-apis
19. WebView2Feedback issue #4828, Widevine support. https://github.com/MicrosoftEdge/WebView2Feedback/issues/4828
20. WebKit, Site Isolation. https://docs.webkit.org/Deep%20Dive/SiteIsolation.html
21. WebKit, "WebKit Features in Safari 18.4". https://webkit.org/blog/16574/webkit-features-in-safari-18-4/
22. Electron, Security. https://www.electronjs.org/docs/latest/tutorial/security
23. Electron, Release timelines. https://www.electronjs.org/docs/latest/tutorial/electron-timelines
24. Apple, FairPlay Streaming. https://developer.apple.com/streaming/fps/
25. crates.io, `cef`. https://crates.io/crates/cef
26. crates.io, `webview2-com`. https://crates.io/crates/webview2-com
27. docs.rs, `objc2-web-kit`. https://docs.rs/objc2-web-kit/latest/objc2_web_kit/
28. CEF, stable C++ API documentation. https://cef-builds.spotifycdn.com/docs/stable.html
