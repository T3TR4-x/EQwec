# ADR-0010: Plugin and extension trust model

- **Status:** ACCEPTED (with prototype and platform-isolation gates)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0003, ADR-0005, ADR-0007, `docs/security/THREAT-MODEL.md`

## 1. Context

EQwec hosts third-party code in several places: AudioCraft audio plugins (CLAP, VST3, LV2), Browser extensions and web content, CraftAnything asset providers, and potential future import/export adapters. Third-party code is untrusted by default. It can crash, hang, exfiltrate data, escalate privileges, or corrupt projects.

The project must decide what "trust" means for each extension surface, how code is discovered, approved, isolated, updated, and revoked. A single "plugins are trusted" assumption would undermine the browser, audio, and data-integrity guarantees.

## 2. Requirements

- Untrusted third-party code must not run with more privilege than necessary.
- A plugin crash must not crash the host where the platform allows isolation.
- Plugin scanning must be bounded, timed, and isolated.
- Users must give informed consent before a new or changed plugin binary runs.
- Plugin identity must be recorded by path, hash, vendor, version, format, and signature state.
- Updates must be verifiable and revocable.
- Browser web content must remain inside the engine's sandbox and site isolation.
- Browser extensions must use a curated capability model, not implicit Chrome Web Store parity.
- No extension may silently access project contents, credentials, browsing history, or other products' data.
- Revocation must work without reinstalling the host.
- The model must be honest about what is a security boundary and what is only crash isolation.

## 3. Alternatives considered

### A. Trust everything installed by the user

Simple, matches some desktop plugin ecosystems. Unacceptable for a browser and for a security-first product: a malicious or compromised plugin gains full user privileges.

### B. In-process hosting with no isolation

Lowest latency and simplest API. A plugin fault crashes the host, and a malicious plugin can read host memory. Acceptable only for first-party, fully reviewed code, not for arbitrary third-party plugins.

### C. Out-of-process hosting with OS sandboxing

Plugin runs in a separate process with restricted filesystem, network, and IPC. Contains crashes and limits damage. Adds IPC, shared-memory, lifecycle, and latency complexity. The target model for audio plugins and extension hosts where feasible.

### D. Out-of-process hosting without OS sandboxing

Contains crashes but not a determined attacker; the plugin still runs with the user's privileges. A pragmatic first step, not a complete security boundary.

### E. Curated allowlist only

Only EQwec-reviewed plugins run. Maximum safety, minimum ecosystem. Useful for browser extensions and high-risk surfaces, too restrictive for audio plugins.

### F. Platform store / notarization only

Relies on OS code signing and store review. Helpful signals, not sufficient: signing proves origin, not safety, and store review does not cover every plugin behavior.

## 4. Evidence

- Audio plugin binaries are native code and run with the host user's privileges unless isolated.
- CLAP, VST3, and LV2 are C ABI plugin formats; a malformed or malicious plugin can crash or exploit the host.
- Browser web content is untrusted and must stay inside Chromium's sandbox and site isolation; CEF does not expose the public Chrome extension API.
- Code signing and notarization establish origin and integrity, not benign behavior.
- No EQwec plugin host, scanner, sandbox, or extension model exists; all runtime isolation claims are `BLOCKED` until implemented and tested.
- The platform-specific sandbox mechanisms (Windows AppContainer/job objects, macOS sandbox profiles/XPC, Linux namespaces/seccomp) differ and require separate implementations.

## 5. Decision

### 5.1 Trust tiers

| Tier | Example | Trust | Required controls |
|---|---|---|---|
| T0 — First-party | EQwec shared crates, signed product components | Reviewed | Normal code review, tests, signed release |
| T1 — Reviewed third-party | Allowlisted browser extensions, signed audio plugins with a known vendor | Limited | Signature or hash pin, capability declaration, user consent, update review |
| T2 — Unreviewed third-party | Arbitrary audio plugins, imported adapters | Untrusted | Out-of-process host, OS sandbox where feasible, explicit consent, bounded scan, crash isolation |
| T3 — Hostile content | Web pages, untrusted documents, media | Hostile | Engine sandbox and site isolation, bounded parsers, no script execution outside the engine |

No T2 or T3 component receives implicit access to another product's data.

### 5.2 Audio plugins

1. Host T2 plugins out of process by default where the platform allows.
2. Scan in a separate process with timeouts, memory limits, and crash isolation.
3. Record path, SHA-256, vendor, version, format, architecture, signature state, and scan result.
4. Require explicit consent before loading a newly discovered or changed binary.
5. Treat a changed hash as a new plugin requiring consent.
6. Provide a blocklist and revocation mechanism.
7. Do not claim a security sandbox until the platform isolation is implemented and tested; out-of-process hosting is crash isolation first.
8. Do not redistribute third-party plugins without a distribution license.

### 5.3 Browser extensions

1. Do not claim Chrome Web Store parity. CEF does not expose the public Chrome extension API.
2. If extensions are supported, define a curated capability model with explicit permissions and a review process.
3. Extensions run inside the engine's process and sandbox model; they do not receive native host privileges.
4. Extension updates are verified and can be disabled or revoked.
5. Web content never receives native IPC access directly; all host communication is validated and versioned.

### 5.4 CraftAnything and import/export adapters

1. Adapters declare the formats and capabilities they support.
2. Adapters process untrusted files with bounded parsers and no network access by default.
3. An adapter cannot mutate a canonical document owned by another product; it produces an interchange package with provenance.
4. Third-party adapters follow the T2 model.

### 5.5 Identity, updates, and revocation

- Every installed plugin or extension has a stable record with identity, hash, source, version, and trust tier.
- Updates require verification and, for T1/T2, user consent.
- A signed revocation list can disable a plugin or extension without reinstalling the host.
- Revocation must fail closed for the affected component, not silently ignore the list.
- Uninstall removes plugin registrations but preserves user project data by default.

### 5.6 IPC boundary

- All host-to-plugin and host-to-extension IPC uses versioned messages, explicit authorization, bounded payloads, and peer identity checks.
- Shared memory is used only for audio/video buffers with a documented synchronization protocol.
- Validate message type, size, origin, and sequence; reject unknown critical fields.
- Do not pass raw pointers, file descriptors, or handles across a trust boundary without a documented ownership protocol.

## 6. Prototype gate

Before shipping plugin or extension hosting, prove:

1. A crashing T2 plugin does not crash the host.
2. A hung plugin is timed out and terminated without blocking the UI or audio thread.
3. A changed plugin binary requires renewed consent.
4. A revoked plugin is disabled after the revocation list is applied.
5. The plugin scanner handles malformed binaries without host compromise.
6. The chosen platform isolation actually restricts filesystem, network, and IPC access (not just process separation).
7. Browser web content remains sandboxed and site-isolated with the extension model enabled.
8. IPC fuzzing finds no host crash or unbounded allocation.

## 7. Consequences

**Positive**

- A clear, tiered trust model instead of an implicit "plugins are trusted" assumption.
- Crash isolation for audio plugins.
- A defensible browser extension position without false parity claims.
- Revocation and consent are designed in, not bolted on.

**Negative**

- Out-of-process hosting adds latency, IPC, and lifecycle complexity.
- OS sandboxing is platform-specific and may not be feasible for every plugin.
- Curated browser extensions limit ecosystem size.
- Consent prompts add friction.
- Revocation infrastructure requires signed lists and key custody.

**Security**

- Third-party code is untrusted by default.
- Crash isolation is not a security boundary; do not describe it as one.
- Signing is provenance, not safety.
- Revocation must fail closed.
- Test escape, injection, and resource-exhaustion scenarios.

**Performance**

- Measure plugin load time, IPC latency, audio callback impact, and memory per hosted plugin.
- Do not claim low latency or low overhead without a recorded workload and environment.
- Out-of-process hosting may be optional for latency-critical first-party plugins, but not for arbitrary third-party code.

**Licensing**

- Plugin formats have their own licenses (CLAP MIT, VST3 MIT since 3.8, LV2 ISC).
- Plugin binaries have their own licenses; do not redistribute without permission.
- VST trademark usage must follow Steinberg's guidelines.
- Browser extension distribution terms must be respected.

**Maintenance**

- Plugin databases and revocation lists are versioned.
- Platform sandbox implementations require per-platform ownership.
- Extension capability models evolve with the engine; re-review on engine upgrades.

## 8. Reconsideration criteria

Revisit this ADR if:

- A platform sandbox proves infeasible for audio plugins and the risk is accepted explicitly.
- Browser extension support is dropped or a different engine changes the model.
- CLAP/VST3/LV2 hosting requires a different isolation strategy.
- A plugin vendor provides a signed sandbox contract.
- Revocation or update infrastructure cannot be operated safely.
- A product needs a plugin capability not covered by the tiers.

## 9. Links

- `docs/security/THREAT-MODEL.md`
- `docs/security/SECURITY-BASELINE.md`
- `docs/architecture/SYSTEM-ARCHITECTURE.md`
- ADR-0003
- ADR-0005
- ADR-0007
