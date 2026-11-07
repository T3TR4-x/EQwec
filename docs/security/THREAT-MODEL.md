# Threat Model

**Status:** planning baseline refined by the Hub developer-preview implementation. Architecture-dependent details for other products will be refined after the browser-engine, GUI, media, and audio decisions are accepted. This model covers the ecosystem-level trust boundaries that every product inherits.

## 1. Scope and assumptions

In scope:

- EQwec Hub: launcher, package manager, installer, updater, repair, uninstall.
- EQwec Browser: Chromium-based shell, profiles, tabs, sessions, downloads, permissions.
- PhotoCraft, VectorCraft, FilmCraft, EffectCraft, CraftAnything, AudioCraft: local desktop editors and workspace.
- Shared crates: storage, security, IPC, updater, graphics, media, logging, config.
- Build, packaging, signing, and release infrastructure.

Assumptions:

- The user's operating system and local account are trusted to the extent the platform provides.
- Web content, downloaded packages, imported documents, media, fonts, plugins, and IPC peers are untrusted.
- The application may run offline; network availability is not guaranteed.
- No mandatory account or cloud service is assumed.
- Third-party code and assets have their own licenses and security posture.

Out of scope for this baseline:

- Nation-state adversaries with platform compromise.
- Physical attacks on the user's device.
- Vulnerabilities in the operating system or Chromium that are not reachable through EQwec's boundary.
- Legal conclusions; see `docs/research/LEGAL-LICENSE-REVIEW.md` when complete.

## 2. Assets

| Asset | Why it matters |
|---|---|
| User projects and documents | Irreplaceable creative work; integrity and confidentiality |
| Browsing history, sessions, cookies, credentials | Privacy and account security |
| Signing keys and release metadata | Supply-chain integrity |
| Installation registry and package cache | Trust in what is launched |
| Plugin and extension binaries | Code execution with user privileges |
| Audio recordings and media | Privacy and irreplaceable content |
| Configuration and preferences | Behavior, privacy, security settings |
| Diagnostics and logs | May contain sensitive paths or content if not redacted |
| Update channel and version state | Downgrade/replay resistance |

## 3. Actors

| Actor | Capability | Trust |
|---|---|---|
| Local user | Full local access under the OS account | Trusted for their own data; may make unsafe choices |
| Malicious document author | Crafts project, SVG, media, font, or manifest input | Untrusted |
| Malicious website | Runs in the browser renderer | Untrusted |
| Malicious plugin/extension | Native or web code loaded by the app | Untrusted until verified and sandboxed |
| Network attacker | Controls DNS, proxies, mirrors, or update responses | Untrusted |
| Compromised dependency | Supply-chain code in the build or runtime | Untrusted until reviewed |
| Local unprivileged process | Attempts IPC spoofing or file access | Untrusted |
| Insider or stolen signing key | Can sign malicious releases | High impact; requires custody controls |
| Accidental user error | Deletes, overwrites, or misconfigures | Mitigate with recovery and confirmation |

## 4. Trust boundaries

```text
[ user / OS ]
     |
     v
[ EQwec app process ] --IPC--> [ helper / renderer / plugin process ]
     |                                   |
     |                                   v
     |                            [ untrusted content ]
     v
[ local files, registry, caches, credentials ]
     ^
     |
[ network: package servers, update metadata, web content ]
```

Boundaries that require explicit validation:

1. Network to updater/package manager.
2. Renderer/web content to browser application.
3. Plugin/extension to host application.
4. Imported document/media/font to parser and renderer.
5. Local IPC peer to privileged operation.
6. Application to operating-system APIs and filesystem.
7. Build inputs to release artifact.

## 5. Threat table

Risk is qualitative: likelihood and impact are `low`, `medium`, `high`, or `critical`. Ratings are planning judgments, not measurements.

### 5.1 Hub and updates

| ID | Threat | Likelihood | Impact | Mitigation | Residual risk |
|---|---|---|---|---|---|
| H-01 | Forged or tampered update manifest | medium | critical | Signature over canonical bytes; pinned trust root; reject unknown critical fields | Key compromise remains |
| H-02 | Revoked or wrong signer accepted | medium | critical | Key ID/epoch binding, revocation check, channel binding | Revocation propagation delay |
| H-03 | Package tampering or truncation | medium | high | Size + strong hash before extraction and activation; re-verify staged files | Collision resistance assumed |
| H-04 | Path traversal or symlink escape in archive | medium | high | Normalize, reject absolute/`..`/links outside root, verify resolved paths | Platform-specific path quirks |
| H-05 | Archive bomb or disk exhaustion | medium | medium | Bound entries, bytes, ratio, time; disk preflight | Shared-disk pressure |
| H-06 | Downgrade or replay | medium | high | Monotonic sequence, expiry, signed downgrade policy, user confirmation | Offline metadata expiry policy |
| H-07 | Interrupted update leaves mixed tree | high | high | Journal, staging, atomic activation, recovery to old or new | Power-loss timing |
| H-08 | Unverified binary executed | medium | critical | Never execute before verification and staging; health check only verified binaries | Test-key bypass misconfiguration |
| H-09 | Signing-key theft | low | critical | Offline/HSM custody, rotation, revocation, incident response | Insider risk |
| H-10 | IPC spoofing to privileged operation | medium | high | Authenticated per-user socket, ACLs, version/authorization checks | Local same-user malware |
| H-11 | Unsafe elevation | low | high | Per-user install by default; absolute paths; no inherited `PATH` | Platform installer quirks |
| H-12 | Malicious local bundle layout (extra files, symlinked manifest or package) | medium | high | Accept exactly `manifest.json` and `package.zip`, both regular files, no symlinks; absolute bundle path; verify before copy | Same-user filesystem control |
| H-13 | Health check hangs or leaves an orphan child | medium | medium | Ten-second bound, kill and reap on timeout, no shell, stdio nulled | Product startup side effects |
| H-14 | Concurrent launcher mutation corrupts the profile | medium | high | Exclusive profile lock; read-only mode with disabled mutations when the lock is unavailable | Same-user lock bypass |
| H-15 | Launch after pointer or tree tampering | medium | high | Pointer-first resolution; pointer/record/tree/entry must agree; reject symlinked tree, bin, or entry; no PATH lookup | Local administrator |

### 5.2 Browser

| ID | Threat | Likelihood | Impact | Mitigation | Residual risk |
|---|---|---|---|---|---|
| B-01 | Renderer compromise reaches app/OS | medium | critical | Chromium sandbox, site isolation, process isolation, minimal IPC surface | Chromium 0-day |
| B-02 | Malicious IPC payload from renderer | medium | high | Validate version, type, size, origin, authorization; no arbitrary paths/commands | Parser bugs |
| B-03 | Profile/session data leak across profiles | medium | high | One engine user-data dir per profile; explicit profile scope; no global default | Engine bugs |
| B-04 | Session restore loses or corrupts state | medium | medium | Atomic session writes, schema versioning, recovery, bounded metadata | Crash timing |
| B-05 | Extension escapes intended permissions | medium | high | Capability model, sandbox, review, disable by default where uncertain | Extension ecosystem risk |
| B-06 | Download executes or escapes | medium | high | Mark-of-the-web, quarantine, explicit open, path validation | User override |
| B-07 | Memory optimization weakens isolation | medium | critical | Never disable sandbox/site isolation for memory; measure only safe lifecycle changes | Policy regression |
| B-08 | Update lag for Chromium security patches | high | critical | Track upstream, patch SLA, staged rollout, kill switch for unsafe versions | Upstream disclosure timing |
| B-09 | Malicious website exploits codec/GPU path | medium | high | Out-of-process codecs/GPU where possible, bounds, update cadence | Driver bugs |
| B-10 | Permission confusion (camera, mic, location) | medium | high | Explicit prompts, per-origin scope, visible indicators, revocation | UI deception |

### 5.3 Documents, media, and parsers

| ID | Threat | Likelihood | Impact | Mitigation | Residual risk |
|---|---|---|---|---|---|
| D-01 | Malformed project/SVG/media crashes or corrupts | high | medium | Bounded parsing, fuzzing, process isolation for risky decoders, safe failure | Parser complexity |
| D-02 | Script or external reference executed on import | medium | critical | No script execution, no network/file fetch, DTD/XXE disabled | Parser feature gaps |
| D-03 | Path traversal on save/export | medium | high | Normalize, resolve, restrict to chosen root, atomic writes | Symlink races |
| D-04 | Silent data loss on save/reopen | medium | high | Round-trip tests, canonical serialization, unknown-data preservation, backup/recovery | Format edge cases |
| D-05 | Font or codec vulnerability | medium | high | Vetted parsers, bounds, isolation, update cadence | Third-party bugs |
| D-06 | Clipboard/paste bypasses validation | medium | medium | Treat paste as untrusted import; revalidate geometry/styles/references | OS clipboard quirks |
| D-07 | Metadata used as markup or script | low | medium | Treat as display-only, sanitize, bound | Renderer bugs |
| D-08 | Autosave/recovery overwrites good data | medium | high | Atomic write, retain previous version, quarantine corrupt recovery | Disk-full behavior |

### 5.4 Plugins and extensions

| ID | Threat | Likelihood | Impact | Mitigation | Residual risk |
|---|---|---|---|---|---|
| P-01 | Malicious plugin executes with full privilege | high | critical | Capability manifest, trust class, sandbox/out-of-process host, explicit install | Native plugin escape |
| P-02 | Plugin crash takes down the app | high | high | Out-of-process hosting, restart, state isolation, crash reporting | Shared-memory corruption |
| P-03 | Plugin scans or exfiltrates user data | medium | high | Capability-scoped filesystem/network, no ambient access, audit | Covert channels |
| P-04 | Plugin license violation | medium | high | License inventory, distribution terms, user notice | Legal interpretation |
| P-05 | Unsigned or tampered plugin loaded | medium | high | Signature/trust policy, hash pinning, allowlist where appropriate | User override |

### 5.5 Shared infrastructure and supply chain

| ID | Threat | Likelihood | Impact | Mitigation | Residual risk |
|---|---|---|---|---|---|
| S-01 | Compromised dependency or build script | medium | critical | Minimal deps, lockfile, advisory/license scan, review, reproducible builds | Upstream compromise |
| S-02 | Secret or key committed | medium | critical | Secret scanning, key custody, no secrets in fixtures/logs | Human error |
| S-03 | Logs leak content or credentials | medium | high | Redaction, bounded logs, no sensitive defaults | Debug builds |
| S-04 | Tenant/profile data crosses scope | medium | high | Explicit scope on every path/query/cache/IPC key | Refactor regressions |
| S-05 | Unsafe deserialization | medium | high | Typed schemas, bounds, reject unknown critical fields | Schema evolution |
| S-06 | CI artifact tampering | low | critical | Pinned actions, least-privilege tokens, signed artifacts, provenance | CI compromise |
| S-07 | Release channel confusion | medium | high | Signed channel binding, explicit switch, UI clarity | Metadata bugs |

## 6. STRIDE mapping

| STRIDE | Primary EQwec concerns |
|---|---|
| Spoofing | Update signer, IPC peer, plugin identity, profile identity |
| Tampering | Packages, manifests, project files, caches, release artifacts |
| Repudiation | Audit records for installs/updates, provenance for assets |
| Information disclosure | Browser history, project contents, audio, logs, cross-profile leaks |
| Denial of service | Archive bombs, parser exhaustion, GPU/CPU starvation, disk exhaustion |
| Elevation of privilege | Renderer/plugin escape, unsafe elevation, path traversal, IPC authorization |

## 7. Security requirements by phase

| Phase | Requirement |
|---|---|
| Planning | Threat model, baseline, architecture review, license review |
| Workspace baseline | Dependency policy, secret scanning, CI security checks, fuzz harness plan |
| Hub MVP | Signed manifests, package verification, transactional install, rollback tests |
| Browser prototype | Sandbox/process isolation, IPC validation, update strategy, profile isolation |
| Creative apps | Bounded parsers, atomic saves, recovery, format security tests |
| Plugin hosting | Trust model, sandbox/out-of-process, capability permissions, crash containment |
| Release | Signing, notarization/Authenticode, integrity, install/update/rollback verification |

## 8. Residual risks and open questions

- Chromium 0-day risk cannot be eliminated; patch latency and rollout must be measured.
- Codec and DRM licensing may force feature limitations; legal review is pending.
- Native plugin sandboxing is platform- and plugin-format-dependent; the achievable isolation level is not yet known.
- GPU driver bugs may be reachable from untrusted content; process isolation and fallback need validation.
- Cross-profile isolation depends on the engine's process model; a prototype must verify it.
- Signing-key custody and release authorization are organizational controls, not code.
- The project has no security incident response process yet; it must be defined before a public release.

## 9. Review triggers

Review this threat model when:

- A new trust boundary, process, plugin format, network service, or file format is added.
- The browser engine, GUI stack, media stack, or plugin host changes.
- A security finding is confirmed or a control is waived.
- A new platform or installer format is supported.
- A release gate is prepared.

## 10. Current status

| Item | State |
|---|---|
| Threat model | This planning baseline |
| Architecture-specific data-flow diagrams | Pending technology decisions |
| Security baseline | `docs/security/SECURITY-BASELINE.md` |
| Product code | None |
| Security tests | `BLOCKED` until workspace exists |
| Security gate | Open |
