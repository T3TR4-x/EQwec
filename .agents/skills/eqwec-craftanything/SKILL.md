---
name: eqwec-craftanything
description: Use when planning, implementing, reviewing, or documenting CraftAnything, the EQwec cross-application creative workspace and asset exchange, including workspace manifests, asset references, canonical document ownership, provenance, interchange packages, conflict handling, and IPC with specialized products. Trigger on changes to apps/craftanything, CraftAnything-specific crates, workspace or interchange formats, or cross-product asset open/return workflows. Do not use for editor internals owned by PhotoCraft, VectorCraft, FilmCraft, EffectCraft, or AudioCraft, for unrelated shared-crate work, or to justify turning CraftAnything into a universal editor.
---

# EQwec CraftAnything

CraftAnything is a flexible workspace that combines ArtCraft workflows (creative editing performed in the specialized products) with asset exchange and canonical document ownership. It organizes, links, previews, launches, and exchanges; it does not replace the editors that own the documents.

**Status: planning only.** The repository is at `0.0.0`; `apps/` and `crates/` do not exist in this worktree. Every path, crate, schema, message family, and workflow below is planned. Verify against the repository before citing it, and never describe a planned capability as shipped.

## Scope gate

Before implementation, record an ADR that defines the supported workspace workflows, asset kinds, integrated products, import/export fidelity, and explicit non-goals. Default non-goals: raster painting, vector path editing, NLE timeline editing, compositing/rendering, audio DSP/mixing, and format-specific project editing. Route such requests to the owning product or define a narrow shared contract. A new CraftAnything capability needs an owner, an acceptance workflow, and a non-duplication check.

## 1. Owned paths, crates, and activation

| Path | Role | State | Ownership |
|---|---|---|---|
| `apps/craftanything` | Workspace UI, orchestration, product discovery and launch, exchange flows | Planned | CraftAnything |
| `crates/eqwec-core` | Typed identifiers, errors, result conventions | Planned | Shared; core owns the API |
| `crates/eqwec-storage` | Tenant/profile-scoped project storage, atomic writes, journals | Planned | Shared |
| `crates/eqwec-ipc` | Versioned IPC envelopes and trust boundaries | Planned | Shared |
| `crates/eqwec-ui` | Design tokens and UI primitives, no product workflow logic | Planned | Shared |
| `crates/craftanything-assets` | Asset-reference schema, resolution, relink, index | Planned | CraftAnything; name provisional |
| `crates/craftanything-provenance` | Append-only provenance records and queries | Planned | CraftAnything; name provisional |
| `crates/craftanything-interchange` | Interchange manifest, validation, packaging, fidelity | Planned | CraftAnything; name provisional |

- Shared crates are owned by the integration/core boundary. A change requires an impact assessment naming every consumer and a cross-product contract test.
- Shared crates must not depend on CraftAnything or any product crate. CraftAnything-specific crates may depend on shared crates. Specialized products may consume published asset-reference and interchange APIs when they participate, but must not depend on `apps/craftanything`.
- Create no crate without a real owner, public API, tests, and a second consumer or a documented architectural reason.
- Scope every path, query, cache key, and IPC message by tenant or profile.

**Activation triggers:** changes to `apps/craftanything` or CraftAnything-specific crates; workspace, asset-reference, provenance, or interchange schemas; canonical-ownership rules; cross-product open/return or export/import flows; conflict handling; CraftAnything documentation, versioning, packaging, or release metadata.

**Non-triggers:** editor internals that do not affect exchange or ownership; unrelated shared-crate work; Hub installer work; generic UI token changes with no workspace behavior. Do not use this skill to design a new editor.

## 2. Domain model and invariants

| Concept | Owner | Rule |
|---|---|---|
| Workspace | CraftAnything | Manifest, membership, reference graph, provenance, index. Holds no canonical payloads. |
| Asset reference | CraftAnything | Stable identity plus locator, owner product, format, revision, and content hash. Not a copy. |
| Canonical document | Owning product | Owns bytes, format, editing, undo, save, migration, and rendering. |
| Import/export contract | Interchange crate plus format owner | Defines package rules; the owner defines format mapping and fidelity. |
| Provenance | CraftAnything records; products report | Append-only event history with actor, time, product version, asset, revision, hash, and action. |
| Project link | CraftAnything link record; project owner | A linked project remains owned by its product and may contain nested assets. |
| Derived data | CraftAnything | Thumbnails, indexes, and caches. Disposable, rebuildable, never canonical. |

**Ownership split.** CraftAnything owns workspace semantics and the reference graph. It does not own document bytes. Each document has exactly one owner product. Another product may read through an explicit export or documented interchange contract; it may not mutate the canonical document. A second product claiming the same document is a conflict, not a second owner.

**Invariants:**

1. One canonical owner per document; no duplicate mutable copies. Import either links an existing document or asks the owner to create a new document with a new identity, never both.
2. `asset_id` and `document_id` are stable; paths are locators, not identity. Rename and move trigger relink, not a new asset.
3. Revision and hash are checked before open, return, save, and export. A mismatch is a conflict or an unresolved reference, never a silent refresh.
4. Provenance is append-only. Do not rewrite history; redact secrets and unnecessary content.
5. Every contract is versioned. An incompatible major version fails closed; do not downgrade or partially import.
6. A conflict is never auto-merged or overwritten. Preserve both sides and require an explicit choice.
7. A missing asset does not delete its reference or block workspace startup.
8. Linking a project does not flatten it. Nested ownership stays with the project owner. Prevent cycles and duplicate links.
9. Removing a link does not delete the underlying document. Deleting a linked project leaves an unresolved link, not a dangling write.
10. Derived data is never a source of truth and is safe to discard.
11. Export fidelity is declared per asset. A lossy or reference-only export is labeled as such.
12. Every operation is scoped to a tenant or profile; no global default for user-owned data.

**Versioning:** the workspace schema, asset-reference schema, provenance schema, interchange manifest, canonical document format, and IPC protocol version independently. A workspace records the versions it requires. An incompatible major version fails closed; a compatible minor version follows the declared compatibility window. Preserve unknown fields only when the contract declares forward compatibility.

**Import/export contract:** an import validates the package before creating or linking anything. A failed import preserves the source and writes nothing outside the workspace. An export creates a package with verified hashes; embedded payloads are read-only snapshots and never become canonical on import without an explicit owner import step.

**Conflict resolution:** detect a stale baseline, concurrent edit, ownership dispute, or format-version mismatch; stop writes; preserve both revisions; classify; present explicit options (keep owner revision, save as a new asset or copy, cancel); apply only the chosen option; record the resolution in provenance; verify the resulting revision. No last-writer-wins and no cross-product auto-merge.

**Missing and moved assets:** unresolved references remain visible and repairable. Relink by stable ID and hash where possible; never substitute a same-named file with a different hash. Distinguish a moved document from a changed document.

## 3. Threat boundaries and security

- **Imported assets and projects are untrusted.** Validate the manifest, schema, sizes, counts, hashes, and ownership fields before use. Reject unknown or inconsistent required fields. Bound parser depth and allocation.
- **Path safety.** Reject absolute paths, `..`, NUL, device paths, Windows reserved names, alternate data streams, and trailing dots or spaces. Canonicalize the workspace root; resolve symlinks and junctions; reject writes that escape the root. Prefer no-follow opens where supported and verify file identity after open to reduce TOCTOU risk.
- **Malicious thumbnails and metadata.** Decode with byte, dimension, and time limits. Do not execute embedded content, resolve external entities, or fetch remote metadata. Treat text as untrusted, strip active content, and never trust MIME type or extension.
- **Cross-product IPC is a trust boundary even locally.** Use a per-user channel, verify peer identity where the platform supports it, validate every field, bound payloads, enforce deadlines, reject replayed request IDs, and reject unknown message types. Never trust a caller-provided path, owner, or capability.
- **Permission boundaries.** Request only workspace and explicitly linked roots. Preserve OS sandboxing and security-scoped access. Do not escalate through a helper or widen access to make a feature convenient. Read-only and external volumes stay read-only.
- **No silent upload.** No network transmission of assets, metadata, paths, or previews without explicit user action and a visible destination. Telemetry is opt-in. An imported project cannot trigger network access.
- **Fail closed on ambiguous ownership.** Quarantine or mark the conflict, preserve every claim and file, and refuse destructive actions until a user assigns an owner.
- **Preserve recoverable data.** Write temporary files with restrictive permissions, fsync where supported, rename atomically, keep a journal, and never delete the source before commit. A failed or interrupted operation leaves the previous committed state recoverable.

## 4. Interfaces and dependencies

**Asset reference (planned fields; exact encoding is an ADR decision):**

```text
AssetRef
  schema_version, workspace_id, asset_id
  owner_product, document_id, locator_kind, locator
  format_id, format_version, revision, content_hash
  kind, display_name, linked_at, provenance_id
```

`locator_kind` is workspace-relative, absolute-local, or product-managed. Absolute locators are machine-specific; export converts them to unresolved references or embedded snapshots with an explicit choice. Hash the canonical bytes or the owner-defined canonical serialization.

**Interchange manifest (planned):**

```text
InterchangeManifest
  schema_version, package_id, created_by_product, created_by_version, created_at
  assets[]: asset_id, owner_product, document_id, format_id, format_version,
            revision, content_hash, payload_path, fidelity, dependencies[]
  provenance[], checksums[], signature?
```

Payload paths are relative, normalized, and free of symlinks, absolute paths, `..`, duplicates, and case collisions. Every payload is listed and every listed payload is present. Verify hashes before import. Fidelity is one of `verified-roundtrip` (fixture-proven for the declared feature set), `partial` (list lost or approximated features), or `reference-only` (requires the owner and original asset). Do not claim full fidelity without round-trip evidence. Verify a signature when present; accept unsigned packages only per policy.

**Canonical document contract.** The owner product exposes a stable document ID, format and version capabilities, revision and hash query, open with an expected revision, atomic save returning a new revision and hash, export, and a typed conflict error. It rejects a stale expected revision. CraftAnything never writes canonical bytes directly.

**Conflict resolution protocol.** The reference carries the baseline revision and hash. The owner rejects a stale expected revision with a typed conflict. CraftAnything stops the write, preserves both revisions, reports the conflict, and applies only an explicit resolution: keep the owner revision, save as a new asset, or cancel. It records the outcome in provenance and verifies the resulting revision.

**IPC message families (planned):** `workspace.*` (open, close, query), `asset.*` (resolve, relink, link, unlink), `document.*` (open, query-revision, save, export, import), `ownership.*` (claim, verify), `provenance.*` (append, read), `version.*` (negotiate), and `conflict.*` (report, resolve). Every message uses a versioned envelope with request ID, capability scope, deadline, payload bound, and typed error taxonomy.

**Version negotiation.** Handshake on protocol major/minor, product ID and version, and supported document and interchange formats. Select the highest common major. If none exists, fail closed with an actionable error. Never downgrade silently; record negotiated versions in the workspace and provenance. Test the compatibility window defined in the ADR.

**Dependency rules:**

| Consumer | May depend on | Must not depend on |
|---|---|---|
| `apps/craftanything` | `eqwec-core`, `eqwec-storage`, `eqwec-ipc`, `eqwec-ui`, CraftAnything-specific crates | Other product app crates or editor engines |
| PhotoCraft, VectorCraft, FilmCraft, EffectCraft, AudioCraft | `eqwec-core`, `eqwec-storage`, `eqwec-ipc`, and `eqwec-ui` as needed; published `craftanything-assets` and `craftanything-interchange` APIs when participating | `apps/craftanything`, `craftanything-provenance` internals, or other product app crates |
| `craftanything-*` crates | Shared core, storage, IPC, and the shared security boundary | Product crates or app entry points |
| Shared crates | Their declared dependencies only | Any product or CraftAnything crate |

## 5. Resource concerns

- **Workspace size:** store references and metadata, not duplicate payloads. Warn or enforce a quota; account for optional embedded snapshots and package staging.
- **Asset indexing:** incremental, persisted, and rebuildable from the manifest. Use bounded batches and cancellation; do not decode full payloads to index.
- **Thumbnail cache:** content-addressed by hash and revision, bounded in memory and on disk, LRU-evicted, and regenerated on miss. Validate before decode; never treat a thumbnail as canonical.
- **Memory:** stream or bound manifest parsing; cap asset count, provenance length, package entries, and decode buffers. Reject over-limit input with a diagnostic instead of exhausting memory.
- **Startup:** open from the manifest without resolving every asset. Resolve lazily, show unresolved state, and measure a startup budget.
- **File watching:** watch only workspace roots and explicitly linked roots. Bound the watch count, debounce events, handle overflow with a rescan, and stop on workspace close. Never watch a home directory, drive root, or the whole filesystem.
- **Background scanning:** low priority, cancellable, bounded concurrency and queue length. Do not scan when the workspace is closed or the volume is unavailable.
- **Install size:** do not link every editor engine into CraftAnything. Use product registration metadata and optional components.
- **Cross-product launch latency:** pass IDs and revisions, not document payloads. Reuse a running owner instance where supported, enforce a timeout, and measure cold and warm launch.

## 6. Required end-to-end workflows

Fixtures below are test data to create during implementation; none is claimed to exist. Run each workflow on every supported platform where applicable.

| Workflow | Fixture | Steps | Success criteria |
|---|---|---|---|
| Create workspace | `ws-empty` | Create, write the manifest atomically, close, reopen | Same workspace ID and schema version; no asset copies; no network |
| Link assets | `assets-mixed`, `assets-external` | Link one asset per product plus an external file | References store owner, format, revision, and hash; payloads are not copied; reopen does not resolve every payload |
| Open in owner | `assets-mixed` | Open an asset from the workspace | The correct owner receives document ID and revision; CraftAnything does not open an editor surface; an unknown owner fails with an actionable error |
| Edit | `assets-mixed` | Edit and save in the owner | Atomic save; new revision and hash; CraftAnything writes no canonical bytes |
| Return | `assets-mixed` | Return to the workspace | Reference updates only after owner acknowledgment; provenance records the edit; one asset, no duplicate |
| Resolve conflict | `conflict-stale` | Edit behind the reference, then return | No overwrite; both revisions preserved; explicit choice applied; resolution recorded |
| Export package | `assets-mixed` | Export with declared fidelity | Manifest validates; hashes match; paths are safe; fidelity is honest |
| Reopen package | Exported package | Import into a clean workspace | Same references and ownership; unresolved stays unresolved; no network; no silent canonicalization |

| Adversarial case | Fixture | Required result |
|---|---|---|
| Missing asset | `missing-asset` | Workspace opens; reference is unresolved; metadata retained; relink possible; no crash |
| Renamed or moved asset | `moved-asset` | Relink by ID and hash; no same-name substitution; provenance records the relink |
| Two products claim ownership | `dual-claim` | Fail closed; both claims preserved; no writes; explicit assignment required |
| Version mismatch | `version-skew` | Actionable failure; no downgrade; no partial import; workspace unchanged |
| Interrupted save | `interrupted-save` | Old revision intact or new revision complete; never truncated; journal recovers; temporary files cleaned |
| Hostile package | `hostile-package` | Traversal, symlink, oversized thumbnail, or malformed manifest is rejected; no write outside the workspace |

## 7. Failure modes, debugging, tests, platforms

| Failure | Required behavior |
|---|---|
| Owner unavailable | Mark unresolved; do not fall back to a local copy or editor |
| IPC timeout or owner crash | No partial write; retry only idempotent operations; report the failure |
| Corrupt manifest or schema | Fail closed; preserve the original; emit a bounded diagnostic |
| Stale revision | Report a conflict; never refresh silently |
| Permission denied | Mark read-only or unresolved; do not escalate |
| External volume removed | Mark affected references unresolved; preserve metadata; allow relink |
| Watcher overflow | Rescan and report that events may have been missed |
| Corrupt cache | Discard and rebuild; never treat it as canonical |
| Disk full | Atomic write fails; previous revision intact; temporary files cleaned |
| Case collision | Reject or flag the import; never merge silently |
| Symlink escape | Reject the operation; no write outside the allowed root |

**Debugging procedure:** reproduce on a fixture and work on a copy; record product, protocol, schema, and format versions; inspect the manifest, references, and provenance read-only; verify hashes and distinguish missing, moved, and changed; check owner registration and the IPC handshake; check permissions, locks, symlinks, case, and mount state; inspect temporary files and the journal to determine the committed revision; fix with a failing regression test, then rerun the full workflow. Never mutate the original evidence; redact secrets and personal paths.

**Regression tests:** schema round-trip and unknown-field handling; traversal, absolute path, symlink, junction, case collision, reserved name, and long path; one-owner enforcement and dual-claim rejection; stale revision and concurrent edit; version negotiation and unknown message type; missing, moved, and relink; same-name different-hash substitution; interrupted save at each stage and journal recovery; verified-roundtrip, partial, and reference-only fidelity; asset, manifest, thumbnail, watch, and queue bounds; cache eviction and rebuild; cross-platform path behavior. Every fix starts with a test that fails before the fix.

**Cross-platform caveats:** path case sensitivity differs (macOS and Windows are usually case-insensitive; Linux is usually sensitive), and Unicode normalization differs on macOS. Symlinks and Windows junctions need containment checks; some volumes do not support them. File locking is mandatory on Windows and advisory on Unix; the owner may hold a lock, so time out instead of deleting a locked file. External drives may remount at a new path, disappear, be read-only, or lack permissions, symlinks, or stable file IDs. Atomic rename requires the same filesystem; fsync directory metadata where supported. Handle separators, reserved names, alternate data streams, and long paths. Network filesystems may have unreliable watching and locking; degrade to manual refresh and unresolved state. Use platform bookmarks or scoped access where the sandbox requires them. Never use modification time as document identity.

## 8. Release, documentation, and handoff

**Documentation triggers:** update the workspace, asset-reference, interchange, and provenance specifications and their ADRs when a schema changes; update the threat model for ownership, conflict, path, or IPC changes; update the compatibility matrix and IPC contract for message or version changes; update the dependency map for a new product integration; update `AGENTS.md` for skill activation; update the changelog, compatibility notes, and version metadata for a release. Update `README.md` status only when the capability is real. Do not create fake commands, links, or shipped claims.

**Release triggers:** all required workflows and adversarial cases pass; cross-product contract tests pass; no unresolved critical security finding; package, signature, install, update, rollback, and uninstall checks pass per `eqwec-engineering-core`; the independent CraftAnything version, changelog, and compatibility notes are correct. Tag as `craftanything/vX.Y.Z`. A CraftAnything release does not require or imply a release of any other product.

**Handoff:** use `eqwec-engineering-core` for architecture, shared-crate ownership, contracts, versioning, security policy, and Git. Use `eqwec-quality-control` for pre-change, implementation, verification, performance, merge, and release gates. Use the owning product skill for canonical format behavior. If this skill conflicts with the core skill, the core skill wins and the conflict is a skill defect.

## Red flags

- CraftAnything becoming a second editor.
- Two products owning or mutating the same document.
- Lossy or reference-only interchange presented as full fidelity.
- Stale asset references treated as valid.
- Silent overwrite or last-writer-wins on conflict.
- File watching or background scanning without bounds.
- Duplicating mutable document bytes into the workspace.
- Trusting imported paths, manifests, thumbnails, or metadata.
- Silent upload, telemetry, or network fetch from an imported project.
- Auto-picking an owner when ownership is ambiguous.
- Treating a thumbnail, index, or cache as the source of truth.
- Scanning a home directory, drive root, or unrelated user data.
