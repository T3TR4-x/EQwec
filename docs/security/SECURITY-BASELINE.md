# Security Baseline

**Status:** approved baseline for planning. The ecosystem threat model exists (`docs/security/THREAT-MODEL.md`); product-specific runtime controls are not implemented. No product, Cargo workspace, or test harness exists, so every runtime security claim is `BLOCKED` until implementation evidence exists.

## 1. Security principles

- Security and privacy are design inputs, not review afterthoughts.
- Correctness and user-data integrity come first.
- Untrusted input is parsed with bounded resources and validated before use.
- Authentication, signature, and credential-store failures fail closed.
- Least privilege and platform sandboxing are preserved.
- Secrets are never logged, returned, committed, or placed in fixtures.
- A partial write, truncated stream, or unverified package is a failure.
- No telemetry, analytics, or remote collection without documented consent.

## 2. Trust boundaries

| Boundary | Untrusted side | Required control |
|---|---|---|
| Local user data | Files, paths, metadata, recent-file entries | Validate paths, scope by tenant/profile, atomic writes, permissions |
| Downloads and updates | Manifests, packages, metadata, redirects | Signature, hash, size, target, channel, replay/downgrade checks |
| Browser web content | Renderer processes, pages, extensions, downloads | Process isolation, sandbox, site isolation, IPC validation |
| Documents and media | Project files, SVG, images, video, audio, fonts | Bounded parsers, safe deserialization, no script/network execution |
| Plugins | Third-party binaries, presets, samples | Capability manifest, trust class, sandbox/out-of-process hosting |
| IPC | Local sockets, named pipes, shared memory | Authentication, versioning, authorization, size bounds |
| Native libraries | FFI boundaries, system APIs | Minimal `unsafe`, documented invariants, validation at the boundary |
| Build and release | Dependencies, CI, signing keys, artifacts | Pinned toolchain, advisory/license checks, key custody, reproducible builds |

## 3. Mandatory controls

### Input and files

- Validate paths against traversal, absolute-path injection, symlink escapes, drive/UNC prefixes, NUL bytes, and platform separators.
- Normalize and resolve before use; verify the final path remains inside the allowed root.
- Create temporary files with restrictive permissions and unpredictable names; clean them on failure.
- Use atomic writes for projects, caches, journals, and metadata.
- Bound file size, entry count, nesting depth, decompressed bytes, compression ratio, and parse time.
- Reject duplicate keys, unknown critical fields, inconsistent lengths, trailing data, and non-finite numbers where the contract requires it.

### Cryptography and integrity

- Use maintained, reviewed cryptographic libraries. Do not invent cryptography.
- Verify signatures over canonical bytes, not a re-serialization of untrusted data.
- Pin trust roots; support key rotation, epochs, and revocation.
- Verify package size and strong hash before extraction and again before activation.
- Keep private signing keys outside the repository, CI logs, fixtures, and release artifacts.
- Document key custody, rotation, revocation, and incident response.

### Process and privilege

- Preserve platform sandboxing and site isolation. Never disable a security control to reduce memory, improve startup, or simplify packaging.
- Isolate risky parsers, codecs, plugins, and browser renderers out of process where the platform and architecture allow.
- Run with least privilege; install per-user by default.
- Do not trust inherited environment variables, `PATH`, current directory, or unprivileged IPC peers.
- Validate every IPC message for version, operation, authorization, and size.

### Secrets and data

- Never log tokens, passwords, private keys, signed URLs, browsing history, project contents, personal paths, or audio recordings by default.
- Use the platform credential store; do not fall back to plaintext.
- Redact diagnostics and bound their size.
- Provide explicit user control over any diagnostics or telemetry.
- Preserve user projects and documents on uninstall by default; purge only after explicit confirmation.

### Failure behavior

- Fail closed on signature, integrity, authentication, authorization, and credential-store errors.
- Do not add an "ignore", "continue", or insecure fallback path to a production build.
- A development bypass must be isolated in a separate build, use test keys only, show a loud warning, and be impossible to enable in release artifacts.
- Recover to a complete old or complete new state, never a mixed tree.
- A non-critical diagnostic failure must not stop an authorized local workflow.

## 4. Dependency and supply-chain security

- Keep dependencies minimal and justified; record license, maintenance status, and replacement cost.
- Pin the toolchain and use `Cargo.lock` for the application workspace.
- Run advisory, license, and secret scans when the workspace and CI exist.
- Generate an SBOM and third-party notices before release.
- Review dependency updates; do not auto-merge a major update without compatibility tests.
- Treat build scripts, proc macros, and downloaded binaries as code with supply-chain risk.

## 5. Security testing

- Fuzz parsers for manifests, project files, SVG, media containers, fonts, and IPC.
- Test wrong key, revoked key, expired metadata, replay, downgrade, tampering, traversal, symlink escape, archive bomb, and unsafe deserialization.
- Test permission isolation, sandbox behavior, and IPC authorization.
- Test install/update/rollback/recovery under interruption.
- Pair automated scans with threat-model review and targeted adversarial tests.
- A clean scan is not proof of security.

## 6. Privacy

- Default to local storage and explicit user control.
- Do not collect browsing history, project contents, personal files, audio recordings, credentials, or prompts by default.
- If diagnostics or telemetry are implemented, document the data, purpose, retention, recipient, and opt-out.
- Provide export and deletion paths for user data.
- Do not require an account or network connection to launch an installed local application unless a documented product requirement justifies it.

## 7. Current state and open items

| Item | State |
|---|---|
| Product code | None; no runtime attack surface yet |
| Threat model | Not yet written; required before product implementation |
| Security baseline | This document |
| Dependency inventory | Candidate technologies under research; see `docs/research/LEGAL-LICENSE-REVIEW.md` when complete |
| `cargo audit` / `cargo deny` | BLOCKED until workspace exists |
| Secret scanning in CI | BLOCKED until CI exists |
| Signing infrastructure | None; BLOCKED until release phase |
| Security gate | Open |

This baseline authorizes planning, not product implementation. The security gate closes only after the threat model, architecture, and release controls are reviewed against the actual design.
