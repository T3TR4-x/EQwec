---
name: eqwec-hub
description: Use when working on EQwec Hub — its launcher, package manager, installer, updater, repair, uninstall, release channels, offline install, diagnostics, installation registry, update manifests, package formats, platform installers, or launcher lifecycle — including changes under apps/eqwec-hub, crates/eqwec-updater, or Hub packaging and signing. Trigger when a change affects installation, update, rollback, repair, uninstall, channel policy, trust, or launcher-to-product contracts. Do not use for another EQwec product's internal UI or workflows, unrelated repositories, or changes that touch no Hub path, package contract, or installation behavior.
---

# EQwec Hub

Hub is the launcher, package manager, installer, updater, repair tool, and installation health manager for EQwec products. It is a product with its own release cycle, not a shared library.

## Status and authority

- The Cargo workspace exists with `eqwec-core`, `eqwec-security`, and `eqwec-updater`. The verification core, durable-state primitives, in-process activation engine, bounded package staging, and a non-production fixture product are implemented; no Hub application, installer, uninstaller, or launcher UI exists. Verify the current repository before editing and report drift instead of creating empty scaffolding.
- Manifest, trust-store, package verification, registry, journal, layout, active-pointer, activation, rollback, startup recovery, and bounded ZIP/ZIP64 staging are implemented and tested. The fixture product at `tests/fixtures/eqwec-hub` exercises install, launch, update, rollback, tampered-package rejection, and one interrupted-activation recovery in process on macOS. Download, uninstall, repair orchestration, installer packaging, and the launcher are not implemented. Do not describe planned behavior as shipped.
- `eqwec-engineering-core` governs architecture, security baseline, Git, versions, and documentation. `eqwec-quality-control` governs gates and evidence. This skill adds Hub domain rules and may not weaken either one.
- Discover real commands, paths, and formats from `AGENTS.md`, CI, and the workspace. Do not invent commands, schemas, or test results.

## Activation

Use this skill when a task touches:

- Hub launcher, installation registry, install layout, shortcuts, uninstall, repair, rollback, or self-update.
- Package manager behavior, update checks, release channels, offline install, diagnostics, or launcher lifecycle.
- Manifest schema, update metadata, package format, Hub IPC, signing/trust, or platform installers.
- `apps/eqwec-hub`, `crates/eqwec-updater`, Hub packaging, or a shared crate change that alters Hub behavior.

Do not use it for:

- Another product's internal UI, editing, rendering, or document workflow, unless that product consumes a Hub package, manifest, or launcher contract.
- Generic Rust work outside Hub paths and shared crates used by Hub.
- Unrelated repositories.
- A pure core-policy question that does not change Hub behavior; core still applies.

## 1. Ownership map

| Path | Owner | State | Rule |
|---|---|---|---|
| `apps/eqwec-hub` | Hub | Planned | Product entry point, launcher UI, Hub policy, self-update orchestration. |
| `crates/eqwec-updater` | Hub-owned, shared API | Exists: verification, durable-state, activation, and staging engine | Generic download, verification, staging, activation, rollback, and recovery engine. No Hub UI or product-specific policy. |
| `crates/eqwec-security` | Security owner | Exists: digest, signature, key id, safe path | Cryptographic verification, path safety, permission checks. Hub consumes; do not reimplement. |
| `crates/eqwec-storage` | Storage owner | Planned | Atomic writes, tenant/profile-scoped persistence. Hub uses it for registry, journal, and cache. |
| `crates/eqwec-config` | Config owner | Planned | Configuration loading, validation, migration. Hub uses it for channel and launcher settings. |
| `crates/eqwec-ipc` | IPC owner | Planned | Versioned transport and message primitives. Hub owns only its message types and authorization rules. |
| `packaging/eqwec-hub/` | Hub + release engineering | Planned | Installers, manifest templates, signing/notarization configuration, and packaging scripts. Never private keys. |
| `tests/fixtures/eqwec-hub/` | Hub | Exists: non-production v1/v2 fixture and in-process end-to-end tests | Real test application, signed test packages, and a public non-production seed. Never reuse the seed or fixture for release signing. |
| `docs/product-specs/eqwec-hub.md` | Hub | Planned | Product scope, channel policy, install layout, registry schema, and supported platforms. |

Shared-ownership rules:

- Hub owns product policy: channel rules, install layout, registry semantics, launcher UX, and self-update behavior.
- `eqwec-updater` owns the generic transaction engine. It must accept a product identity and manifest; it must not know Hub product IDs, UI strings, or release policy.
- `eqwec-security` owns cryptographic and path-safety primitives. Hub must call them rather than duplicate verification logic.
- A change to a shared crate requires an impact assessment naming every consumer, compatibility expectations, and tests that exercise Hub and the other consumers.
- Shared crates must not depend on `apps/eqwec-hub` or any product crate.
- Packaging and signing changes require the release gate; test keys must never be production keys.

## 2. Domain model and invariants

| Term | Meaning |
|---|---|
| Product ID | Stable canonical identifier. It is immutable and must not be derived from a display name, install path, or executable name. |
| Version | Independent `MAJOR.MINOR.PATCH` per product. Build metadata never determines ordering. |
| Target | Operating system, architecture, and minimum platform. A universal package must declare every supported target explicitly. |
| Channel | Signed release stream such as stable, beta, or alpha. Channel policy defines acceptable versions and binds the channel to allowed signers. |
| Package source | HTTPS URL or an explicitly selected local offline bundle. Production sources are allowlisted; insecure schemes are rejected. |
| Size and hashes | Exact byte size and strong cryptographic digest. Both are verified before extraction and before activation. |
| Signature metadata | Key ID, algorithm, signature, signing time, key epoch, and revocation state, chained to a pinned trust root. |
| Dependencies | Exact product ID, version, and package hash edges. The graph must be acyclic. |
| Installation registry | Authoritative, schema-versioned record of active installations, channels, versions, roots, and signer identity. |
| Staged installation | Verified but inactive tree. It is never launched and never treated as active. |
| Active installation | The tree the launcher starts. It is not modified in place while current. |
| Transaction journal | Durable intent and phase record used to recover interrupted operations. |
| Health check | Bounded launch and version check of a verified staged or activated binary. |
| Repair | Restore missing or corrupted files of the same version from a verified package. |
| Rollback | Transaction that reactivates a retained, verified previous version. |
| Offline bundle | Signed manifest, packages, and trust data sufficient to install or repair without network access. |

Invariants:

- Product ID, manifest, package metadata, registry record, install root, shortcut, and IPC payload must agree. Reject any mismatch.
- Version ordering is per product and per channel. A downgrade requires a signed policy allowance and explicit user confirmation; a version string alone never authorizes it.
- Target matching is exact unless the package declares the target. Reject a wrong OS or architecture before extraction or activation.
- A channel is bound to its allowed signer identity and key epoch. Cross-channel packages are rejected unless a signed channel switch is in progress.
- Package source, size, and hash are security-relevant manifest fields. They must be covered by the manifest signature and checked against the received bytes.
- Dependencies install before dependents. Cycles, missing dependencies, and incompatible versions fail before activation. Uninstall refuses to break a dependent installation without explicit user action.
- Minimum platform is checked before activation. Unsupported systems fail with an actionable message and no partial install.
- The registry has at most one active record per product and profile. Writes are atomic, schema-versioned, and recoverable from the journal.
- A staged tree becomes active only through a transactional activation. Never write directly over an active tree.
- Activation writes an intent record, verifies the staged tree, swaps the active pointer or directory atomically, updates the registry, and commits. A crash at any phase must recover to a complete old or complete new state, never a mix.
- The previous known-good version remains available until the new version passes its health check and the retention policy permits pruning. Rollback uses only verified retained bits.
- Every mutating operation has an idempotency key. Repeated install, update, repair, rollback, or uninstall converges without duplicate registry entries, shortcuts, services, or downloads.
- A partial download is never complete. Resume only with a strong validator, and re-verify the full size and hash after resume. Partial staging is discarded or resumed from a verified checkpoint.
- On startup, reconcile the journal, locks, staging directories, and registry. Clean stale temporary data; never guess that an interrupted operation succeeded.
- Repair preserves the version, channel, and user data. If the verified package is unavailable, fail closed and offer reinstall rather than silently changing version.
- Uninstall removes Hub-owned files, registry entries, shortcuts, services, and scheduled tasks. User projects and documents are preserved by default; purge requires explicit confirmation.
- The launcher starts without network access, shows cached registry state, and performs update checks in the background. A network failure must not block launching an installed product.
- Offline install uses the same signature, integrity, target, channel, and disk checks as online install. No insecure network fallback.
- Diagnostics are bounded and redacted. Record operation ID, product, version, channel, target, phase, and error code; never record secrets or signed URLs.

## 3. Threat boundaries and security

| Threat | Required control | Failure behavior |
|---|---|---|
| Manifest tampering or forgery | Verify the signature over the canonical manifest bytes using a pinned trust root. Reject unknown critical fields, duplicate keys, and trailing data. | Fail closed before download or install. |
| Wrong signer or channel | Bind key ID and epoch to allowed channels. Check revocation. Require a signed, explicit channel switch. | Reject and report signer/channel mismatch. |
| Package tampering or truncation | Verify exact size and strong hash before extraction. Re-verify staged files before activation. Reject truncated streams. | Delete the partial artifact; keep the active install unchanged. |
| Path traversal | Normalize archive paths. Reject absolute paths, `..`, drive and UNC prefixes, NUL bytes, duplicate names, and case-colliding names. | Reject the package and clean staging. |
| Symlink or hardlink escape | Do not follow links out of staging. Reject unsafe links or materialize them only under an explicit safe policy. Verify resolved paths remain inside the root. | Reject the package. |
| Malicious archive | Bound entry count, path length, decompressed bytes, compression ratio, and extraction time. Reject unsupported methods, device files, and nested archives unless the format explicitly allows them. | Abort before exceeding a limit. |
| Downgrade attack | Enforce monotonic version and metadata sequence per channel. Allow downgrade only with signed policy and explicit user confirmation. | Reject; retain the current active version. |
| Replay or stale metadata | Use channel sequence, issued/expiry times, and a nonce where needed. Reject older sequences. Bound clock skew. Define offline expiry policy explicitly. | Fail closed or require a verified refresh. |
| Disk-space exhaustion | Preflight download, staging, rollback, and safety margin. Reserve space and monitor writes. | Fail before mutating the active installation; clean temporary data. |
| Unverified binary execution | Never execute a downloaded binary before signature and hash verification and staging. Run health checks only on verified binaries. | Do not launch. No "version probe" shortcut. |
| Signing-key compromise or poor custody | Keep private keys offline or in an HSM. Separate test and production keys. Support rotation with overlap and revocation. Document incident response. Never place keys in the repository, CI logs, or fixtures. | Revoke the key, fail closed, and block release. |
| Credential-store failure | Use the platform credential store where required. Do not fall back to plaintext files. | Abort the operation with an actionable diagnostic. |
| Local IPC spoofing | Use a per-user authenticated socket or named pipe with restrictive ACLs. Validate message version, operation, and authorization. Accept no arbitrary path, URL, or command from an unprivileged peer. | Reject the message; log bounded context. |
| Unsafe elevation | Install per-user by default. Elevate only for an approved machine-wide install. Use absolute paths and do not trust `PATH` or inherited environment. | Refuse unsafe elevation. |

Additional security rules:

- Parse untrusted input with explicit bounds. Signature verification must use the exact canonical bytes defined by the schema, not a re-serialization of untrusted data into a different form.
- Any signature, integrity, or credential-store failure is terminal. Do not add an "ignore", "continue", or insecure fallback path to a production build.
- If a development bypass exists, isolate it in a separate build, use test keys only, show a loud UI warning, and make it impossible to enable in release artifacts.
- Do not log signed URLs, tokens, private keys, or raw user paths in diagnostics.

## 4. Interfaces and dependencies

### Manifest schema

The schema must define at least: `schema_version`, product ID, version, channel, target (OS, architecture, minimum platform), package source, package size, package hashes, signature metadata, dependencies, metadata sequence, issue/expiry times, and optional release notes. The concrete wire format and field names require an ADR and a versioned schema document before implementation.

Schema rules:

- Canonical encoding, bounded parsing, and explicit limits on field length, list count, and nesting.
- Reject duplicate keys, unknown critical fields, inconsistent lengths, and trailing data.
- No executable logic, dynamic code, or arbitrary command fields.
- The signature covers every security-relevant field, including source, size, hashes, target, channel, dependencies, and minimum platform.

### Update metadata

- A channel index carries sequence, issue/expiry time, manifest hash, signature, minimum launcher version, and revocation information.
- Sequences are monotonic per channel. An older sequence is a replay and is rejected.
- Staged rollout and delta updates are optional. A delta result must match the full package hash; otherwise fall back to the full package.
- Metadata and package must belong to the same channel and signer.

### Launcher IPC

- Local-only, typed, versioned contract with explicit authorization. Operations include status, list, install, update, repair, rollback, uninstall, cancel, and export diagnostics.
- Support progress events, cancellation, timeouts, backpressure, and an idempotency key per mutation.
- An unprivileged peer cannot supply arbitrary executable paths, URLs, or commands.
- Launcher-to-product contract: registry-provided executable path, documented arguments/environment/working directory, exit-code meaning, and single-instance handoff. A product must not require Hub at runtime unless the contract says so.

### Package formats

- Use one canonical file-manifest layout with deterministic ordering and per-file path, type, size, hash, and mode.
- Reject absolute paths, unsafe links, setuid/setgid bits, device files, and unbounded metadata.
- Platform wrappers may use different containers, but must preserve the same file manifest and hashes. Verify uncompressed content.
- Packaging must be reproducible for a given toolchain and inputs.

### Platform installers

- The Hub installer is a thin, signed/notarized wrapper that installs the launcher per-user by default, registers uninstall and shortcuts only with consent, and can repair or uninstall offline.
- Hub installs product packages. The OS package manager is not the default product installation path; any integration requires an approved, tested design.
- Self-update uses a helper process or atomic replacement. Never overwrite a running binary in place.

### Dependency direction

| Component | May depend on | Must not depend on |
|---|---|---|
| `apps/eqwec-hub` | `eqwec-updater`, `eqwec-security`, `eqwec-storage`, `eqwec-config`, `eqwec-ipc`, `eqwec-core`, `eqwec-logging`, `eqwec-ui` as needed | Product crates; another product's app code |
| `crates/eqwec-updater` | `eqwec-security`, `eqwec-storage`, `eqwec-config`, `eqwec-core`, `eqwec-logging` | Hub UI, Hub app, product crates |
| `crates/eqwec-security` | `eqwec-core` | Updater, Hub app, product crates |
| Shared crates | Lower-level shared crates | Any app or product-specific crate |
| Product apps | Shared crates and their own product code | Hub code; another product's code |

- No product-to-product coupling. If two products need shared behavior, extract a shared crate with an owner, public API, compatibility expectations, and tests, or define an explicit interchange contract.
- Version the manifest schema, registry schema, IPC contract, and updater API independently. Provide migration and compatibility tests for every version boundary.

## 5. Resource concerns

| Concern | Required treatment | Evidence |
|---|---|---|
| Install size | Report compressed download bytes and installed bytes per product. Set a per-release budget and investigate unexplained growth. | Package manifest and measured install tree. |
| Download size | Record full and delta sizes. Use conditional requests and caching; do not re-download a verified cached package. | Transfer log and cache state. |
| Cold startup | Measure time to an interactive launcher with cached registry and network disabled. Do not block startup on an update check. | Per-platform baseline. |
| Warm startup | Measure the same workload after OS cache warm-up. | Per-platform baseline. |
| Idle memory | Measure after 60 seconds idle. Avoid a resident background daemon unless justified and bounded. | Profiler and process tree. |
| Disk footprint | Account for active, staging, download temporary files, rollback retention, logs, and a safety margin. Preflight before mutation. | Disk math test and measured usage. |
| Update bandwidth | Measure bytes over the wire. Deltas are optional, verified, and always have a full-package fallback. | Transfer log. |
| Concurrent operations | One writer per installation root. Bound parallel downloads. Queue or reject conflicting operations. Use a crash-safe lock with stale detection. | Concurrency and kill tests. |
| Bounded extraction | Cap entry count, path length, decompressed bytes, compression ratio, and extraction time. Reject before exceeding a cap. | Adversarial archive tests. |
| Rollback storage | Retain at least the previous known-good version until the new version passes its health check. Prune by count or bytes; never prune during an update. | Registry and disk tests. |
| Registry, cache, and log growth | Bound and prune each store. Vacuum stale records. Keep logs rotated and redacted. | Long-run test. |

Do not state a performance number without hardware, OS, build profile, workload, command, and result. If a measurement is unavailable, mark the claim `BLOCKED` per `eqwec-quality-control`.

## 6. Required end-to-end workflows

Fixture requirements:

- Build a real fixture product with at least v1 and v2, packages for every claimed target, stable and beta manifests, an offline bundle, and non-production test keys.
- The fixture executable must print its version and exit nonzero on failure. A mock HTTP server alone is not an application fixture.
- Run each workflow on a clean per-user profile and a disposable VM or container for the target platform.
- Record registry snapshots, file-tree hashes, process exit codes, bounded logs, and disk usage.
- Never use production signing keys in tests. Never place test keys in release artifacts.

| ID | Workflow | Success criteria |
|---|---|---|
| H-E2E-01 | Install fixture v1 from stable on a clean profile. | One active registry record; files match the manifest; no writes outside install, data, and cache roots; shortcut is correct; launcher lists the product. |
| H-E2E-02 | Verify installed files. | All sizes, hashes, permissions, and file types match. Missing, extra, or tampered files are detected. Symlink escape attempts are rejected. |
| H-E2E-03 | Launch the installed fixture. | Correct version and exit code; cold and warm startup measured; launch succeeds with network disabled; no unverified binary runs. |
| H-E2E-04 | Update v1 to v2 from signed stable metadata. | v1 retained until v2 passes its health check; registry and shortcuts point to v2; user data preserved; v2 launches; rollback remains available. |
| H-E2E-05 | Interrupt an update at each phase and recover. | Kill during download, after download, during extraction, before activation, during activation, after activation before registry commit, and during registry write. Restart recovers to a complete v1 or v2, never a mixed tree; journal resolves; temporary files clean up; retry succeeds. |
| H-E2E-06 | Repair a damaged installation. | Delete or corrupt one file, then repair the same version from a verified package. Version and channel do not change; user data is untouched; repaired product launches; offline repair works. |
| H-E2E-07 | Roll back from v2 to v1. | Retained package is verified; registry and shortcuts revert; v1 launches; user data remains compatible; missing or corrupt rollback bits fail closed. |
| H-E2E-08 | Uninstall the fixture. | Hub-owned files, registry entry, shortcut, service, and scheduled task are removed; user data is preserved by default; purge removes data only after explicit confirmation; no orphan process or file remains. |
| H-E2E-09 | Install and repair offline. | Network disabled; signed local bundle verifies; install, update, and repair behave as online; no network fallback; expired or revoked bundle fails closed. |
| H-E2E-10 | Reject signature failures. | Tampered manifest, tampered package, wrong key, and revoked key each fail before install; no execution; no registry change; diagnostic identifies the phase and signer without leaking secrets. |
| H-E2E-11 | Reject the wrong channel. | A stable request receiving beta metadata or a beta-key signature is rejected. An explicit channel switch succeeds only with correctly signed target-channel metadata. |
| H-E2E-12 | Reject the wrong architecture. | ARM64 package on x86_64 and the reverse are rejected before extraction or activation; the message is actionable; no partial install remains. |
| H-E2E-13 | Reject insufficient disk space. | Free space below the preflight threshold fails before mutation; no partial files or registry entry; temporary data is cleaned; the threshold is recorded. |
| H-E2E-14 | Reject corrupted packages. | Truncation, hash mismatch, zip bomb, and traversal archive each fail closed; extraction stays within bounds; no partial install remains. |
| H-E2E-15 | Repeat operations and run them concurrently. | Repeated install, update, repair, and uninstall converge with no duplicate records or shortcuts. Same-product mutations serialize; different products respect the concurrency bound; a stale lock is recovered safely. |
| H-E2E-16 | Self-update the Hub launcher. | A running launcher is not overwritten in place; helper or atomic replacement succeeds; restart shows the new version; failure restores the previous launcher; product installations are unaffected. |

Every negative workflow must also prove that no unverified binary executed, no active installation changed, and the diagnostic is actionable without exposing secrets.

## 7. Failure modes, debugging, regression, and platforms

### Failure modes

- Signature or integrity failure, expired metadata, replay, downgrade, or signer/channel mismatch.
- Network, TLS, proxy, redirect, or range-request failure; never fall back to insecure transport.
- Disk full, read-only filesystem, `noexec` mount, or permission denied.
- File locks, antivirus/EDR interference on Windows; Gatekeeper/quarantine on macOS; sandbox denial on Linux.
- Stale or corrupt lock, journal, registry, or staging directory.
- Activation failure after the new tree is staged; recover to the old version.
- Self-update of a running binary; require a helper or atomic replace.
- Missing dependency, version conflict, or unsupported minimum platform.
- User-data migration conflict; preserve the old data and fail closed.
- Package-manager ownership conflict on Linux.

### Debugging procedure

1. Reproduce on a clean profile and record Hub version, product, version, channel, target, operation ID, phase, and error code.
2. Export bounded diagnostics. Redact tokens, signed URLs, keys, and personal paths.
3. Verify the manifest signature and package hash independently with test keys. Do not disable checks to debug.
4. Inspect the registry and transaction journal. Compare staged, active, and retained trees; check locks and temporary directories.
5. Check disk space, permissions, filesystem features, antivirus/EDR, and platform logs.
6. Reproduce with the local fixture and an offline bundle to isolate network effects.
7. Identify the root cause, add a regression test, fix it, and rerun the affected end-to-end matrix.
8. If the environment cannot run a check, mark it `BLOCKED`; never record it as `PASS`.

### Regression tests

- Unit: manifest bounds and canonicalization, signature matrix, signer/channel binding, version ordering, path normalization, archive limits, disk preflight math, registry migrations.
- Integration: range and resume behavior, truncation, redirects, TLS failure, local offline bundle, filesystem fault injection, process kill at every transaction phase, stale lock recovery, concurrency.
- End-to-end: the matrix in section 6 on every claimed platform and installer format.
- Property and fuzz: manifest parser, archive extractor, and IPC decoder must not panic or exceed memory bounds.
- Security: wrong key, revoked key, expired metadata, replay, downgrade, traversal, symlink escape, zip bomb, and permission-denied cases.
- Platform: Authenticode verification, macOS notarization/Gatekeeper, Linux sandbox behavior.
- Every fixed defect gets a regression test before the fix is accepted.

### Cross-platform caveats

Windows:

- Authenticode-sign every executable, DLL, and helper; timestamp signatures; verify the signature after build. Expect SmartScreen reputation effects.
- Choose MSI, MSIX, or a custom installer in an ADR. Install per-user by default; require UAC only for machine-wide install.
- Handle file locks, antivirus/EDR, long paths, reserved names, case-insensitive collisions, and DLL search order.
- Self-update a running executable through a helper. Handle reboot-pending and uninstall registry entries.

macOS:

- Build a proper `.app` bundle; sign nested code inside-out; enable the hardened runtime; notarize and staple; verify with Gatekeeper.
- Do not run from the DMG. Expect translocation and quarantine; install to `/Applications` or the per-user Applications directory.
- Decide universal2 versus separate architectures. Test helpers, launchd items, TCC permissions, and keychain access.
- Do not write into the app bundle at runtime. Use atomic bundle replacement and a helper for self-update.

Linux:

- Choose deb, rpm, AppImage, Flatpak, or Snap in an ADR. Sandboxed formats may not be able to install host applications; use portals and explicit permissions.
- Use XDG per-user directories, `.desktop` files, icons, and systemd user units. Respect `noexec`, glibc/ABI compatibility, polkit, SELinux, and AppArmor.
- Atomic rename works only within one filesystem. Handle case-sensitive names, symlinks, and package-manager ownership conflicts.

Cross-compilation is not platform verification. If a platform check cannot run, mark it `BLOCKED`.

## 8. Release, documentation, and handoff

Release review is required when a change affects the manifest, package, registry, or IPC schema; channel policy; trust or signing; install layout; supported platform matrix; installer, self-update, rollback, repair, or uninstall semantics; or a user-visible install/update workflow.

Update the affected documents or record why no change is needed:

- `docs/product-specs/eqwec-hub.md` — planned product spec, channel policy, install layout, and registry schema.
- `docs/architecture/SYSTEM-ARCHITECTURE.md` and `docs/architecture/DEPENDENCY-MAP.md` — planned.
- `docs/security/THREAT-MODEL.md` — planned.
- Hub performance budgets and measurement records — planned.
- `README.md` status, `AGENTS.md` skill activation map, changelog, and release notes.

Version Hub independently. Version each schema separately and provide migrations. Do not describe an alpha or prototype as stable.

Handoff:

- `eqwec-engineering-core` — architecture, shared-crate ownership, security baseline, Git, versioning, and documentation lifecycle.
- `eqwec-quality-control` — pre-change, implementation, verification, performance, merge, and release gates. Provide the evidence matrix and the exact commands actually run.
- If this skill conflicts with core, core wins and the conflict is a skill defect.

## Red flags

- Trusting a manifest without verifying the signer identity, key epoch, and channel binding.
- Verifying the manifest but not the package size and hash, or verifying only after extraction.
- Installing into a non-atomic directory or writing directly over the active version.
- Deleting the previous version before the new version passes verification and a health check.
- Treating an interrupted download as complete, or resuming without re-verifying the full hash.
- Extracting archive paths without normalization, or following symlinks out of staging.
- Executing a downloaded binary "just to check its version" before verification.
- Accepting a downgrade or replay because the version string looks newer.
- Falling back to HTTP, disabling TLS verification, or skipping signatures on network failure.
- Storing signing keys, update tokens, or private URLs in the repository, fixtures, or CI logs.
- Requiring administrator rights for an install that could be per-user.
- Overwriting a running launcher instead of using a helper or atomic replacement.
- Mixing channels, or allowing a stable installation to consume a beta package.
- Updating the registry before files are active, or activating files before the journal is durable.
- Unbounded extraction without a disk-space preflight.
- Repair that silently changes version or channel, or uninstall that deletes user projects by default.
- Claiming update, rollback, or recovery is verified without interruption tests on a real fixture.
- Treating a passing unit test or a successful compile as proof of an install/update workflow.
- Deferring platform code signing, notarization, or sandbox testing until release day.
