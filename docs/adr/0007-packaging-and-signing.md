# ADR-0007: Packaging, updates, and signing

- **Status:** ACCEPTED (with a release gate; keys and tooling do not exist yet)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0001, `docs/security/SECURITY-BASELINE.md`, `docs/security/THREAT-MODEL.md`, `docs/development/RELEASE-STRATEGY.md`

## 1. Context

EQwec Hub installs, updates, repairs, and removes products. Every artifact must be authentic, complete, and safe to activate. A compromised update is the highest-impact failure in the ecosystem: it can execute arbitrary code with the user's privileges and persist across releases.

The project has no signing keys, installer, or release artifact, and packaging is unimplemented. Its CI workflow runs on GitHub Actions and passed on macOS, Ubuntu, and Windows on 2026-11-07. This ADR defines the packaging and signing architecture so implementation can proceed without inventing trust ad hoc.

## 2. Requirements

- Per-user installation by default; no unnecessary elevation.
- Signed release manifests and signed artifacts.
- Integrity verification before extraction, activation, and launch.
- Atomic or recoverable updates with rollback.
- Offline installation where supported.
- Stable, beta, and development channels.
- Downgrade and replay resistance.
- Platform-native packaging on Windows, macOS, and Linux.
- SBOM and third-party notices per artifact.
- Key custody, rotation, and revocation.
- No private keys in the repository, CI logs, or release artifacts.
- A release must be installable and verifiable from the published artifact.

## 3. Alternatives considered

### A. Platform stores only (Microsoft Store, Mac App Store, Flathub)

Stores provide distribution, updates, and sandboxing, but restrict package formats, update control, and Hub's ability to install arbitrary EQwec products. They also create platform silos and review delays. Use stores as optional channels, not the only path.

### B. Custom installer and updater for all platforms

Maximum control and a consistent Hub experience. Requires platform-specific packaging, signing, notarization, sandbox/entitlement work, and a security response process. This is the primary model.

### C. MSIX on Windows

Modern packaging, clean install/uninstall, and update support. MSIX restrictions (filesystem/registry virtualization, package identity) complicate a launcher that installs and manages other applications. Use MSI for the launcher initially; evaluate MSIX for Store distribution later.

### D. Sparkle on macOS

Mature update framework, but it duplicates Hub's update logic and signing model. Hub should own updates; use Developer ID, hardened runtime, notarization, and signed update packages.

### E. Full TUF repository

The Update Framework provides strong roles, delegation, and compromise resilience. It is the right long-term model but adds metadata roles, key management, and client complexity. Start with a TUF-inspired signed manifest and evaluate full TUF before scaling the release channel.

## 4. Evidence

- Authenticode is the Windows code-signing mechanism; Apple requires Developer ID signing, hardened runtime, notarization, and stapling for direct macOS distribution.
- MSIX and MSI have different installation and virtualization models; a launcher that manages other applications needs broad filesystem control.
- Flatpak provides sandboxing and cross-distro packaging but restricts a launcher's ability to install arbitrary apps outside its sandbox.
- The Update Framework specifies root, targets, snapshot, and timestamp roles, key rotation, and rollback protection.
- minisign and Ed25519 provide compact signatures; the exact tool must be verified before use.
- No EQwec signing key, certificate, notarization account, or release channel exists. All release steps are `BLOCKED` until they do.

## 5. Decision

### 5.1 Platform packaging matrix

| Platform | Initial package | Signing | Update model | Notes |
|---|---|---|---|---|
| Windows | Per-user MSI (WiX Toolset) | Authenticode | Hub downloads and verifies signed package; MSI executes per-user | Optional MSIX for Store later; no elevation by default |
| macOS | `.app` in signed DMG or ZIP | Developer ID, hardened runtime, notarization, stapling | Hub downloads and verifies signed package; replaces the app bundle atomically | Direct distribution first; Mac App Store later if sandbox constraints allow |
| Linux | Portable `tar.zst` with `.desktop` entry and icons | minisign/Ed25519 over artifacts and manifests | Hub downloads, verifies, stages, and activates | `.deb`/`.rpm` later; Flatpak for sandboxed distribution with documented Hub limitations |

### 5.2 Release manifest

The manifest is a JSON envelope with a detached or embedded signature over the exact payload bytes. Do not sign a re-serialization.

Required fields:

- `schema_version`
- `product_id`
- `product_version`
- `channel` (`stable`, `beta`, `dev`)
- `platform` and `architecture`
- `min_os_version`
- `package_url` or offline source
- `package_size`
- `package_sha256`
- `published_at`
- `expires_at`
- `dependencies`
- `release_notes_url`
- `key_id`
- `signature`

Rules:

- Verify the signature over the raw payload before parsing untrusted fields.
- Verify package size and SHA-256 before extraction and again before activation.
- Bind the signature to product, version, channel, platform, and architecture.
- Reject unknown critical fields, duplicate keys, inconsistent lengths, and expired metadata.
- Store the last accepted version per channel and reject downgrades unless a signed downgrade policy allows them.
- Provide a signed revocation or kill-switch mechanism for a bad release.

### 5.3 Key custody

- **Root key:** offline, used only to certify release keys and revocation lists.
- **Release key:** online or CI-held, used to sign manifests and artifacts; certified by the root.
- **Rotation:** root-signed key rotation with overlapping validity.
- **Revocation:** root-signed revocation list; clients check it before accepting a release.
- **Storage:** hardware token, HSM, or platform key store; never in the repository, CI logs, fixtures, or release artifacts.
- **Incident response:** documented procedure for key compromise, bad release, and rollback.

Full TUF roles and delegation are the target; the first implementation is a TUF-inspired manifest with root/release separation. Do not claim TUF compliance until the metadata model is implemented and tested.

### 5.4 Installation and update flow

1. Resolve product, channel, platform, and architecture.
2. Check platform compatibility, disk space, and dependencies.
3. Download the manifest and package.
4. Verify signature, size, hash, channel, version, expiry, and revocation.
5. Stage into a temporary directory with restrictive permissions.
6. Verify staged files and the package layout.
7. Activate atomically; retain the previous version for rollback.
8. Register the installed product and health-check it.
9. On failure, recover to the previous complete state.
10. Record the operation in a local audit log with redacted paths.

Interrupted downloads and updates must resume or restart safely. A partial package is never activated.

### 5.5 Offline installation

- Support a signed offline package and manifest on removable media.
- Verify the same signatures and hashes as online installation.
- Do not require a network connection to launch an installed product.
- Provide a documented export/import path for offline machines.

### 5.6 SBOM and notices

- Generate an SBOM (SPDX or CycloneDX) per artifact.
- Include third-party notices, licenses, and source offers required by LGPL/MPL.
- Record the exact toolchain, dependency lockfile, and build flags.
- Publish the SBOM and notices with the artifact, not only in the repository.

## 6. Security requirements

- Fail closed on signature, hash, size, channel, expiry, revocation, or dependency errors.
- Never execute a downloaded binary before verification and staging.
- Never trust a manifest field before signature verification.
- Validate archive paths and reject traversal, absolute paths, symlinks, and duplicate entries.
- Bound archive size, entry count, compression ratio, and extraction time.
- Use TLS with certificate validation for downloads; do not disable validation for mirrors.
- Keep signing keys out of the repository and CI logs.
- Log install/update operations without secrets or personal paths.
- Test wrong key, revoked key, expired metadata, replay, downgrade, tampering, truncation, traversal, symlink escape, archive bomb, and interrupted update.

## 7. Consequences

**Positive**

- A consistent Hub-owned install/update experience across platforms.
- Signed manifests and artifacts provide a clear trust chain.
- Atomic staging and rollback protect user data.
- Offline installation supports users without reliable networks.
- Per-user installation avoids unnecessary elevation.

**Negative**

- EQwec owns packaging, signing, notarization, and update infrastructure on three platforms.
- Certificate and key custody are organizational obligations, not code.
- MSI custom actions are a security risk if not carefully written.
- Flatpak limits Hub's ability to manage arbitrary apps.
- Full TUF is deferred; the first model must be reviewed carefully.

**Security**

- The update channel is the highest-value target; key custody and revocation are critical.
- Signature verification must happen before parsing or extraction.
- Rollback and replay protection are required, not optional.
- A compromised release key must be revocable without reinstalling every product.

**Performance**

- Measure download, verification, extraction, activation, and startup impact.
- Keep installers as small as practical without removing required runtime components.
- Do not claim a size or speed improvement without a measured baseline.

**Licensing**

- WiX, Flatpak, AppImage, minisign, and platform signing tools have their own licenses; verify before use.
- LGPL/MPL source offers and notices are part of the artifact.
- Codec/DRM components follow ADR-0004.

**Maintenance**

- Signing certificates expire; rotation must be scheduled.
- Platform packaging changes with OS releases; test on each supported version.
- The manifest schema is versioned and migrated deliberately.

## 8. Reconsideration criteria

Revisit this ADR if:

- MSIX or a store-only model becomes required for distribution.
- Full TUF is adopted or a different update framework is chosen.
- Flatpak becomes the primary Linux channel and Hub's scope changes.
- A platform requires a different signing or notarization mechanism.
- The update manifest cannot provide rollback/replay protection without excessive complexity.
- Key custody cannot be funded or operated safely; then the release model must change.

## 9. Links

- `docs/security/SECURITY-BASELINE.md`
- `docs/security/THREAT-MODEL.md`
- `docs/development/RELEASE-STRATEGY.md`
- `docs/research/LEGAL-LICENSE-REVIEW.md`
- ADR-0001

## 10. Implementation note — developer-preview packaging (2026-11-07)

The first launcher artifacts follow this ADR's direction but are not releases:

- macOS: an ad-hoc-signed `.app` inside a DMG built by `packaging/eqwec-hub/macos/build-dmg.sh`. Developer ID signing, hardened runtime, notarization, and stapling remain `BLOCKED` on certificates and an Apple account.
- Windows: a per-user WiX v4 MSI installed to `%LOCALAPPDATA%\Programs\EQwec Hub` with no elevation, built by `packaging/eqwec-hub/windows/build-msi.ps1`. Authenticode signing remains `BLOCKED`.
- Linux: a `.deb` with a `.desktop` entry and a `tar.zst` archive built by `packaging/eqwec-hub/linux/build-deb.sh`. The §5.1 matrix listed `.deb` as a later addition; it is included now because it makes install and self-check verifiable in CI. Flatpak and RPM remain future work.
- The artifacts are unsigned developer builds. They are not published, carry no release channel, and must not be described as releases. Run `42398364283` built all three and passed the platform smoke checks: the macOS DMG mounted and self-checked, the Windows MSI installed/self-checked/uninstalled, and the Linux `.deb` installed/self-checked/removed.
