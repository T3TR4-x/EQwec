# ADR-0006: Project serialization and storage

- **Status:** ACCEPTED
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0001, ADR-0004, `docs/architecture/SYSTEM-ARCHITECTURE.md`, `docs/security/THREAT-MODEL.md`

## 1. Context

Every EQwec creative product needs a durable project format. Users must be able to create, edit, save, reopen, recover, and export work without data loss. The format also carries security obligations: project files are untrusted input, may contain large assets, and must not execute scripts or fetch external resources.

A single shared format across PhotoCraft, VectorCraft, FilmCraft, EffectCraft, and AudioCraft would either be too generic to serve each domain or become a universal editor by another name. The project needs a shared container and serialization discipline, while each product owns its document schema.

## 2. Requirements

- Crash-safe saves with a recoverable previous version.
- Versioned schemas with tested migrations.
- Large binary assets without duplicating them in memory or on disk.
- Human-inspectable metadata where practical.
- Bounded parsing of untrusted projects.
- Content addressing for assets and cache keys.
- A single-file package for sharing and a directory bundle for working projects.
- Product-owned document schemas; no shared universal document model.
- Cross-product interchange through explicit export/import packages, not live foreign project mutation.
- No script execution, network fetch, or external entity resolution during open.
- Deterministic serialization for tests and signatures where needed.

## 3. Alternatives considered

### A. Single SQLite database per project

Crash-safe transactions, WAL, migrations, and random access. But a project database is opaque to users, large blobs require careful streaming, and schema changes couple every product to SQLite. SQLite is public domain and mature, but it is better for catalogs and caches than for the canonical creative document.

### B. Custom binary format

Compact and fast, but requires a parser, versioning, tooling, and migration infrastructure for every product. Hard to inspect and easy to make unsafe. Not justified for the first release.

### C. ZIP container with JSON manifest and assets

A directory bundle packed into a ZIP. Human-inspectable, widely understood, easy to export, and compatible with existing archive tools. Requires bounded ZIP parsing and a clear asset index. Good fit for the first release.

### D. CBOR/MessagePack/Protobuf

Compact and schema-friendly, but adds a dependency and tooling for little benefit at the first-release scale. JSON metadata is small relative to pixel/audio/video payloads. Reconsider if profiling shows metadata is a bottleneck.

### E. Directory bundle only

Simple, crash-safe with atomic writes, and easy to inspect. Users often expect a single file for sharing and backups. Offer both: a working directory bundle and a single-file ZIP package.

## 4. Evidence

- SQLite is public domain and supports transactions, WAL, and crash recovery. It is widely used for catalogs and caches.
- ZIP is a widely supported container; its parser must be bounded against entry count, uncompressed size, compression ratio, duplicate names, absolute paths, and symlinks.
- JSON is human-readable and has mature Rust support (`serde_json`, MIT OR Apache-2.0).
- SHA-256 is a standard content-addressing hash with mature Rust implementations.
- No EQwec project format exists yet; this ADR defines the container and leaves product schemas to the owning product.

## 5. Decision

### 5.1 Project bundle

A working project is a directory bundle:

```text
project.eqwec/
  manifest.json        # schema version, product, document ID, timestamps, asset index
  document.json        # product-owned document tree and properties
  assets/<sha256>      # content-addressed binary payloads
  preview.png          # optional preview
  recovery/            # optional journal or previous manifest
```

Rules:

- `manifest.json` has a schema version, product ID, document ID, created/modified timestamps, asset entries with SHA-256 and byte size, and a product-specific schema version.
- `document.json` is owned by the product. Shared crates provide serialization helpers, not a universal document model.
- Assets are content-addressed by SHA-256. The same bytes are stored once per project. A product may reference external media by path plus hash for FilmCraft; it must not silently copy or mutate the original.
- `preview.png` is optional and never authoritative.
- Unknown fields are preserved or explicitly rejected according to the schema; they are never silently dropped.
- A project may be packed into a single-file `.eqwec` ZIP with the same layout. The ZIP parser enforces entry count, uncompressed size, compression ratio, path rules, and duplicate-name rejection.

### 5.2 Writes and recovery

1. Write to a temporary file in the target directory.
2. Flush the file and the directory entry.
3. Rename atomically over the target.
4. Retain the previous manifest or a journal for recovery.
5. On open, validate the manifest, verify asset hashes on demand, and offer recovery if the last write was incomplete.
6. Never leave a mixed old/new tree. Recovery yields a complete old or complete new state.

### 5.3 Catalogs and caches

- Use SQLite for catalogs, indexes, thumbnails, and caches. SQLite is not the canonical project state.
- Caches are disposable, rebuildable, bounded, and scoped by product/profile.
- Define eviction policy and size limits per product.
- Do not store secrets, credentials, or unredacted personal data in caches.

### 5.4 Migrations

- Every schema has a version and a migration path.
- Migrations are tested from every supported previous version to the current version.
- A failed migration leaves the original project intact and produces an actionable error.
- Downgrades are not silently attempted; a project saved by a newer version opens read-only or refuses with a clear message.

### 5.5 Interchange

- Cross-product exchange uses an explicit package with a manifest, provenance, and fidelity declaration.
- CraftAnything references canonical documents; it does not own or mutate their bytes.
- Import/export adapters are product-owned and tested against representative fixtures.
- Proprietary formats are imported/exported only within documented fidelity and legal boundaries; no full-compatibility claim without evidence.

## 6. Security requirements

- Treat project files, manifests, assets, and ZIP containers as untrusted.
- Bound file size, entry count, nesting depth, decompressed bytes, compression ratio, and parse time.
- Reject absolute paths, `..`, symlinks, duplicate names, NUL bytes, and platform-specific path escapes.
- Do not execute scripts, resolve external entities, or fetch network resources during open.
- Verify asset hashes before use; a mismatch is a corruption error, not a warning.
- Use restrictive file permissions for temporary files and clean them on failure.
- Fuzz the manifest, document, and ZIP parsers.
- Do not log project contents or personal paths by default.

## 7. Consequences

**Positive**

- Crash-safe, inspectable projects with a clear recovery path.
- Product-owned schemas avoid a universal document model.
- Content-addressed assets reduce duplication and support cache keys.
- A single-file ZIP package is easy to share and back up.
- SQLite handles catalogs and caches without becoming the project format.

**Negative**

- JSON metadata can be large for very complex documents; profiling may require a binary format later.
- ZIP parsing is a security-sensitive surface and needs fuzzing.
- Two representations (bundle and ZIP) require round-trip tests.
- External media references in FilmCraft can break if the user moves files; link health and relinking are required.

**Security**

- The parser is a primary attack surface; bounds and fuzzing are mandatory.
- Hash verification detects corruption and tampering but is not a signature.
- Project signatures, if added, require a separate design and key management.

**Performance**

- Measure open/save time, memory, and asset verification cost on representative projects.
- Large assets are streamed, not loaded whole, where the product allows.
- Do not claim performance without a fixture and a baseline.

**Licensing**

- `serde_json` is MIT OR Apache-2.0; SQLite is public domain; `rusqlite` is MIT; SHA-256 implementations are permissive.
- ZIP tooling is permissive; verify the exact crate at admission.
- No proprietary format is required by this decision.

**Maintenance**

- Schema migrations are a permanent obligation; every product owns its migration tests.
- The container is shared; changes require cross-product impact assessment.
- A future binary metadata format can be added behind the same manifest contract.

## 8. Reconsideration criteria

Revisit this ADR if:

- JSON metadata becomes a measured bottleneck for large documents.
- ZIP parsing cannot be made safe enough for untrusted projects.
- A product requires random access or incremental saves that the bundle cannot provide.
- Cross-product interchange requires a richer package than the manifest/assets model.
- SQLite catalogs become a shared service with concurrency or corruption issues.
- Project signatures or encryption become product requirements.

## 9. Links

- `docs/architecture/SYSTEM-ARCHITECTURE.md`
- `docs/security/THREAT-MODEL.md`
- `docs/security/SECURITY-BASELINE.md`
- ADR-0004
- ADR-0001
