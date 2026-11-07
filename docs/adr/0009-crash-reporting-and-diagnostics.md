# ADR-0009: Crash reporting and diagnostics

- **Status:** ACCEPTED (implementation gated on the workspace and a privacy review)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0001, ADR-0007, `docs/security/SECURITY-BASELINE.md`, `docs/security/THREAT-MODEL.md`

## 1. Context

EQwec products will crash. Users need a way to report a crash, and engineers need enough information to diagnose it. Crash reporting is also a privacy and security boundary: a crash dump can contain project data, browsing history, file paths, credentials, or memory from other processes. A default-on upload service would violate the project's local-first and no-telemetry-without-consent rules.

The project has no product, no crash handler, and no remote service. This ADR defines the diagnostic architecture so products do not invent incompatible or unsafe reporting later.

## 2. Requirements

- Local crash capture on Windows, macOS, and Linux.
- A human-readable crash record with a redacted log bundle.
- No automatic upload and no third-party analytics SDK by default.
- Explicit, informed user consent before any export or upload.
- Redaction of secrets, personal paths, project contents, browsing history, and clipboard data.
- Bounded file size, retention, and collection time.
- Crash capture that does not deadlock, recurse, or mask the original fault.
- Symbolication support for EQwec binaries and approved third-party components.
- A clear separation between local diagnostics and any future telemetry.
- A documented data list, retention, and deletion policy for any opt-in export.
- Crash handling that works when the main process is compromised or hung, where the platform allows.

## 3. Alternatives considered

### A. Third-party crash service (Sentry, Crashpad + hosted backend, Backtrace)

Mature symbolication, aggregation, and alerting. But it introduces a data processor, a network dependency, default-upload risk, and vendor lock-in. A hosted service may be acceptable later as an explicit opt-in, but not as the default architecture.

### B. Platform crash reporters (Windows Error Reporting, macOS CrashReporter, systemd-coredump)

Free, OS-integrated, and familiar. They send data to the platform vendor under the user's OS settings, not to EQwec, and they do not provide product-specific context. Useful as a fallback, not as the primary product diagnostic.

### C. Self-hosted crash collector

Full control over data, retention, and symbolication. Requires infrastructure, on-call, security review, and a privacy policy. Deferred until there is a released product and an approved operating budget.

### D. Local-only crash records with manual export

Captures a crash record and redacted logs locally. The user chooses whether to attach them to a bug report or export a bundle. No network service, no default upload, no third-party processor. Lowest privacy risk and lowest operational cost. Chosen for the first release.

### E. No crash reporting

Simplest, but leaves engineers blind and users without a recovery path. Rejected.

## 4. Evidence

- Crash dumps and minidumps can contain arbitrary process memory, including secrets and user data; they must be treated as sensitive.
- Windows Error Reporting, macOS CrashReporter, and systemd-coredump are OS mechanisms with their own privacy terms and are not controlled by EQwec.
- The project's security baseline forbids telemetry and remote collection without documented consent.
- No EQwec crash handler, symbol server, or collector exists; all runtime claims are `BLOCKED` until implemented.
- The exact crash-handler crate or platform API must be selected and reviewed at implementation time; this ADR does not approve a specific dependency.

## 5. Decision

### 5.1 Local crash record

On an unhandled fault, the product writes a crash record to a per-user, per-product diagnostics directory:

```text
diagnostics/
  crashes/<timestamp>-<id>/
    crash.json          # product, version, build ID, OS, architecture, fault summary
    report.txt          # human-readable summary
    log.txt             # redacted recent log tail
    dump.<ext>          # optional minidump, only if the user enables dumps
  sessions/
    latest-session.json # clean-shutdown marker and version
```

Rules:

- `crash.json` contains product ID, product version, build ID, commit, OS version, architecture, crash time, fault type, module list with hashes, and a correlation ID. It does not contain project content, browsing history, clipboard data, or credentials.
- `log.txt` is a bounded tail of the structured log with redaction applied.
- A minidump is optional and off by default; if enabled, the user is warned that it may contain sensitive memory.
- The record is written with restrictive permissions and is readable only by the user.

### 5.2 Redaction

Redact before writing:

- secrets, tokens, passwords, API keys, and session cookies;
- full home-directory paths and usernames where they are not needed;
- project names and contents;
- URLs and browsing history;
- clipboard contents;
- environment variables that may contain credentials;
- plugin license keys and account identifiers.

Redaction is deny-by-default for unknown fields. A field is included only if it is on the documented allowlist.

### 5.3 User flow

1. On the next launch after a crash, show a clear notice: the product crashed, a local record exists, and nothing was uploaded.
2. Offer: view the record, export a bundle, delete the record, or dismiss.
3. Export produces a ZIP with the redacted record and logs; the user chooses the destination.
4. Upload, if ever added, requires explicit consent, a documented data list, retention, and deletion policy, and a separate ADR.
5. Never upload in the background, on a timer, or as a condition of use.

### 5.4 Retention and deletion

- Keep at most a bounded number of crash records (for example, the ten most recent) and a bounded total size.
- Delete older records automatically.
- Provide a "delete all diagnostics" action.
- Uninstall preserves user data by default; diagnostics deletion is explicit or documented.

### 5.5 Symbolication

- Build IDs and symbols are produced per release and stored with the release artifacts.
- Symbol servers, if added, are authenticated and access-controlled; symbols are not public by default.
- Third-party symbols are used only under their licenses.
- Do not claim symbolication for a platform or component until it is tested.

### 5.6 Relationship to telemetry

- Diagnostics are not telemetry.
- No usage analytics, feature tracking, or remote logging is enabled by default.
- Any future telemetry requires a separate ADR, explicit consent, a data list, retention, deletion, and a privacy review.

## 6. Security requirements

- Treat crash records and dumps as sensitive user data.
- Bound file size, log tail, module list, and collection time.
- The crash handler must not allocate unboundedly, deadlock, or recurse.
- Do not run the crash handler with elevated privileges.
- Do not load untrusted modules or symbol files during capture.
- Validate the diagnostics directory path and permissions.
- Encrypt or restrict dumps at rest where the platform supports it.
- Test that a crash in the crash handler does not loop.
- Test redaction with seeded secrets and personal data.

## 7. Consequences

**Positive**

- No default upload, no third-party processor, no hidden data collection.
- Users can inspect and control what is shared.
- Engineers get structured, redacted crash context.
- The design leaves room for an opt-in collector later without changing the product contract.

**Negative**

- Manual reporting yields fewer crash reports than automatic upload.
- Redaction and symbolication are ongoing engineering work.
- Minidumps are off by default, which limits some diagnoses.
- A future hosted collector will require infrastructure and a privacy review.

**Security**

- Crash data is a high-risk artifact; redaction and access control are mandatory.
- A crash handler is privileged code and must be minimal and tested.
- Do not include secrets, project contents, or browsing history by default.

**Performance**

- Crash capture must not affect normal runtime; handlers are installed once and dormant.
- Measure capture time and record size on the defined hardware profiles.
- Do not claim a startup impact without measurement.

**Licensing**

- The crash-handler dependency must be permissively licensed and reviewed at admission.
- Platform crash APIs follow OS terms.
- Symbol files and third-party symbols follow their own licenses.

**Maintenance**

- Crash formats and symbol storage are versioned.
- Retention and deletion policies must be enforced in code, not only documented.
- A future collector requires a separate ADR and operating plan.

## 8. Reconsideration criteria

Revisit this ADR if:

- Users or support cannot diagnose crashes with local records alone.
- An opt-in collector is approved with a privacy review and operating budget.
- A platform crash API changes or becomes mandatory.
- Minidumps become necessary by default for a product.
- Symbolication cannot be provided under acceptable licenses.
- Regulatory requirements change the data-handling model.

## 9. Links

- `docs/security/SECURITY-BASELINE.md`
- `docs/security/THREAT-MODEL.md`
- `docs/development/RELEASE-STRATEGY.md`
- ADR-0007
- ADR-0001
