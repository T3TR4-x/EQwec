# EQwec Hub — Product Specification

- **Status:** DRAFT product requirements. The Hub v1 wire formats and layout are normative in `docs/product-specs/eqwec-hub-schemas.md`; implementation status is tracked there and in `docs/development/QUALITY-GATES.md`.
- **Version:** `0.0.0` (planning).
- **Owner:** Hub product workstream.
- **Skill:** `.agents/skills/eqwec-hub/SKILL.md`.
- **Related:** `docs/product-specs/eqwec-hub-schemas.md`, `docs/adr/0011-hub-manifest-registry-and-layout.md`, `docs/architecture/SYSTEM-ARCHITECTURE.md`, `docs/security/THREAT-MODEL.md`, `docs/development/ROADMAP.md`.

## 1. Problem and target user

EQwec is an ecosystem of independently installed desktop products. Users need one trustworthy place to discover, install, launch, update, repair, and remove them without hunting for installers, risking unverified binaries, or losing projects during an interrupted update.

Target users:

- A new user installing their first EQwec product.
- An existing user managing several products and release channels.
- A user on a slow, intermittent, or offline connection.
- A user recovering from a failed or corrupted installation.
- A technical user who needs diagnostics and deterministic behavior.

## 2. Goals

- Be a real package manager and launcher, not a window of shortcuts.
- Verify authenticity and integrity before executing or installing anything.
- Make installation and updates transactional and recoverable.
- Work offline for installed products and for signed offline bundles.
- Preserve user projects and documents by default.
- Provide clear, actionable diagnostics without leaking secrets.
- Remain small, fast, and native.

## 3. Non-goals for the MVP

- A store with ratings, social features, or recommendations.
- Mandatory accounts, cloud sync, or telemetry.
- A general-purpose OS package manager.
- Delta updates in the first milestone (optional later, with full-package fallback).
- Machine-wide installation by default.
- Automatic installation without explicit user action.
- Managing non-EQwec applications.

## 4. Core user workflows

| ID | Workflow | Acceptance |
|---|---|---|
| U-01 | First launch | Launcher opens offline, shows cached registry, and does not block on a network update check |
| U-02 | Discover and install | User selects a product/channel, sees version, size, platform, and release notes, confirms, and installs a verified package |
| U-03 | Launch | Installed product launches with the documented arguments/environment and correct exit-code handling |
| U-04 | Update | New signed version is verified, staged, health-checked, activated, and the previous version is retained |
| U-05 | Interrupted update | Kill at any phase; restart recovers to a complete old or new state, never a mixed tree |
| U-06 | Repair | Corrupted files are restored from a verified package of the same version; user data is untouched |
| U-07 | Rollback | User returns to a retained verified version; compatibility and user-data impact are explained |
| U-08 | Uninstall | Hub-owned files, registry entries, shortcuts, services, and scheduled tasks are removed; projects are preserved |
| U-09 | Offline install/repair | Signed local bundle works with no network; expired/revoked bundle fails closed |
| U-10 | Diagnostics | User exports bounded, redacted diagnostics with operation ID and phase |

## 5. Functional requirements

### 5.1 Product catalog and channels

- A product has a stable immutable product ID, display name, description, supported platforms, and release channels.
- Channels: `stable`, `beta`, `alpha`, and internal `dev` (not distributed).
- A channel is bound to allowed signer identities and key epochs.
- The catalog may be cached; an offline launch shows cached state.
- The catalog does not execute code or include arbitrary command fields.

### 5.2 Manifest

The manifest must define at least:

```text
schema_version
product_id
product_version
channel
target { os, architecture, minimum_platform }
package { source, size, hashes[] }
signature { key_id, algorithm, signature, signing_time, key_epoch }
dependencies[] { product_id, version_constraint, package_hash }
metadata_sequence
issued_at, expires_at
release_notes (bounded, display-only)
```

Requirements:

- Canonical encoding; bounded field lengths, list counts, and nesting.
- Reject duplicate keys, unknown critical fields, inconsistent lengths, and trailing data.
- The signature covers source, size, hashes, target, channel, dependencies, and minimum platform.
- No executable logic, scripts, or arbitrary commands.
- Schema, canonicalization, and validation order are defined in `docs/product-specs/eqwec-hub-schemas.md` and ADR-0011.

### 5.3 Download and verification

- HTTPS with certificate validation; production sources allowlisted.
- Exact size and strong hash verified before extraction and before activation.
- Resume only with a strong validator; re-verify the full size and hash after resume.
- A partial download is never treated as complete.
- No insecure fallback on network or TLS failure.

### 5.4 Installation

- Per-user install by default; machine-wide install only with explicit elevation.
- Stage, verify, activate atomically, update the registry, and commit.
- Never write over an active tree.
- Keep the previous known-good version until the new version passes a health check.
- Preflight disk space for download, staging, rollback, and a safety margin.
- Bound extraction by entry count, path length, decompressed bytes, ratio, and time.
- Reject absolute paths, `..`, drive/UNC prefixes, NUL bytes, unsafe links, device files, and setuid/setgid bits.
- Verify resolved paths remain inside the staging root.

### 5.5 Registry and journal

- The registry is the authoritative record of active installations, channels, versions, roots, and signer identity.
- Schema-versioned, atomically written, and recoverable.
- At most one active record per product and profile.
- The journal records intent and phase; startup reconciles journal, locks, staging, and registry.
- Mutations are idempotent; repeated operations converge without duplicates.

### 5.6 Launch

- The launcher provides the documented executable path, arguments, environment, working directory, and single-instance handoff.
- A product does not require Hub at runtime unless the contract says so.
- A launch failure produces an actionable diagnostic and does not corrupt the registry.

### 5.7 Update, rollback, repair, uninstall

- Update uses the same verification and transaction path as install.
- Rollback reactivates a retained verified version; missing/corrupt bits fail closed.
- Repair restores the same version and channel; it never silently changes either.
- Uninstall removes Hub-owned artifacts; user projects are preserved unless purge is explicitly confirmed.
- Uninstall refuses to break a dependent installation without explicit user action.

### 5.8 Offline and diagnostics

- Installed products launch with no network.
- Offline bundles use the same signature, integrity, target, channel, and disk checks.
- Diagnostics are bounded, redacted, and exportable; they include operation ID, product, version, channel, target, phase, and error code.
- Never log tokens, signed URLs, keys, or personal paths.

## 6. Installation layout (planned)

The exact layout is normative in `docs/product-specs/eqwec-hub-schemas.md` and ADR-0011. Constraints:

- Per-user data and cache directories follow platform conventions (XDG on Linux, known folders on Windows, Application Support on macOS).
- Active, staging, download temp, rollback retention, logs, and cache are separated.
- Paths are scoped to the user/profile.
- No writes outside the documented roots.
- Atomic rename is used only within one filesystem; cross-filesystem moves are explicit and verified.
- Shortcuts and services are created only with consent and are removable.

## 7. Security and privacy

- Threat model: `docs/security/THREAT-MODEL.md`, Hub section.
- Signature, integrity, channel, target, replay, downgrade, traversal, symlink, archive bomb, and unverified execution controls are mandatory.
- Signing keys are never in the repository, CI logs, fixtures, or release artifacts.
- Test keys are non-production and isolated.
- No telemetry by default; diagnostics are user-initiated.
- Least privilege; per-user install by default.

## 8. Performance budgets

Provisional, to be validated:

| Metric | Provisional target | Method |
|---|---|---|
| Cold start to interactive, cached registry, network disabled | Under 1 second on mid-range hardware | Measured on each platform |
| Idle memory | No resident daemon unless justified; bounded RSS | Process-tree measurement |
| Update check | Background, non-blocking; bounded timeout | Network trace and UI responsiveness |
| Install/update overhead | Bounded by package size and disk; no unnecessary copies | I/O and disk measurement |
| Install size | Report compressed and installed bytes; investigate growth | Package manifest and measured tree |

No number is a release claim until measured with hardware, OS, build profile, workload, and command.

## 9. Accessibility and localization

- Keyboard-first navigation; visible focus; logical order.
- Screen-reader labels where the GUI framework exposes them.
- Clear error messages with a recovery action.
- Localization-ready strings; initial candidates English, Ukrainian, Slovak.
- Reduced-motion support; no decorative animation that blocks work.

## 10. Testing

Adopt the Hub skill's H-E2E-01..16 matrix. Minimum fixtures:

- A real fixture application with v1 and v2, version output, and nonzero failure exit.
- Packages for every claimed target.
- Stable and beta signed manifests.
- Offline bundle.
- Non-production test keys.
- Adversarial archives and tampered manifests.

Each workflow records registry snapshots, file-tree hashes, exit codes, bounded logs, and disk usage.

## 11. Release and versioning

- Hub has an independent version, starting at `0.0.0`.
- First milestone: `0.1.0-alpha.1` after the MVP gate.
- Manifest, registry, and IPC schemas are versioned separately with migrations.
- Release channels are signed and bound to allowed signers.
- No release until install/update/rollback/repair/uninstall are verified on each claimed platform.

## 12. Open questions

| Question | Owner | Needed by |
|---|---|---|
| GUI/rendering stack for Hub | Technology ADR | Phase 2 |
| Installer technology per platform | Packaging ADR | Phase 3 |
| Package container and canonical file manifest | Packaging ADR | Phase 3 |
| Delta updates in MVP or later | Product decision | Phase 3 |
| Machine-wide install support | Product/security decision | Phase 3 |
| Update server and signing infrastructure | Release/infra decision | Before first release |
| Crash reporting/diagnostics format | Diagnostics ADR | Phase 3 |
| Linux package format and sandbox interaction | Packaging ADR | Phase 3 |

## 13. MVP acceptance criteria

The Hub MVP is complete only when:

1. A real fixture application installs from a signed manifest on a clean profile.
2. Installed files match the manifest; tampering is detected.
3. The fixture launches and reports the expected version.
4. An update succeeds, retains the previous version, and passes a health check.
5. An interrupted update at every phase recovers to a complete old or new state.
6. Repair restores a corrupted file without changing version or user data.
7. Rollback returns to a verified previous version.
8. Uninstall removes Hub-owned artifacts and preserves user data.
9. Offline install/repair works; expired/revoked bundles fail closed.
10. Signature, hash, channel, target, traversal, and bomb attacks fail closed.
11. Diagnostics are bounded and redacted.
12. The above are verified on every claimed platform, or the platform is explicitly unsupported.
