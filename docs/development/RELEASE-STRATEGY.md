# Release Strategy

**Status:** approved policy. No release exists. No product has a package, installer, signature, or published artifact.

## 1. Release principles

- Each product releases independently. A shared-crate change does not force a release of every product.
- A release is a verified artifact plus metadata, not a branch name or a commit message.
- Release only from a scoped `release/<product>/vX.Y.Z` branch assembled from `production`.
- Never describe a prototype, alpha, or release candidate as stable.
- Signing keys stay outside the repository. Test keys never enter a release artifact.
- A release without a rollback plan and a verified install path is not approved.

## 2. Release channels

| Channel | Audience | Stability | Update behavior |
|---|---|---|---|
| `stable` | General users | Released and supported | Default; only stable versions |
| `beta` | Opt-in testers | Feature-complete for the milestone, may have defects | Explicit opt-in; signed beta metadata |
| `alpha` | Developers and early testers | Incomplete, may break projects | Explicit opt-in; clear warnings |
| `dev` | Internal builds | Unverified | Never distributed as a supported release |

Channel policy is signed and bound to allowed signers. A stable installation must not consume a beta package without an explicit, signed channel switch.

## 3. Release assembly

1. Confirm the product version and milestone scope.
2. Create `release/<product>/vX.Y.Z` from the current `production`.
3. Promote only reviewed product changes and required, tested dependencies. Use a reviewed cherry-pick sequence or scoped commits.
4. Verify that unrelated unfinished work did not enter.
5. Update version metadata, changelog, release notes, compatibility notes, and dependency/engine versions.
6. Run the release gate and the product's end-to-end workflows.
7. Build packages reproducibly for every claimed target.
8. Verify hashes, signatures, installer behavior, install, launch, update, rollback, repair, and uninstall.
9. Merge into `production` with protected review.
10. Tag the release with a namespaced tag.
11. Publish through an authorized channel only after explicit authorization.
12. Verify installation from the published artifact, not only from the local build.
13. Reconcile `production` back into `integration` and the affected `dev-*` branch.

## 4. Release metadata

Each release records:

```text
product, version, channel, release date, commit SHA, tag,
build environment and toolchain, dependency/engine versions,
artifact names, sizes, hashes, signature/key identity,
supported platforms and architectures, minimum OS versions,
changelog, compatibility notes, known limitations,
rollback target, approval reference
```

The manifest is versioned, canonically encoded, bounded, and signed. The signature covers every security-relevant field, including package source, size, hashes, target, channel, dependencies, and minimum platform.

## 5. Packaging by platform

### Windows

- Signed installer (technology chosen by ADR) and signed executables/DLLs/helpers.
- Timestamp signatures; verify after build.
- Per-user install by default; UAC only for an approved machine-wide install.
- Handle file locks, antivirus/EDR, long paths, reserved names, case-insensitive collisions, and DLL search order.
- Self-update a running executable through a helper or atomic replacement.

### macOS

- Proper `.app` bundle; sign nested code inside-out; hardened runtime; notarize and staple.
- Verify with Gatekeeper. Do not run from the DMG; expect translocation and quarantine.
- Decide universal2 versus separate architectures; test helpers, launchd items, TCC permissions, and keychain access.
- Do not write into the app bundle at runtime; use atomic bundle replacement and a helper for self-update.

### Linux

- Choose deb, rpm, AppImage, Flatpak, or Snap by ADR. Sandboxed formats may not be able to install host applications; use portals and explicit permissions.
- Use XDG per-user directories, `.desktop` files, icons, and systemd user units.
- Respect `noexec`, glibc/ABI compatibility, polkit, SELinux, and AppArmor.
- Atomic rename works only within one filesystem. Handle case-sensitive names, symlinks, and package-manager ownership conflicts.

Cross-compilation is not platform verification. If a platform check cannot run, mark it `BLOCKED`.

## 6. Update and rollback

- Signed release manifests and cryptographic verification.
- Atomic or recoverable updates; never write over the active tree.
- Retain the previous known-good version until the new version passes a health check.
- Interrupted updates recover to a complete old or complete new state, never a mix.
- Downgrades require a signed policy allowance and explicit user confirmation.
- Replay, stale metadata, wrong signer, wrong channel, and wrong architecture are rejected before mutation.
- Offline bundles use the same signature, integrity, target, channel, and disk checks as online installs.

## 7. Release gate

A release is blocked if any of the following is unresolved:

- Critical defect or critical security finding.
- Missing mandatory skill, invalid skill, or blocked quality gate.
- Failed essential end-to-end workflow.
- Unverified package integrity or signature.
- Failed install, launch, update, rollback, repair, or uninstall check.
- Unsupported platform claimed without evidence.
- Inaccurate changelog or release notes.
- Missing authorization to publish.
- No rollback plan.

## 8. Publishing and authorization

- No remote is configured and nothing has been pushed. Publishing is `BLOCKED` until an authorized remote, channel, and signing infrastructure exist.
- Do not invent a GitHub URL, create a public repository, or upload artifacts without explicit authorization.
- Signing keys are generated and stored outside the repository, with documented custody, rotation, and revocation.
- Release approvals name the accountable owner, the approved artifact hashes, the channel, and the release date.

## 9. Post-release

- Monitor installation and update failures through user-initiated diagnostics; do not add telemetry without documented consent.
- Verify that the published artifact installs on a clean profile.
- Record known issues and compatibility notes.
- If a critical defect is found, stop the rollout, publish an advisory if appropriate, and prepare a hotfix from `production`.
- Reconcile the release commit back into `integration` and `dev-*`.

## 10. Current state

| Item | State |
|---|---|
| Release branch | None |
| Production branch | Not created |
| Installer/package | None |
| Signing key | None |
| Release manifest | None |
| Published artifact | None |
| Remote | None |
| Release gate | `BLOCKED` until a product exists |

The first release will be a narrow, verified milestone, not a claim that the ecosystem is complete.
