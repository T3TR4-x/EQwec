# Security Baseline

## Trust boundaries

Identify the principal, resource, and security outcome for every boundary:

- Local user ↔ application UI.
- Application ↔ local project files and databases.
- Application ↔ downloaded packages and updates.
- Browser application ↔ Chromium renderer/GPU/network/utility processes.
- Application ↔ plugins, extensions, and native libraries.
- Application ↔ IPC peers.
- Build/release infrastructure ↔ signing keys and distribution.
- Untrusted document/media/parser input ↔ in-process memory.

A boundary without an explicit validation rule is a defect waiting to happen.

## Input handling

- Validate length, type, range, encoding, and structure before use.
- Bound parser depth, item count, decompressed size, and memory growth.
- Reject path traversal, absolute-path injection, Windows drive/UNC prefixes, symlink escapes, and NUL bytes where the contract forbids them.
- Treat duplicate keys, inconsistent lengths, unknown enum values, and truncated streams as errors.
- Do not execute, load, or install an artifact before verifying its integrity and authenticity.

## Secrets

- Never log, return, commit, or place in fixtures: API keys, session tokens, proxy tokens, passwords, private signing keys, prompts, or raw credentials.
- Redact at the logging boundary, not at every call site. Test the redaction.
- Store secrets in the platform credential store where available. Do not invent a plaintext secrets file.
- Document key custody, rotation, revocation, and recovery. Keep production private keys out of the repository and CI logs.
- Fail closed on credential-store or authentication failure. Do not fall back to an insecure default.

## Files and persistence

- Use restrictive permissions for temporary files and directories.
- Create temp files in a private directory with unpredictable names; clean them on success and failure.
- Write project files atomically: write to a temp file in the same directory, fsync where the platform requires it, then rename. Keep a recoverable previous version when the format supports it.
- Validate that a path stays within its tenant/profile root after resolving symlinks.
- Do not follow a symlink out of a managed root unless the user explicitly chose that target and the operation documents it.
- Encrypt or otherwise protect sensitive local data according to the threat model; do not claim encryption that is not implemented.

## Network and updates

- Use secure transport with certificate validation. Do not disable verification for convenience.
- Verify a signed manifest and the package hash before installation.
- Verify the signer identity and the channel; do not accept a valid signature from an unexpected key.
- Stage the installation, validate it, then activate it atomically or with a rollback path.
- Handle interruption at every step: download, verify, stage, activate, register.
- Do not execute an unverified binary because a server manifest references it.

## Browser and native isolation

- Preserve Chromium sandboxing and site isolation. Do not disable them to reduce memory or simplify packaging.
- Treat web content as untrusted. Validate IPC messages from renderer processes as untrusted input.
- Keep the browser application process, renderer processes, GPU process, network/utility processes, and extension processes separated according to the engine's security model.
- Isolate native libraries and plugins. Define the permissions and data they can access.
- Do not expose a filesystem, shell, or network capability to a plugin unless the product explicitly grants it.

## Dependencies and supply chain

- Record each dependency's purpose, license, version policy, and maintenance status.
- Run the repository's advisory and license checks.
- Pin or lock dependencies for applications. Review lock-file changes.
- Do not add a dependency for a few lines of code that the standard library or an existing dependency already provides.
- Treat a clean scanner as one signal, not proof of security.

## Failure behavior

- Fail closed on authentication, authorization, signature, integrity, and credential-store errors.
- A non-critical diagnostic or telemetry failure must not block an authorized local workflow.
- A truncated stream, partial write, or unverified package is a failure, never a success.
- Do not leak internal paths, secrets, or raw payloads in a user-facing error.
- Record enough bounded diagnostic context to reproduce the failure without exposing sensitive data.

## Security testing

- Dependency advisory and license checks.
- Static analysis and Clippy security-relevant lints.
- Fuzzing for parsers and file formats.
- Malformed-input and adversarial-file tests.
- Update signature and integrity verification tests.
- Path traversal and symlink escape tests.
- Permission and tenant-isolation tests.
- Regression tests for every fixed security defect.

Report findings with severity, evidence, affected component, remediation, and verification state. A clean scan is not a security proof.
