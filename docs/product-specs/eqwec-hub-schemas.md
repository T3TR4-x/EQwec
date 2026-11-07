# EQwec Hub — Normative Schemas v1

- **Status:** NORMATIVE for Hub manifest v1, trust store v1, registry v1, journal v1, installation layout v1, and package archive v1.
- **Version:** `1` for each schema unless stated otherwise.
- **Owner:** Hub product workstream.
- **Decision:** `docs/adr/0011-hub-manifest-registry-and-layout.md` and `docs/adr/0012-hub-package-archive.md`.
- **Product requirements:** `docs/product-specs/eqwec-hub.md`.
- **Implementation status:** see §10. A schema being normative does not mean its runtime path is implemented or tested.

This document is the single source of truth for the bytes, fields, bounds, and validation order of the Hub security formats. Code that disagrees with this document is defective. A change to canonical bytes is a new schema version, not a refactor.

## 1. Conventions

### 1.1 Encoding

- All documents are UTF-8 without a byte-order mark.
- JSON follows RFC 8259.
- Timestamps follow RFC 3339 with exactly `YYYY-MM-DDTHH:MM:SSZ`: UTC, seconds precision, no fractional seconds, no numeric offset, no leap second.
- Hexadecimal values are lowercase ASCII `0-9a-f`. Uppercase hex is rejected, not normalized.
- Versions follow Semantic Versioning 2.0.0.
- Version constraints follow Cargo's semver constraint grammar.
- Identifiers are opaque. They are never derived from a display name or a filesystem path.

### 1.2 Canonical JSON

The manifest signature covers a canonical byte sequence. Canonical JSON has:

1. Exactly the fields defined by the schema, in the order listed in this document.
2. No whitespace between tokens.
3. No byte-order mark and no trailing newline.
4. Integers in decimal with no leading zero, sign, exponent, or fraction. All integers are non-negative and fit in `u64`.
5. No floating-point numbers and no `null` values.
6. Strings encoded as UTF-8 with this escaping only:
   - `"` becomes `\"`
   - `\` becomes `\\`
   - U+0008 becomes `\b`
   - U+0009 becomes `\t`
   - U+000A becomes `\n`
   - U+000C becomes `\f`
   - U+000D becomes `\r`
   - every other scalar value U+0000 through U+001F becomes `\u00xx` with lowercase hex
   - every other Unicode scalar value is emitted as raw UTF-8
   - `/` is never escaped
7. Arrays in schema order. Dependencies are sorted by `product_id` in ascending byte order before encoding.
8. Optional fields omitted entirely when absent. An optional field is never encoded as `null`.

A parser may accept non-canonical input formatting, but it must re-encode the semantic value canonically before signature verification. Two documents with the same fields and values therefore produce the same canonical bytes regardless of input whitespace, key order, or JSON escape spelling.

### 1.3 Signature domain

The signed payload is the canonical JSON of every manifest field except `signature`. `schema_version` is inside the payload, so a v1 signature cannot be replayed as another schema version. `product_id` and `target` are inside the payload, so a signature cannot be moved to another product or platform.

`signature.key_id`, `signature.key_epoch`, `signature.algorithm`, and `signature.signing_time` are outside the signed payload. They are policy inputs used to select and validate the key. Substituting them does not help an attacker who lacks the corresponding private key.

### 1.4 Rejection rules

Every parser rejects, with a typed error:

- input above the size bound;
- invalid UTF-8;
- malformed JSON;
- duplicate object keys at any level;
- unknown object fields at any level;
- trailing non-whitespace data;
- wrong JSON type for any field;
- a value outside its declared bound;
- an unsupported schema version;
- an inconsistent cross-field value.

Silent coercion, default substitution for a required field, and "best effort" parsing are forbidden.

## 2. Manifest v1

### 2.1 Size and shape

- Maximum encoded size: **65,536 bytes (64 KiB)**.
- Top level: one JSON object.
- Maximum nesting depth: 4 (manifest, target/package/signature, hash/dependency, and no deeper).
- Maximum dependency count: **32**.
- Maximum release-notes size: **4,096 bytes**.
- Maximum package size: **8 GiB** (`8,589,934,592` bytes); minimum `1`.

### 2.2 Top-level fields

The canonical order is the table order. `signature` is excluded from the payload.

| # | Field | Type | Required | Bound / rule |
|---|---|---|---|---|
| 1 | `schema_version` | integer | yes | exactly `1` |
| 2 | `product_id` | string | yes | `ProductId`: 1–64 bytes, lowercase ASCII alphanumeric and hyphen, starts and ends alphanumeric |
| 3 | `product_version` | string | yes | SemVer 2.0.0; raw value at most 128 bytes |
| 4 | `channel` | string | yes | `stable`, `beta`, `alpha`, or `dev` |
| 5 | `target` | object | yes | §2.3 |
| 6 | `package` | object | yes | §2.4 |
| 7 | `dependencies` | array | yes | §2.5; may be empty |
| 8 | `metadata_sequence` | integer | yes | `1..=u64::MAX`; strictly increases per product and channel |
| 9 | `issued_at` | string | yes | RFC 3339 second-precision UTC |
| 10 | `expires_at` | string | yes | RFC 3339; later than `issued_at` |
| 11 | `release_notes` | string | no | 1–4,096 bytes; display-only plain text |
| 12 | `signature` | object | yes | §2.6; excluded from the payload |

### 2.3 `target`

Canonical order: `os`, `architecture`, `minimum_platform`.

| Field | Type | Required | Bound / rule |
|---|---|---|---|
| `os` | string | yes | `windows`, `macos`, or `linux` |
| `architecture` | string | yes | `x86_64` or `aarch64` |
| `minimum_platform` | string | yes | 1–64 bytes; ASCII `A-Z a-z 0-9 . _ -`; starts with an alphanumeric |

`minimum_platform` is an opaque platform policy string, for example `11.0`, `10.0.19045`, or `glibc-2.39`. The updater matches `os` and `architecture` exactly. The platform-specific minimum comparison is a separate installer gate and must not be claimed until implemented per platform.

### 2.4 `package`

Canonical order: `source`, `size`, `hashes`.

| Field | Type | Required | Bound / rule |
|---|---|---|---|
| `source` | string | yes | 12–2,048 bytes; strict HTTPS URL per §2.7 |
| `size` | integer | yes | `1..=8,589,934,592` bytes |
| `hashes` | array | yes | exactly one entry in v1 |

Hash entry canonical order: `algorithm`, `value`.

| Field | Type | Required | Bound / rule |
|---|---|---|---|
| `algorithm` | string | yes | exactly `sha256` |
| `value` | string | yes | exactly 64 lowercase hex characters |

The declared size and SHA-256 are verified against the package bytes before extraction, staging, or activation. A package is never executed or unpacked before both checks pass.

### 2.5 `dependencies`

Each dependency is an object with canonical order `product_id`, `version_constraint`, `package_hash`.

| Field | Type | Required | Bound / rule |
|---|---|---|---|
| `product_id` | string | yes | valid `ProductId`; must differ from the manifest's `product_id` |
| `version_constraint` | string | yes | SemVer constraint; raw value 1–128 bytes |
| `package_hash` | string | yes | exactly 64 lowercase hex characters |

Rules:

- At most 32 dependencies.
- Sorted by `product_id` in ascending byte order.
- No duplicate `product_id`.
- No self-reference.
- A dependency hash identifies the exact package the manifest was built against. Resolution and compatibility checks are Hub policy, not part of signature verification.

### 2.6 `signature`

Canonical order: `key_id`, `algorithm`, `signature`, `signing_time`, `key_epoch`.

| Field | Type | Required | Bound / rule |
|---|---|---|---|
| `key_id` | string | yes | 1–64 bytes; lowercase ASCII alphanumeric and hyphen; starts and ends alphanumeric |
| `algorithm` | string | yes | exactly `ed25519` |
| `signature` | string | yes | exactly 128 lowercase hex characters (64 bytes) |
| `signing_time` | string | yes | RFC 3339 second-precision UTC |
| `key_epoch` | integer | yes | `1..=u64::MAX` |

The signature is Ed25519 over the canonical payload from §2.2. Verification uses strict Ed25519 verification, which rejects non-canonical and small-order encodings.

### 2.7 Package source URL

A v1 source is accepted only when all of the following hold:

- It starts with exactly `https://` in lowercase.
- Total length is 12–2,048 bytes.
- It contains only printable ASCII; no spaces, control characters, non-ASCII bytes, or backslash.
- The authority is non-empty and contains no userinfo (`@`).
- The host is a DNS name: dot-separated labels of 1–63 characters, ASCII letters, digits, and hyphen; labels do not start or end with a hyphen; at least two labels.
- An optional port is decimal, has no leading zero, and is `1..=65535`.
- The path is empty or starts with `/`; `.` and `..` path segments are rejected.
- A query is allowed; a fragment (`#`) is rejected.
- Every `%` starts a valid two-digit hexadecimal escape.
- Characters are limited to RFC 3986 unreserved, sub-delimiters, `:`, `@`, `/`, `?`, and `%`.

The production download layer additionally enforces a host allowlist. URL validation alone is not authorization to download.

### 2.8 Time rules

- `issued_at` is earlier than `expires_at`.
- The validity window is at most **366 days**.
- `signing_time` lies within `[issued_at - 300 s, expires_at + 300 s]`.
- At verification, `now` must satisfy `now + 300 s >= issued_at` and `now <= expires_at + 300 s`.
- The 300-second allowance is clock skew, not permission to use an expired manifest indefinitely.
- A release build has no ignore-expiry option.

### 2.9 Release notes

`release_notes`, when present, is non-empty plain text of at most 4,096 bytes. It must not contain:

- NUL or other C0 control characters except `\t`, `\n`, and `\r`;
- C1 control characters U+0080–U+009F;
- bidi control characters U+200E, U+200F, U+202A–U+202E, U+2066–U+2069.

It is displayed as text, never parsed as HTML, Markdown, a command, or a script.

### 2.10 Canonical payload example

The following is the exact canonical payload shape. Whitespace is shown only for readability; the real bytes contain none. `signature` is absent.

```json
{
  "schema_version": 1,
  "product_id": "eqwec-hub",
  "product_version": "0.1.0",
  "channel": "stable",
  "target": {
    "os": "macos",
    "architecture": "aarch64",
    "minimum_platform": "11.0"
  },
  "package": {
    "source": "https://packages.eqwec.example/hub/0.1.0/eqwec-hub-0.1.0-macos-aarch64.tar.zst",
    "size": 1048576,
    "hashes": [
      {
        "algorithm": "sha256",
        "value": "0000000000000000000000000000000000000000000000000000000000000000"
      }
    ]
  },
  "dependencies": [],
  "metadata_sequence": 1,
  "issued_at": "2026-11-07T00:00:00Z",
  "expires_at": "2027-02-05T00:00:00Z",
  "release_notes": "Initial Hub MVP fixture."
}
```

The compact canonical bytes begin:

```text
{"schema_version":1,"product_id":"eqwec-hub",...
```

The hash value in this example is a placeholder, not a real package digest.

## 3. Trust store v1

### 3.1 Purpose

The trust store pins the public keys allowed to sign Hub metadata. It is not a certificate chain and it does not fetch keys from the network.

- The initial trust store is compiled into the Hub binary.
- An on-disk trust-store update must be signed by a root key and is `BLOCKED` until root-key custody and rotation exist.
- A release build must not accept an unsigned trust store from a user-writable path.

### 3.2 Format

Maximum encoded size: **65,536 bytes**. Maximum keys: **64**.

```json
{
  "schema_version": 1,
  "keys": [
    {
      "key_id": "release-2026-01",
      "algorithm": "ed25519",
      "public_key": "0000000000000000000000000000000000000000000000000000000000000000",
      "key_epoch": 1,
      "channels": ["stable", "beta"],
      "not_before": "2026-01-01T00:00:00Z",
      "not_after": "2028-01-01T00:00:00Z",
      "revoked": false
    }
  ]
}
```

Rules:

- Unknown and duplicate fields are rejected.
- `key_id` is unique and sorted ascending.
- `algorithm` is exactly `ed25519`.
- `public_key` is exactly 64 lowercase hex characters and must decode to a valid Ed25519 point.
- `key_epoch` is at least 1.
- `channels` is non-empty, unique, and in the order `stable`, `beta`, `alpha`, `dev`.
- `not_before` is earlier than `not_after`; the window is at most 10 years.
- `revoked` is a boolean.
- The store is rejected entirely if any entry is invalid. Invalid entries are not skipped.

### 3.3 Key selection

A manifest is verified only if:

1. `signature.key_id` exists in the store.
2. The entry is not revoked.
3. `signature.key_epoch` equals the entry's `key_epoch`.
4. The entry's `channels` contains the manifest channel.
5. `signing_time` lies within `[not_before - 300 s, not_after + 300 s]`.
6. The strict Ed25519 signature check succeeds.

Failure at any step is terminal. There is no fallback key, no "try all keys" loop, and no network key discovery.

## 4. Registry v1

### 4.1 Purpose and storage

The registry is the authoritative record of active installations for one user profile.

- Path: `<base>/registry.json`.
- Maximum size: **262,144 bytes (256 KiB)**.
- Maximum installation records: **64**.
- Written only by atomic replace after the staged tree is verified and the activation pointer is committed.
- At most one record per `product_id`.
- Records sorted by `product_id` ascending.
- Unknown schema versions fail closed.

### 4.2 Format

```json
{
  "schema_version": 1,
  "profile_id": "default",
  "updated_at": "2026-11-07T00:00:00Z",
  "installations": [
    {
      "product_id": "eqwec-hub",
      "product_version": "0.1.0",
      "channel": "stable",
      "install_root": "/home/user/.local/share/eqwec/installations/eqwec-hub",
      "active_tree": "versions/0.1.0-0123456789ab",
      "retained": [
        {
          "product_version": "0.0.9",
          "tree": "versions/0.0.9-fedcba987654",
          "package_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
          "signer_key_id": "release-2025-01",
          "key_epoch": 1,
          "metadata_sequence": 17
        }
      ],
      "signer_key_id": "release-2026-01",
      "key_epoch": 1,
      "package_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
      "metadata_sequence": 42,
      "installed_at": "2026-11-07T00:00:00Z"
    }
  ]
}
```

Field rules:

- `profile_id` is a valid `ScopeId`.
- `install_root` is absolute, normalized, at most 1,024 bytes, contains no NUL or control character, and is inside the profile base.
- `active_tree`, retained `tree`, and every path derived from the registry are relative `SafeRelativePath` values that resolve inside `install_root`.
- `retained` has at most 3 entries, unique `product_version`, and each `tree` differs from `active_tree`.
- `signer_key_id`, `key_epoch`, `package_sha256`, and `metadata_sequence` come from the verified manifest that produced the active tree. Retained entries carry the same identity for their own tree so a rollback can rebuild the record without re-downloading the manifest.
- `installed_at` is the local activation time recorded when the active pointer was committed.
- A registry record is removed only after uninstall completes and the journal records it.

### 4.3 Atomic write

1. Serialize the complete document.
2. Write to `registry.json.tmp-<operation-id>` in the same directory.
3. Flush the file.
4. Rename over `registry.json`.
5. Flush the directory where the platform supports it.

A failed write leaves the previous registry intact. A partial temporary file is removed on recovery.

## 5. Journal v1

### 5.1 Storage

- Path: `<base>/journal/operations.jsonl`.
- Append-only JSON Lines: one complete record per line, `\n` terminated.
- Maximum record size: **4,096 bytes**.
- Maximum file size: **16 MiB**.
- Maximum retained records: **4,096**; completed terminal records are pruned after 30 days or when the record cap is reached.
- Each record is flushed before the next filesystem mutation.
- A partial trailing line is discarded on recovery.
- A malformed complete line is a fail-closed condition: the journal is quarantined and no mutation proceeds until an operator or repair path resolves it.

### 5.2 Record

```json
{
  "schema_version": 1,
  "operation_id": "0123456789abcdef0123456789abcdef",
  "idempotency_key": "install:eqwec-hub:stable:0.1.0",
  "product_id": "eqwec-hub",
  "phase": "intent",
  "timestamp": "2026-11-07T00:00:00Z",
  "context": {
    "from_version": "0.0.9",
    "to_version": "0.1.0",
    "channel": "stable",
    "package_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
    "signer_key_id": "release-2026-01",
    "key_epoch": 1,
    "metadata_sequence": 42
  }
}
```

Field rules:

- `operation_id` is exactly 32 lowercase hex characters.
- `idempotency_key` is 1–128 characters from `A-Z a-z 0-9 . _ : -`, starts and ends alphanumeric.
- `product_id` is a valid `ProductId`.
- `phase` is one of the values in §5.3.
- `context` is optional. When present, it has only the fields shown; unknown fields are rejected. `channel` and `package_sha256` are required; `from_version` and `to_version` may be omitted; `error_code` is present only for `failed` and is rejected for every other phase.
- `signer_key_id`, `key_epoch`, and `metadata_sequence` come from the verified manifest. They appear together or not at all. Recovery requires them to rebuild a registry record and fails closed when they are missing or partial.
- `from_version` and `to_version` are SemVer strings.
- `package_sha256` is 64 lowercase hex characters.
- A terminal record repeats the same `operation_id` and `idempotency_key` as the opening `intent`.

### 5.3 Phases

| Phase | Meaning | Next mutation |
|---|---|---|
| `intent` | Operation accepted; intent durable | Acquire lock, prepare download |
| `downloaded` | Package bytes complete in `downloads/` | Verify size and hash |
| `verified` | Size and hash match the manifest | Extract into staging |
| `staged` | Tree complete in staging and verified | Prepare activation |
| `activation-intent` | Pointer swap is about to happen | Replace `active.json` |
| `activated` | Pointer points at the new tree | Commit registry |
| `registry-committed` | Registry contains the new record | Mark committed |
| `committed` | Operation complete | Prune staging and old downloads |
| `failed` | Operation failed before activation | Reconcile and prune |
| `rolled-back` | Previous tree is active again after an activation attempt or an explicit rollback | Reconcile and prune |
| `uninstall-intent` | Record and containment validated; product removal is about to start | Remove the product root |
| `uninstalled` | Product tree and registry record removed | Terminal; prune |

### 5.4 Recovery

On startup, after acquiring the profile lock:

1. Read complete journal lines; discard a partial trailing line.
2. Find the newest operation without a terminal phase.
3. Reconcile against the filesystem and registry:
   - `intent`, `downloaded`, `verified`, `staged`: remove the operation's download and staging directories; no registry change.
   - `activation-intent`: read `active.json`. If it points at the new verified tree, write the registry record and continue. Otherwise restore the previous pointer and mark `rolled-back`. A tree that already belongs to the previous registry record or its retained list is never deleted.
   - `activated`: verify the active tree, then write the registry record and mark `registry-committed`, then `committed` for an install or update, or `rolled-back` when `from_version` is greater than `to_version`.
   - `registry-committed`: mark `committed` for an install or update, or `rolled-back` for a rollback.
   - `uninstall-intent`: remove the product root, remove the registry record when present, write the registry atomically, and mark `uninstalled` (ADR-0013).
   - `committed`, `failed`, `rolled-back`, `uninstalled`: terminal; prune.
4. Append a recovery record with the same `operation_id` before deleting any remaining operation data.
5. Never leave a mixed tree active. A tree is either fully verified or not referenced.

Recovery resolves one operation per call. The caller repeats it until it returns `NothingToRecover`. A new operation is rejected while any other operation has no terminal record. Recovery is idempotent: running it twice produces the same registry and no duplicate records.

## 6. Installation layout v1

### 6.1 Base directory

| Platform | Base |
|---|---|
| Windows | `%LOCALAPPDATA%\EQwec` |
| macOS | `~/Library/Application Support/EQwec` |
| Linux | `$XDG_DATA_HOME/eqwec`, falling back to `~/.local/share/eqwec` |

### 6.2 Layout

```text
<base>/
  registry.json
  active.lock
  journal/
    operations.jsonl
  installations/
    <product-id>/
      active.json
      versions/
        <tree-id>/
      staging/
        <operation-id>/
  downloads/
    <operation-id>/
  cache/
  logs/
  diagnostics/
```

- `active.json` is the only activation pointer. It is a small bounded JSON document, atomically replaced.
- `versions/<tree-id>/` is immutable after activation.
- `tree-id` is `<product-version>-<first 12 hex characters of package_sha256>`.
- `staging/<operation-id>/` is never referenced by `active.json`.
- `downloads/<operation-id>/` holds the partial and complete package.
- `cache/`, `logs/`, and `diagnostics/` are bounded and non-authoritative.
- No active tree is modified in place.
- Atomic rename is used only within one filesystem. A cross-filesystem move is explicit, verified, and never described as atomic.
- User projects and documents are never stored under `<base>`.

### 6.3 `active.json`

Maximum size: **4,096 bytes**. Canonical order: `schema_version`, `tree`, `package_sha256`, `activated_at`.

```json
{
  "schema_version": 1,
  "tree": "versions/0.1.0-0123456789ab",
  "package_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
  "activated_at": "2026-11-07T00:00:00Z"
}
```

Rules:

- `tree` is a relative `SafeRelativePath` inside the product installation root.
- The referenced tree must exist and match `package_sha256` before activation is considered complete.
- A missing or invalid `active.json` with a committed registry record is a repair condition, not a silent reinstall.
- Rollback replaces `active.json` atomically with a pointer to a retained verified tree.

## 7. Validation order

The updater applies checks in this order and stops at the first failure:

1. Encoded size bound.
2. UTF-8 decoding and strict JSON parse.
3. Duplicate-key, unknown-field, type, and trailing-data rejection.
4. `schema_version`.
5. Identifier, version, channel, target, package, dependency, sequence, timestamp, release-notes, and signature-field validation.
6. Cross-field checks: validity window, signing time, dependency sort/uniqueness/self-reference, hash count.
7. Requested channel and exact target match.
8. Current-time validity with 300-second skew.
9. Trust-store key lookup, revocation, epoch, channel, and key validity.
10. Strict Ed25519 verification over canonical bytes.
11. `metadata_sequence` regression check against the caller's minimum.
12. Package size and SHA-256 verification before extraction.
13. Package archive validation and bounded extraction into staging per §11.
14. Return a `VerifiedManifest`, `VerifiedPackage`, and `StagedTree`.

A failure at steps 1–11 produces no verified manifest. A failure at step 12 produces no verified package. A failure at step 13 produces no staged tree and no promotion. No partial result is promoted.

## 8. Bounds summary

| Value | Bound |
|---|---|
| Manifest document | 64 KiB |
| Trust store document | 64 KiB |
| Registry document | 256 KiB |
| Journal record | 4 KiB |
| Journal file | 16 MiB |
| `active.json` | 4 KiB |
| Release notes | 4,096 bytes |
| Package source URL | 2,048 bytes |
| `minimum_platform` | 64 bytes |
| `product_version` raw value | 128 bytes |
| `version_constraint` raw value | 128 bytes |
| Dependencies | 32 |
| Trust keys | 64 |
| Registry installations | 64 |
| Retained versions per product | 3 |
| Journal records | 4,096 |
| Package size | 8 GiB |
| Archive entries | 100,000 |
| Archive total uncompressed bytes | 16 GiB |
| Archive uncompressed bytes per entry | 4 GiB |
| Archive compression ratio per entry | 200:1 |
| Archive total compression ratio | 100:1 |
| Archive path length | 1,024 bytes |
| Archive path component | 255 bytes |
| Archive path depth | 32 components |
| Archive extraction time | 600 seconds |
| Manifest validity | 366 days |
| Clock skew | 300 seconds |
| Key validity | 10 years |

## 9. Conformance requirements

A conforming implementation must pass at least:

- a fixed canonicalization vector with a known key and signature;
- duplicate top-level and nested keys;
- unknown top-level and nested fields;
- trailing data;
- oversized manifest, trust store, registry, and journal inputs;
- invalid UTF-8;
- wrong schema version;
- invalid identifier, version, channel, target, URL, hash, and timestamp;
- unsorted, duplicate, and self-referential dependencies;
- expired, not-yet-valid, and over-long validity windows;
- wrong channel, target, key ID, key epoch, and revoked key;
- tampered payload and tampered signature;
- metadata sequence regression;
- package size mismatch, truncation, and hash mismatch;
- archive traversal, absolute paths, drive and UNC prefixes, backslashes, NUL and control characters, empty components, trailing dots, and Windows reserved device names;
- archive symlinks, special-file modes, setuid, setgid, and sticky bits;
- archive duplicate names, ASCII case collisions, file/directory conflicts, and file-as-ancestor conflicts;
- archive entry-count, per-entry size, total-size, ratio, path-length, component-length, depth, and timeout bounds;
- archive header confusion: duplicate central-directory names, local/central disagreement, overlapping entries, unlisted local headers, prepended data, and a non-empty comment;
- archive encryption, unsupported compression methods, multi-disk archives, and empty archives;
- registry atomic replace and journal recovery with fault injection at every phase.

Tests use non-production keys. Test private keys never enter a release artifact.

## 10. Implementation status

Last verified: 2026-11-07 on macOS 26.7.2 arm64 with Rust 1.99.1.

| Area | State | Evidence |
|---|---|---|
| Manifest parse and canonicalization | `PASS` for v1 parsing | `crates/eqwec-updater/src/manifest.rs`; 19 unit tests; fixed canonical payload asserted byte-for-byte in `tests/verification.rs` |
| Trust store parse and key selection | `PASS` for in-memory policy | `crates/eqwec-updater/src/trust.rs`; revocation, epoch, channel, and key-window tests pass |
| Ed25519 and SHA-256 primitives | `PASS` | `crates/eqwec-security`; RFC 8032 vector, strict verification, weak-key rejection, SHA-256 known-answer tests |
| Package size and hash verification | `PASS` for reader and in-memory paths | `crates/eqwec-updater/src/package.rs`; truncation, size-mismatch, and hash-mismatch tests pass |
| Fixed canonicalization and signature vector | `PASS` | OpenSSL-generated Ed25519 vector from a non-production seed; exact payload, public key, and signature asserted in `tests/verification.rs` |
| Registry v1 | `PASS` for parse, serialize, atomic write, and layout containment | `crates/eqwec-updater/src/registry.rs`; round-trip, sort, duplicate, oversize, traversal, and atomic-write tests pass |
| Journal v1 | `PASS` for record, append, read, and recovery mapping | `crates/eqwec-updater/src/journal.rs`; phase, partial-line, corrupt-line, size-bound, and recovery-action tests pass |
| Installation layout | `PASS` for path construction and lexical validation | `crates/eqwec-updater/src/layout.rs`; normalized-absolute, containment, tree-id, and root-rejection tests pass |
| Active pointer | `PASS` for parse and atomic replacement | `crates/eqwec-updater/src/active.rs`; round-trip, unknown-field, oversize, and atomic-write tests pass |
| Activation transaction and recovery orchestration | `PASS` for install, update, rollback, and recovery with fault injection | `crates/eqwec-updater/src/activation.rs`; 25 tests cover phase ordering, missing/symlinked staging, existing trees, downgrades, retention, rollback, all five fault points, recovery before and after the pointer, rollback recovery, single-in-flight enforcement, pruning, and the retention cap |
| Package archive validation and staging | `PASS` for in-process archive gates 1–11 | `crates/eqwec-updater/src/staging.rs`; 31 tests cover a valid fixture, ZIP64, traversal and unsafe names, links and special modes, duplicate and case-colliding names, every bound, header confusion, encryption, unsupported compression, multi-disk and empty archives, pre-existing staging, failed-extraction cleanup, and activation from a staged tree |
| Real fixture product (v1/v2) | `PASS` for in-process macOS install, launch, update, rollback, tampered-package rejection, and interrupted-activation recovery | `tests/fixtures/eqwec-hub/`; two real executables, signed v1/v2 manifests with a public non-production seed, and 3 end-to-end tests in `tests/hub_end_to_end.rs` |
| Installer orchestration (download, uninstall, launcher) | `BLOCKED` | No transport, uninstaller, or UI |
| Platform fixtures | `BLOCKED` | Only the macOS in-process fixture runs; Windows and Linux fixture installs do not exist |

Verification command: `cargo test --workspace --all-features --locked` — 163 tests passed, 0 failed. `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`, `cargo audit`, and `cargo deny check` all exited 0.

This table is updated with real command output when an area is implemented. Parsing, cryptographic verification, durable-state primitives, an in-process activation engine, package staging, and a non-production fixture workflow are not evidence of a working installer. Download, uninstall, the launcher UI, installer packaging, and Windows/Linux acceptance remain `BLOCKED`.

## 11. Package archive v1

- **Status:** NORMATIVE for Hub product packages.
- **Decision:** `docs/adr/0012-hub-package-archive.md`.
- **Container:** one ZIP or ZIP64 archive.
- **Coverage:** the manifest `package.size` and `package.hashes[0].value` cover the complete archive bytes.

### 11.1 Container rules

- The archive starts at byte zero. Prepended data is rejected.
- Multi-disk and spanned archives are rejected.
- The archive comment is empty.
- The archive contains at least one regular file.
- Encryption, passwords, and AES entries are rejected.
- The only accepted compression methods are `stored` (0) and `deflate` (8). Every other method is rejected before decompression.
- The central directory is authoritative. The local file header must agree with it.
- The archive-level header-consistency check must pass before any entry is written. Duplicate central-directory names, overlapping entries, local/central disagreement, and local headers missing from the central directory are rejected.
- Entry names are read from the raw central-directory bytes. The Unicode Path extra field and every other name override are ignored.
- Data descriptors are accepted only when the central directory declares the sizes. The extractor bounds actual output independently of declared sizes.

### 11.2 Entry types and modes

- Only regular files and directories are allowed.
- Symbolic links, hardlinks, device nodes, FIFOs, sockets, and every other special type are rejected. A Unix mode with type bits other than regular file or directory is rejected.
- File modes are `0644` or `0755`. Directory modes are `0755`.
- Setuid, setgid, and sticky bits are rejected.
- A missing Unix mode means `0644` for a file and `0755` for a directory.
- An executable file must declare `0755`. The extractor does not infer executability from the name or extension.
- On platforms without Unix modes, the mode is validated and then ignored for creation.

### 11.3 Entry paths

Every entry name is valid UTF-8 and uses only `A-Z`, `a-z`, `0-9`, `.`, `_`, `-`, and `/`.

- Non-empty and at most 1,024 bytes.
- Relative, with no leading `/`.
- No trailing `/` unless the entry is a directory.
- No empty, `.`, or `..` component.
- No `\`, `:`, NUL, or control character.
- Each component is at most 255 bytes.
- At most 32 components.
- No component ends with `.`.
- No component is a Windows reserved device name, case-insensitively, with or without an extension: `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`, `CONIN$`, `CONOUT$`.
- Paths are unique after ASCII case folding.
- A path cannot be both a file and a directory, and a file cannot be an ancestor of another entry.

The ASCII-only rule is intentional. It removes Unicode normalization collisions on macOS and case-folding ambiguity on Windows. Extending it requires a new archive schema version.

### 11.4 Bounds

The bounds in §8 apply. The ratio is `declared uncompressed / max(compressed, 1)`. A stored entry has ratio 1; a zero-byte entry has ratio 0. The total ratio uses the sum of entry compressed sizes, not the package size.

The caller may lower a bound. Raising a default bound requires an explicit policy decision and must still fit the declared package size and available disk. The extraction time bound is measured with a monotonic clock and checked before parsing, before each entry, and during each bounded read.

### 11.5 Staging sequence

1. Open the package file once.
2. Verify size and SHA-256 on that open handle against the verified manifest.
3. Seek to the start and parse the archive from the same handle. Do not reopen by path.
4. Parse the central directory and apply §11.1.
5. Run the archive-level header-consistency check.
6. Validate every entry's name, type, mode, compression, encryption, declared size, conflicts, totals, and ratio. Write nothing in this pass.
7. Create a fresh staging directory. A pre-existing file, directory, or symlink at the operation path is rejected. Every ancestor from the profile base to the staging root must be a real directory, not a symlink.
8. Extract with `create_new` for every file. Never follow a pre-existing final path. Count actual bytes and hash each file. Reject short and long entries.
9. Sync file data and, on Unix, directory entries before recording `staged`.
10. Walk the staged tree with `symlink_metadata`. Reject symlinks and non-regular files. Compare the actual file set, sizes, and modes with the plan. Reject extra and missing entries.
11. Append the `staged` journal record. Only then may activation move the tree.

Any failure removes the partial staging tree. The journal remains at `verified`; startup recovery discards the staging directory and marks the operation failed. No partial tree is promoted.

### 11.6 Journal integration

The staging function requires the last journal record for the operation to be `downloaded` or `verified`, and its context must match the operation identity. It records `verified` after the package hash check when the last phase is `downloaded`, and records `staged` only after the tree is durable and verified.

## 12. Change control

- Manifest v1 canonical bytes are frozen once a production signature exists.
- Adding, removing, reordering, or retyping a payload field requires manifest v2 and a new signature domain.
- Registry, journal, trust store, and layout versions migrate forward only.
- A migration is tested on a copy, fails closed on unknown versions, and records its own journal phase.
- Downgrades require a signed policy allowance and explicit user confirmation.
- This document, ADR-0011, ADR-0012, and the implementation change in the same commit.
