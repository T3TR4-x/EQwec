# ADR-0011: Hub manifest, registry, journal, and installation layout

- **Status:** ACCEPTED (implementation gated on schema tests and platform fixtures)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0006, ADR-0007, ADR-0009, ADR-0010, `docs/product-specs/eqwec-hub.md`, `docs/product-specs/eqwec-hub-schemas.md`

## 1. Context

The Hub product specification defines the required manifest fields and states that the concrete wire format, canonicalization, registry schema, journal schema, and installation layout require an ADR before implementation. No Hub crate or application exists. Implementing these formats ad hoc would make signatures ambiguous, registry writes unrecoverable, and future migrations unsafe.

The formats are security contracts. The manifest signature must cover every security-relevant field, the registry must be the authoritative record of active installations, and the journal must let startup recover to a complete old or complete new state after an interruption.

## 2. Requirements

- A canonical, bounded manifest encoding that rejects duplicate keys, unknown critical fields, inconsistent lengths, and trailing data.
- A signature that covers product identity, version, channel, target, package source, size, hashes, dependencies, metadata sequence, and validity times.
- A trust model that binds a key ID and key epoch to allowed channels and supports revocation.
- An authoritative registry with at most one active record per product and profile.
- A durable journal that records intent and phase before mutation.
- Atomic activation that never writes over an active tree.
- A per-user installation layout that separates active, staging, download, rollback, logs, and cache.
- Schema versioning and migration for manifest, registry, and journal independently.
- Human-inspectable formats where practical; no executable logic in any schema.
- Testability with non-production keys and a real fixture product.

## 3. Alternatives considered

### 3.1 Wire format

**A. JSON (chosen).** Ubiquitous, inspectable, well supported by Rust, and adequate for a small manifest. Requires strict parsing and an explicit canonical form.

**B. CBOR.** Compact and deterministic encoding is possible, but tooling and human inspection are weaker, and canonical CBOR rules add complexity without a measured size problem.

**C. TOML.** Readable, but nested arrays of tables and strict duplicate-key handling are awkward for a signed security document.

**D. Protocol Buffers.** Strong schema and compact encoding, but the schema compiler and generated code add build complexity; unknown-field behavior must be configured carefully.

### 3.2 Canonicalization

**A. Schema-ordered canonical JSON (chosen).** The schema defines the exact field order, string escaping, integer form, and timestamp form. The signer signs those bytes. Because the schema is fixed and small, this is simpler and easier to audit than a general canonicalization algorithm.

**B. RFC 8785 JSON Canonicalization Scheme.** General and standardized, but it requires correct number and Unicode handling and would make the signature depend on a general-purpose canonicalizer. The manifest has no floating-point numbers, so the benefit is limited.

**C. Sign the raw manifest bytes.** Simplest to implement, but formatting, whitespace, and key order become signature-relevant. A harmless reformat invalidates the signature, and duplicate-key handling becomes ambiguous.

### 3.3 Registry storage

**A. Versioned JSON file with atomic replace (chosen).** The registry is small, human-inspectable, and can be replaced atomically. It matches ADR-0006's rule that SQLite is for catalogs and caches, not the authoritative installation record.

**B. SQLite.** Strong transactional behavior, but it adds a native dependency and a second source of truth for a small record set. Deferred unless registry size or concurrency proves otherwise.

**C. Per-product files.** Simple isolation, but cross-product invariants (dependency checks, one active record per profile) become harder to enforce atomically.

### 3.4 Journal storage

**A. Append-only JSON Lines with fsync (chosen).** Each phase record is a complete line; a partial trailing line is discarded on recovery. Bounded by operation count and size. The registry remains the committed state.

**B. Single JSON file replaced atomically.** Simple, but rewriting the whole journal on every phase increases the chance of losing prior phase records and makes append-only auditing harder.

**C. SQLite write-ahead log.** Durable and transactional, but again adds a native dependency for a small local journal.

### 3.5 Signature algorithm

**A. Ed25519 (chosen).** Small keys and signatures, deterministic signatures, a well-reviewed Rust implementation, and no parameter choices. Matches ADR-0007.

**B. ECDSA P-256.** Widely supported by platform tooling, but signature encoding and nonce handling are more error-prone.

**C. RSA.** Broad compatibility, but large keys and signatures and a larger implementation surface.

## 4. Evidence

- RFC 8032 defines Ed25519; RFC 8259 defines JSON; RFC 3339 defines timestamps.
- The Update Framework documents signed metadata, key roles, expiry, and revocation as a proven update-security model.
- ADR-0006 selects a versioned JSON manifest/document plus content-addressed assets for project data and reserves SQLite for catalogs and caches.
- ADR-0007 selects Ed25519 with root/release key separation, rollback protection, and signed manifest envelopes.
- ADR-0009 requires bounded, redacted diagnostics and no default upload.
- The Hub product specification requires canonical encoding, bounded parsing, duplicate-key rejection, exact size and hash verification, atomic activation, a journal, and a per-user layout.
- No Hub code, fixture product, test key, registry, or journal exists. Every runtime claim is `BLOCKED` until implemented and tested.

## 5. Decision

### 5.1 Manifest v1

The normative field table, bounds, canonical byte order, and validation order are in `docs/product-specs/eqwec-hub-schemas.md`. The schema document is normative; this ADR records the decision and its rationale.

Summary:

- UTF-8 JSON, maximum 64 KiB.
- `schema_version` is `1`.
- Required fields: `product_id`, `product_version`, `channel`, `target`, `package`, `dependencies`, `metadata_sequence`, `issued_at`, `expires_at`, `signature`.
- `signature` is excluded from the signed payload. Every other field is covered.
- Canonical payload bytes are schema-ordered JSON with no insignificant whitespace, no floating-point numbers, lowercase hex for hashes and signatures, and UTC RFC 3339 timestamps with `Z` and seconds precision.
- `package.hashes` contains exactly one `sha256` entry in v1.
- Dependencies are sorted by `product_id`, unique, and never self-referential.
- Unknown fields are rejected at every level.
- Duplicate keys are rejected by strict typed deserialization; a regression test must prove it.
- The package source must be `https://`; the download layer additionally enforces the production allowlist.
- `release_notes` is optional, bounded, display-only, and never interpreted as markup or a command.

### 5.2 Trust and verification

Verification order:

1. Bound and parse the manifest; reject malformed, duplicate, unknown, or trailing data.
2. Validate schema version, identifiers, version, channel, target, package fields, dependencies, sequence, and timestamps.
3. Match the target OS and architecture exactly.
4. Reject expired or not-yet-valid metadata outside the allowed clock skew.
5. Look up `key_id` in the pinned trust store.
6. Reject a revoked key, an epoch mismatch, or a key not allowed for the manifest channel.
7. Verify the Ed25519 signature over the canonical payload bytes with `verify_strict`.
8. Only then expose a `VerifiedManifest`.
9. Verify the package size and SHA-256 before extraction or activation.

Any signature, integrity, trust, or validity failure is terminal. There is no ignore or insecure fallback path in a release build.

### 5.3 Registry v1

- A single versioned JSON document per user profile, replaced atomically.
- At most one active record per `product_id`.
- Each record stores `product_id`, `product_version`, `channel`, `install_root`, `active_tree`, retained verified trees, `signer_key_id`, `key_epoch`, `package_sha256`, `metadata_sequence`, and `installed_at`.
- `install_root` is absolute and normalized. Tree paths are relative to it and must resolve inside it.
- Unknown schema versions fail closed; migrations are explicit and tested.
- The registry is not written until the staged tree is verified and activated.

### 5.4 Journal v1

- Append-only JSON Lines, one complete record per phase, fsynced before the next mutation.
- Fields: `schema_version`, `operation_id`, `idempotency_key`, `product_id`, `phase`, `timestamp`, and bounded context.
- Phases: `intent`, `downloaded`, `verified`, `staged`, `activation-intent`, `activated`, `registry-committed`, `committed`, `failed`, `rolled-back`.
- Startup reads complete lines, discards a partial trailing line, and reconciles the journal, locks, staging directories, and registry.
- A crash before `committed` recovers to a complete old or complete new state; never a mixed tree.
- Journal size and operation count are bounded; old completed operations are pruned after the retention window.

### 5.5 Installation layout v1

Per-user base directories:

| Platform | Base |
|---|---|
| Windows | `%LOCALAPPDATA%\EQwec` |
| macOS | `~/Library/Application Support/EQwec` |
| Linux | `$XDG_DATA_HOME/eqwec` or `~/.local/share/eqwec` |

Layout:

```text
<base>/
  registry.json
  active.lock
  journal/
    operations.jsonl
  installations/<product-id>/
    active.json             # atomic pointer to a verified tree
    versions/<tree-id>/     # immutable verified trees, including retained versions
    staging/<operation-id>/
  downloads/<operation-id>/
  cache/
  logs/
  diagnostics/
```

- `active.json` is never modified in place. Activation atomically replaces the pointer within one filesystem; version trees are immutable.
- Rollback atomically replaces `active.json` with a pointer to a retained verified tree.
- Staging, downloads, version trees, and logs are separated and bounded.
- Cross-filesystem moves are explicit and verified; atomic rename is never assumed across filesystems.
- Shortcuts, services, and scheduled tasks are created only with consent and are removable.
- User projects and documents are never inside these roots and are preserved by uninstall unless purge is explicitly confirmed.

### 5.6 Schema versioning

- Manifest, registry, journal, trust store, and installation layout are versioned independently.
- A schema change that alters canonical bytes requires a new manifest schema version and a new signature domain.
- Registry and journal migrations are forward-only, tested on a copy, and fail closed on unknown versions.
- Downgrades require a signed policy allowance and explicit user confirmation.

## 6. Implementation gates

1. A canonicalization test vector with a known key and signature passes.
2. Duplicate keys, unknown fields, trailing data, oversized input, invalid timestamps, wrong channel, wrong key epoch, revoked key, and tampered payload are rejected.
3. Package size and SHA-256 are verified before extraction.
4. Registry atomic replace and journal recovery are tested with fault injection.
5. A real fixture product installs, updates, rolls back, repairs, and uninstalls on macOS, then Windows and Linux.
6. Non-production test keys are isolated from release artifacts.

## 7. Consequences

**Positive**

- One explicit schema and canonical form instead of per-implementation guesses.
- Signature ambiguity is removed before any Hub code exists.
- Registry and journal formats are inspectable and recoverable.
- The installation layout keeps active, staging, download, and rollback data separate.

**Negative**

- A custom canonical form must be documented and kept in sync with the code.
- JSON is larger than a binary format, though the manifest is small.
- Append-only journaling and atomic replace require careful filesystem handling.
- Registry size will eventually justify a different store if product count grows.

**Security**

- The manifest signature covers every security-relevant field.
- Trust is pinned by key ID, epoch, and channel; revocation fails closed.
- Duplicate keys and unknown fields are rejected, not silently accepted.
- The package is verified before extraction or execution.
- The registry is not authoritative until the journal and activation complete.

**Performance**

- Manifest parsing is bounded and small; no performance claim is made until measured.
- Registry and journal I/O are on the install/update path, not the launch-critical path.
- No numeric budget is asserted here.

**Licensing**

- Ed25519, SHA-256, JSON, and RFC 3339 implementations must use permissively licensed crates with recorded notices.
- No codec or DRM dependency is introduced by this ADR.

**Maintenance**

- Canonicalization, registry, and journal code require regression tests and versioned migrations.
- A schema change is a compatibility event, not a refactor.
- Platform path handling requires per-platform tests.

## 8. Reconsideration criteria

Revisit this ADR if:

- The manifest outgrows JSON or canonicalization becomes a maintenance burden.
- Registry concurrency or size exceeds what atomic JSON replace can support.
- A platform installer requires a different layout or elevated machine-wide install.
- A standard canonicalization or signing envelope is adopted ecosystem-wide.
- A key-rotation or revocation design requires a different trust-store format.
- Fault injection shows the journal cannot recover a required phase.

## 9. Links

- `docs/product-specs/eqwec-hub.md`
- `docs/product-specs/eqwec-hub-schemas.md`
- `docs/security/THREAT-MODEL.md`
- `docs/security/SECURITY-BASELINE.md`
- `docs/architecture/DEPENDENCY-MAP.md`
- ADR-0006
- ADR-0007
- ADR-0009
- ADR-0010
