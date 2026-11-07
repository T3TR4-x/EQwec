---
name: eqwec-browser
description: Use for EQwec Browser work, including Chromium engine integration, Rust-owned browser application logic, native UI, profiles, tabs, sessions, downloads, permissions, and browser security or performance review. Trigger when touching apps/eqwec-browser, browser-specific crates, the engine FFI or renderer IPC boundary, or a browser end-to-end workflow. Do not use for Hub, creative products, or generic Rust work outside browser-owned paths; apply eqwec-engineering-core for repository-wide rules.
---

# EQwec Browser

EQwec Browser is a Chromium-based browser whose application logic, orchestration, persistence, and native UI are Rust-owned; Chromium owns web rendering.

Status: research and feasibility. The Cargo workspace does not exist yet; every path below is planned. No browser feature is implemented. This skill adds domain constraints and never weakens `eqwec-engineering-core`. Do not describe planned behavior as shipped.

## 1. Scope, ownership, activation

| Path | Responsibility | Ownership | Status |
|---|---|---|---|
| `apps/eqwec-browser` | Product entry point, native UI shell, window and process supervision | Browser | Planned |
| `crates/eqwec-browser-profiles` | Profile identity, user-data scoping, profile locks | Browser | Planned |
| `crates/eqwec-browser-tabs` | Tab, workspace, and split-view model | Browser | Planned |
| `crates/eqwec-browser-sessions` | Session schema, persistence, restore policy | Browser | Planned |
| `crates/eqwec-browser-downloads` | Download records, destination validation, lifecycle | Browser | Planned |
| `crates/eqwec-browser-engine` | Engine lifecycle, FFI, capability handshake | Browser; sole FFI boundary | Planned |
| `crates/eqwec-ipc` | Versioned IPC contracts and trust boundaries | Shared; browser consumer | Planned |
| `crates/eqwec-security` | Cryptographic verification, path safety, permission checks | Shared | Planned |
| `crates/eqwec-storage` | Profile-scoped persistence and atomic writes | Shared | Planned |
| `crates/eqwec-config` | Configuration loading, validation, migration | Shared | Planned |

Native UI primitives and tokens come from `crates/eqwec-ui`; browser workflow logic stays in the app. Do not create a crate merely to match this table.

Ownership rules:

- The browser workstream owns the app and browser-specific crates. A shared crate has its own owner, public API, compatibility expectations, and tests.
- Change a shared crate only through a versioned API, with an impact assessment naming every consumer. Do not fork a shared crate into the browser.
- Keep browser policy in browser crates. Do not push profile, tab, permission, or download policy into generic shared crates.
- `eqwec-browser-engine` is the only crate allowed to contain engine FFI. Confine `unsafe` there, document every safety invariant, and keep other browser crates able to forbid unsafe code.

Activation triggers:

- Any change under `apps/eqwec-browser` or a browser-specific crate.
- Engine selection, revision, build flags, codecs, extensions, packaging, or patch cadence.
- Profile, session, permission, download, tab, workspace, or split-view schema and behavior.
- Renderer-to-app IPC contracts or any trust-boundary change.
- A browser performance, security, compatibility, or release claim.
- A shared-crate change that affects a browser consumer.

Non-triggers:

- Hub packaging, installation, update, or launcher work; use the Hub skill and core rules.
- Creative-product document, media, or rendering workflows.
- A shared-crate change with no browser consumer and no browser-visible behavior.
- Generic Rust style, Git, or versioning questions; use `eqwec-engineering-core`.
- Documentation edits outside browser-owned documents and skills.

## 2. Domain model and invariants

### Processes

- The browser application process is Rust-owned. It owns windows, native UI, profile selection, tab/session/download/permission policy, and engine supervision. It never renders web content.
- Renderer, GPU, network, utility, and extension processes are engine-owned. Rust observes and supervises lifecycle; it does not schedule them or assume their state is trustworthy.
- Treat a profile as the unit of engine process-tree isolation. Concurrent profiles must not share an engine user-data directory; the planned default is one engine process tree per active profile.
- A renderer crash is a process failure, not a page error. Keep the tab record, preserve recoverable state, show a crash surface, and offer reload.

### Profile, workspace, tab

- A profile is a security, privacy, and persistence boundary. It owns cookies, storage, cache, history, bookmarks, downloads, permissions, extensions, and settings. An ephemeral profile, if implemented, writes nothing to disk and is never silently converted to persistent.
- A workspace is a user grouping of tabs inside one profile. It is not a security boundary and never substitutes for profile isolation.
- A tab is a Rust-owned record with a stable tab ID, profile ID, workspace ID, URL, title, favicon, loading/error state, history index, pin state, sleep state, and an engine view handle. Vertical and horizontal layouts use the same model; do not fork the model per orientation.
- Pinned tabs persist, are restored, and are not auto-discarded by default.
- Temporary tabs, such as external-link or command-palette previews, have an explicit lifetime. Never promote one to persistent without a user action.
- Split view places two independent panes in one window. Each pane has its own tab and engine view. Focus is explicit. Persist the layout only as part of the window session, and never close the other pane's tab as a side effect.

### Session, sleep, downloads, permissions, commands

- Session restoration is per profile and per window. Restore the window skeleton and tab metadata, then instantiate engine views lazily; never restore all tabs eagerly by default. Crash recovery uses the same schema through a distinct path.
- Tab sleeping or discarding selects background, non-audible, non-pinned, non-dirty candidates. Discard the engine view, keep the tab record, and recreate the view on activation. Declare which state survives; committed session state must never be lost.
- Downloads are profile-scoped Rust records. The engine transfers bytes; Rust validates destination, filename, size, and type, and records state. Never auto-open or auto-execute a download.
- Permissions are keyed by profile, top-level origin, and permission. Default deny. Prompt in native UI. Persist only an explicit decision. An origin change invalidates the decision.
- The command palette is a typed command registry. Each command declares its required capability. It may navigate, open tabs, or change UI, but must not bypass permission, download, profile, or security rules.
- Bookmarks and history are Rust-owned, profile-scoped records with migrations and bounded retention. History is not telemetry.

### State ownership

| State | Owner | Persistence rule |
|---|---|---|
| Session state | `eqwec-browser-sessions` | Profile-scoped, atomic, versioned |
| Profile state | `eqwec-browser-profiles` plus storage | Profile-scoped; no global default |
| Engine state | Engine | Ephemeral; user-data dir lives inside profile scope; never serialized as Rust records |

Invariants:

1. Every record, file path, cache key, and IPC message carries an explicit profile scope. Never rely on a global default for profile-owned data.
2. Web content is untrusted at every boundary.
3. An engine handle is not a tab identity and is never persisted.
4. Session and download writes are atomic. A truncated or partial file is a failure, never a success.
5. A tab ID survives sleep, restore, and engine restart.
6. Permission decisions default to deny and never cross profile boundaries.
7. Downloads stay inside the chosen destination and are never auto-opened.
8. Engine version and capabilities are handshaken at startup; a mismatch fails closed with a diagnostic.
9. One writer owns a profile user-data directory; a lock is enforced.
10. A feature is complete only when its end-to-end workflow passes.

## 3. Threat boundaries and security

| Boundary | Rule |
|---|---|
| Web content to renderer | Untrusted. Preserve Chromium sandboxing and site isolation. |
| Renderer, utility, extension to app | Untrusted IPC. Validate schema, version, sender, scope, and size. |
| App to engine | FFI is trusted code but fragile. Validate handles and inputs; never unwind across the boundary. |
| Profile to profile | No shared cookies, cache, storage, downloads, or engine process tree. |
| Network and downloads to disk | Untrusted bytes. Validate path, type, size, and provenance. |
| Engine update to installation | Verify signature and integrity; stage and roll back on failure. |

Security requirements:

- Keep the sandbox and site isolation enabled on every supported platform. Verify them at runtime and in release checks. If a platform build cannot enable them, the release is `BLOCKED`; do not ship a degraded-security mode to save memory, startup time, or packaging effort.
- Validate renderer IPC with typed, versioned schemas. Bind each message to its sender process, profile, and tab. Bound length, depth, and count. Reject unknown fields where the contract requires it. Close the channel on violation and log with redaction.
- Never trust renderer-supplied filesystem paths, profile IDs, tab IDs, or permission decisions. Broker native capabilities through the app with explicit capability checks.
- Permission prompts are native UI, show the top-level origin, default to deny, and require an explicit grant. Support revocation. Do not inherit grants across profiles or origins.
- Extensions are less trusted than the app. Define an explicit trust and sideloading policy, apply least privilege, verify updates, and never grant silent native or filesystem access.
- Validate downloads before use: sanitize filenames, confine to the chosen directory, reject traversal, absolute paths, and symlink escapes, bound size, warn on type mismatch or executable content, and verify signatures where the format requires it.
- Track upstream engine security releases. Document a patch cadence, an emergency path, staged rollout, and rollback. A fork or engine patch is not shippable without a maintained security update path.
- Control supply chain risk: pin the engine revision, record provenance and build flags, maintain an SBOM and license inventory, verify distributed binaries, review advisories, and keep signing keys out of the repository.
- Do not collect browsing history, URLs, download names, or page content in diagnostics or telemetry by default. Crash reporting is opt-in and redacted.
- Never weaken a security control to reduce memory, improve startup, or simplify CI.

## 4. Engine strategy

No engine is selected by this skill. An ADR is required before implementation. The ADR must name the chosen foundation, pinned revision, owner, update cadence, patch SLA, rollback plan, rejected options, and benchmark plan.

| Option | Strengths | Costs and risks |
|---|---|---|
| CEF | Mature embedding API, multi-process model, cross-platform, known integration path | CEF release lag behind Chromium security fixes, large binaries, limited extension support, codec build flags, wrapper and FFI maintenance, license notices |
| Maintained Chromium fork | Full control of features, UI, extensions, and upstream patches | Very high build, patch, and security burden; infrastructure and divergence cost; trademark and licensing exposure; viable only with sustained maintenance |
| Native system webview (WebView2, WKWebView, WebKitGTK) | Smaller installer, OS-managed updates, native integration | Engine and feature skew by OS, weaker extension and codec control, different process and security models, cross-platform parity risk |
| Other embeddable engines (Servo, Gecko) | Alternative control and licensing | Maturity, compatibility, ecosystem, and maintenance risk; evaluate only with evidence |

Decision criteria: security patch latency, cross-platform parity, process and sandbox model, extension support, codec coverage, installer size, FFI ergonomics, build reproducibility, licensing, and the team's sustained maintenance capacity.

Rust FFI boundary:

- Put all engine FFI in `eqwec-browser-engine`. Use opaque handle newtypes and a C-compatible ABI. Do not pass Rust types with unclear layout.
- Catch panics before they cross FFI; never unwind into engine code. Convert errors to typed results.
- Respect thread affinity between the engine UI thread and I/O threads. Callbacks must be reentrancy-safe and must not call the engine during teardown.
- Validate every handle and ID. A stale handle is a typed error, not undefined behavior.
- Perform a versioned capability handshake at startup and reject an incompatible engine.
- Shut down in order: stop navigation, close windows, drain callbacks, terminate the process tree, reap children. A cancelled operation must not leak a process.
- Never store engine pointers or internal structs in session or profile data.

Engine obligations: document the process model, update cadence, patch maintenance, extension support matrix, codec build flags and licensing, packaging layout, and installer size per platform. Do not promise Chrome Web Store parity, codec support, or extension behavior without a test on the packaged build.

Do not claim the engine is rewritten in Rust. Chromium owns rendering; Rust owns application logic, orchestration, persistence, and the native UI.

## 5. Resource concerns

- Measure the process tree, not one process: app, engine browser process, renderers, GPU, network, utility, and extension hosts. A single-process RSS is not browser memory.
- Track per-process memory and process count, cold and warm startup, tab switch latency, session restore time and process spawn count, suspension reclaimed memory and restore latency, cache size and eviction, IPC payload bytes and copies, GPU memory, and installer size compressed and installed.
- Treat the 20-30% memory reduction as an exploratory target, never a promise. It requires the same hardware, OS, build profile, engine revision, and workload; a recorded baseline; repeated runs with median and range; and raw evidence. Otherwise mark the claim `BLOCKED` and remove it from docs and release notes.
- Set per-scenario budgets before optimizing. Investigate a regression before accepting it. Do not present a synthetic microbenchmark as proof of a real-workload improvement.
- Bound the per-profile disk cache and define eviction and clear-on-exit behavior. Never share a cache across profiles.
- Bound IPC payloads, avoid unnecessary copies, and use shared memory or file handles where the engine supports them. Measure the result.
- Account for GPU compositor and video-decode memory. On a GPU-process crash, fall back to software compositing; do not disable the sandbox.
- The engine framework dominates installer size. Track compressed and installed size per platform and include it in release evidence.

## 6. Required end-to-end workflows

Discover the repository's actual harness commands from `AGENTS.md`, CI, or build files. Do not invent commands. If a harness or fixture is missing, record the workflow as `BLOCKED` and name the missing piece.

Fixtures:

- A deterministic local HTTP fixture site covering static HTML, a JS-heavy page, media playback, cookies and local storage, permission requests, and download endpoints with traversal, oversized, and executable filenames.
- A pinned real-site allowlist with at least three public sites: a static article, a JS application, and a media page. Record URL, date, and engine revision. A network failure is `BLOCKED`, not a pass.
- Two profiles, A and B, with distinguishable cookies, storage, history, downloads, and permission decisions.
- A session fixture with multiple windows, workspaces, pinned and temporary tabs, and a split view.
- Benchmark fixtures: 1 tab, 5 sites, 15 mixed tabs, 30 mixed tabs, video, heavy JS, three workspaces, and a long session.

| Workflow | Steps | Success criteria |
|---|---|---|
| Render real websites | Load the fixture site and the pinned allowlist; inspect title, DOM marker, and security indicators | Expected content renders; no crash; expected engine processes exist; sandbox and site isolation verified |
| Navigate | Back, forward, reload, redirect, 404, certificate error, blocked scheme, invalid URL | URL and history are correct; error pages are distinguishable; no crash; no scheme bypass |
| Open multiple tabs | Open 10 tabs across two windows; switch, reorder, close, and reopen | Tab IDs stay stable; no cross-tab state bleed; renderer count is consistent with site isolation; memory recorded |
| Restart and restore | Quit cleanly, relaunch, then repeat with a forced process kill | Windows, workspaces, tabs, pinned tabs, and split view restore; restore is lazy; no duplicates; a corrupted session fixture falls back safely |
| Download a file | Download a known file; cancel; retry; exercise traversal, absolute path, symlink, oversized, and executable fixtures | Completed file matches size and hash; partial files are cleaned or resumable; unsafe destinations are rejected; nothing auto-opens; records are profile-scoped |
| Verify profile and permission isolation | Set data and grant a permission in A; inspect B; revoke in A | B sees no A data or grant; default is deny; revocation takes effect; an ephemeral profile leaves no disk trace |
| Suspend and restore a tab | Load a heavy background tab; wait for suspension; activate it | Process count and memory drop; the tab remains; URL, history, and declared state restore; pinned, audible, and active tabs are not discarded |
| Measure process-tree memory | Settle, snapshot the tree, repeat, and compare with the baseline | Median and range recorded for every process; environment and engine revision recorded; raw evidence retained |

Benchmark methodology:

- Record hardware, OS, build profile, app commit, engine revision, flags, extensions, cache state, display, and GPU.
- Use a fresh profile unless the scenario requires warmth. Use a fixed viewport and local fixtures for determinism. Record the settle period.
- Discard one warm-up run. Run at least five measured repetitions and report median, range, and process count. Do not cherry-pick the best run.
- Measure at defined points: after launch, after settle, after load, after tab switch, after suspension, and after restore.
- Scenarios and primary metrics:
  - 1 tab: idle and loaded process-tree memory, startup.
  - 5 sites: process-tree memory, process count, tab switch latency.
  - 15 mixed tabs: memory, process count, switch p95.
  - 30 mixed tabs: memory, process count, switch p95, crash count.
  - Video: CPU, GPU memory, dropped frames where measurable.
  - Heavy JS: CPU, responsiveness, tab switch latency.
  - Three workspaces: switch cost and restore time.
  - Long session, such as two hours or 500 navigations: memory growth slope, process leaks, crash count.
- Compare against the same scenario on the baseline app and engine revision. A performance claim is valid only when reproduced; otherwise report it as not demonstrated.

## 7. Failure modes, debugging, regression, platforms

| Failure | Required behavior |
|---|---|
| Renderer crash or OOM | Keep the tab record and session; show a crash surface; offer reload; apply crash-loop backoff |
| GPU process crash | Fall back to software compositing; record a diagnostic; keep the sandbox enabled |
| Network or utility crash | Isolate the failure; reconnect or retry; fail downloads cleanly without corrupting records |
| Engine hang | Watchdog, diagnostic dump, terminate the process tree, restore recoverable tabs |
| Corrupted or partial session file | Reject it, use a backup, and fail to a safe state; never silently discard all tabs |
| Profile lock contention | Focus the existing instance or return an explicit error; never share the user-data directory |
| Download collision or traversal | Sanitize, uniquify, or reject; never overwrite outside the destination |
| Permission-prompt spoofing | Prompt only in native UI and display the verified origin |
| Extension crash or compromise | Disable or isolate it, surface the event, and preserve other tabs |
| Engine ABI or version mismatch | Fail the capability handshake with a diagnostic; roll back |
| Malformed or oversized IPC | Reject, close the channel, and log redacted details; do not crash the app |
| Disk full | Fail atomically and preserve the previous session and download records |
| Quit hang or zombie processes | Ordered shutdown, timeout, terminate, and reap |

Debugging procedure:

1. Reproduce with the recorded build, engine revision, profile, and fixture.
2. Classify the layer: Rust app, IPC, engine, platform, or fixture/network.
3. Collect redacted app logs, engine logs, a process-tree snapshot, and a crash dump with consent.
4. Confirm the profile scope and the engine user-data directory.
5. Isolate with a fresh profile, one tab, and extensions disabled where supported.
6. Compare against the previous app and engine revisions.
7. Write the regression test and add the fixture before fixing.
8. Re-verify that sandboxing and site isolation remain enabled.

Regression tests:

- Unit: tab state machine, session serialization and migration, download path validation, permission store, workspace membership.
- Contract: IPC schema fuzzing, malformed and oversized messages, version mismatch, engine capability handshake.
- Integration: profile isolation, crash restore, download lifecycle, sleep and restore.
- End-to-end: every workflow in section 6 on each supported platform.
- Security: assert sandbox and site isolation are active, path traversal corpus, default-deny permissions, extension policy.
- Performance: the benchmark scenarios with recorded thresholds and variance handling. Do not make a flaky benchmark a hard gate without a stability plan.
- Adversarial: malicious filenames, huge headers, redirect loops, certificate errors, popup and permission spam.

Cross-platform caveats:

- CEF and fork builds: per-platform binaries, helper executables, locale resources, snapshot blobs, codec flags, and version skew. Run a build and test matrix per platform and architecture.
- macOS: nested helper apps in the bundle, signing every executable, hardened runtime, JIT entitlements, notarization, Gatekeeper, and quarantine. An update must preserve valid signatures.
- Windows: restricted-token and job-object sandbox behavior, DLL search order, Authenticode, SmartScreen, long paths, per-user versus per-machine install, and GPU driver variance.
- Linux: the sandbox needs user namespaces or an approved setuid helper; distro AppArmor policy, Wayland versus X11, glibc baseline, and packaging format all affect it. If no approved sandbox path exists, the release is `BLOCKED`.
- Account for case sensitivity, path separators, Unicode filenames, download directories, and locale differences in session and download tests.

## 8. Release, documentation, handoff

Update or assess documentation when:

- The engine revision, build flags, codec set, extension support, or packaging changes: update the ADR, threat model, SBOM, licenses, benchmarks, and release notes.
- Profile, session, tab, permission, or download schema changes: add migrations, compatibility tests, and user-facing documentation.
- An IPC contract changes: bump its version, assess every consumer, and add contract tests.
- A security control or permission default changes: require security review and document the user-visible behavior.
- A performance claim is added: attach reproducible evidence or remove the claim.
- A new platform or architecture is proposed: apply the platform checklist; do not call it supported before the gates pass.
- Only impacted documents change. `main` contains `README.md` only; skills and technical documents live on `integration`, `dev-*`, and eligible release branches.

Handoff:

- Hand off to `eqwec-engineering-core` for architecture, Rust and FFI rules, security policy, Git, versioning, and documentation conflicts. Core wins any conflict.
- Hand off to `eqwec-quality-control` for pre-change, implementation, verification, performance, merge, and release gates. Supply the domain fixtures, failure modes, and benchmark scenarios from this skill.
- Record a missing mandatory skill or unavailable gate as `BLOCKED`, not as a pass.

## Red flags

- Disabling the sandbox or site isolation to save memory, improve startup, or simplify CI.
- Restoring all tabs eagerly on startup or after a crash.
- Storing session, history, cookies, or downloads outside the profile scope, or under a global default.
- Shipping an engine patch or fork without a maintained security update path and patch SLA.
- Treating a renderer crash as a page error and dropping recoverable session state.
- Trusting renderer-supplied paths, profile IDs, tab IDs, or permission decisions.
- Auto-opening or auto-executing a download.
- Granting permissions by default or persisting a grant without an explicit prompt.
- Claiming a 20-30% memory reduction without a reproducible process-tree benchmark.
- Calling the engine Rust-written, or claiming Chromium rendering was rewritten in Rust.
- Adding `unsafe` FFI outside `eqwec-browser-engine` or allowing unwinding across FFI.
- Sharing one user-data directory or engine process tree between concurrent profiles.
- Measuring one process and calling it browser memory.
