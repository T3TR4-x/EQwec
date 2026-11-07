# ADR-0012: Hub package archive v1

- **Status:** ACCEPTED (in-process gates 1–11 pass locally; gate 12 is partially exercised on macOS and remains `BLOCKED` for uninstall and the Windows/Linux fixture runs)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0006, ADR-0007, ADR-0011, `docs/product-specs/eqwec-hub.md`, `docs/product-specs/eqwec-hub-schemas.md`, `docs/security/THREAT-MODEL.md`

## 1. Context

The Hub manifest v1 declares a package `source`, exact `size`, and one SHA-256 `hash`. The product specification requires exact size and hash verification before extraction and activation, and bounded extraction by entry count, path length, decompressed bytes, ratio, and time. It also requires rejecting absolute paths, `..`, drive and UNC prefixes, NUL bytes, unsafe links, device files, and setuid/setgid bits.

The specification does not define the package container. Without a normative format, extraction code would have to guess at entry types, permission bits, duplicate handling, and header precedence. Those guesses are security decisions. A wrong guess permits traversal, link escape, archive bombs, or header-confusion attacks.

ADR-0011 defines the signed manifest, registry, journal, and installation layout but deliberately leaves the package container open. ADR-0007 mentions `tar.zst` as a Linux platform-packaging format for Hub itself, not as the product package format. ADR-0006 selects a ZIP-based single-file project package. No Hub download, extractor, or fixture product exists.

## 2. Requirements

- The container must be covered by the manifest SHA-256; no per-entry signature is required.
- Size and hash must be verified before the container is parsed.
- The extractor must reject every path that can escape the staging root on Windows, macOS, or Linux.
- Links, device files, and special files must be impossible to create.
- Permission bits must be deterministic and must never carry setuid, setgid, or sticky semantics.
- Entry count, declared and actual uncompressed bytes, compression ratio, path depth and length, and extraction time must be bounded before or during extraction.
- Duplicate and case-insensitive-colliding paths must be rejected, including Unicode-normalization collisions.
- The parser must reject ambiguous or contradictory headers rather than choose one.
- Extraction must write only inside a fresh staging directory and must not follow a pre-existing symlink.
- The staged tree must be durable before the journal records `staged`.
- The format must be buildable and testable with permissively licensed, pure-Rust dependencies on the verified platform.
- The format must be inspectable with common tooling so release engineering does not depend on custom code.

## 3. Alternatives

### 3.1 ZIP, stored or deflate (chosen)

A single ZIP or ZIP64 archive with regular files and directories. The central directory gives the entry count, names, compression methods, sizes, and Unix modes before any entry is decompressed. Deflate is the only compressed method; stored entries are allowed for already-compressed or tiny files.

Advantages:

- Preflight entry count and declared sizes without decompressing the archive.
- Random access to each entry, so one bad entry can be rejected before writing earlier entries.
- Unix mode bits are representable in external attributes; non-Unix writers can omit them and receive deterministic defaults.
- The pure-Rust `zip` crate is MIT-licensed and widely used. Deflate is provided by `flate2` with the pure-Rust `zlib-rs` backend; `flate2` is MIT or Apache-2.0 and `zlib-rs` is Zlib-licensed. Both are permissively licensed.
- Release tooling on every platform can produce and inspect ZIP files.

Costs:

- ZIP has two headers per entry. The central directory is authoritative, but a malicious archive can disagree with the local header. The extractor must validate both and reject disagreement.
- The format supports encryption, links, and many compression methods. The extractor must reject all of them explicitly.
- Deflate is weaker than zstd for large binaries, so packages are larger than a `tar.zst` equivalent.

### 3.2 `tar.zst`

A POSIX tar stream compressed with zstd.

Advantages:

- Natural Unix metadata and streaming extraction.
- Better compression than deflate for large binaries.
- `tar` and `zstd` have pure-Rust crates with permissive licenses.

Costs:

- Entries are discovered only by decompressing the stream. Entry count, total bytes, and ratio cannot be fully preflighted without a first decompression pass.
- Tar has several generations of extensions: GNU long names, PAX headers, sparse files, hardlinks, and vendor extensions. A safe extractor must reject most of them, which is a larger parser surface than the v1 product needs.
- zstd adds a second decompressor and its own frame and window bounds. The manifest size bound limits the compressed input but not the decompressor's memory window unless configured separately.
- Hardlinks have no place in an immutable product tree, and rejecting them requires explicit tar-type handling.

### 3.3 Custom content-addressed bundle

A directory of content-addressed blobs plus a JSON index.

Advantages:

- Trivial to bound and verify per file.
- No archive parser.

Costs:

- Not a single file, so the manifest package hash no longer covers one artifact.
- Requires custom tooling for creation, inspection, and distribution.
- Duplicates the project-format decision in ADR-0006 without a product need.

### 3.4 Per-entry signatures

Sign every file and directory independently.

Advantages:

- A single corrupt entry can be isolated.

Costs:

- The manifest already signs the package hash; per-entry signatures add size and verification cost without changing the trust anchor.
- Revocation, expiry, and key selection would have to be repeated per entry.

Per-entry signatures are rejected for v1.

## 4. Evidence

- APPNOTE 6.3.x defines the ZIP central directory, local file header, general-purpose flags, compression methods, ZIP64 records, and external attributes.
- The `zip` crate documents `validate_header_consistency`, which rejects duplicate central-directory names, overlapping entries, local/central mismatches, and local headers missing from the central directory.
- The `zip` crate exposes raw entry names separately from the optional Unicode Path extra field, so the extractor can ignore an unauthenticated name override.
- `flate2` with `zlib-rs` is a pure-Rust deflate implementation. `flate2` is MIT or Apache-2.0; `zlib-rs` is Zlib-licensed. `cargo deny` permits both.
- The Hub product specification requires bounded extraction, rejection of links and special files, and verification before extraction.
- The threat model records path traversal, symlink escape, and archive bombs as Hub threats H-04 and H-05.
- No extractor, fixture package, or end-to-end install existed when this ADR was accepted. A non-production fixture product and in-process install/update/rollback tests now exist; uninstall, installer orchestration, and Windows/Linux runs remain `BLOCKED`.

## 5. Decision

### 5.1 Container

- A Hub product package is one ZIP or ZIP64 archive.
- Multi-disk and spanned archives are rejected.
- The archive must start at byte zero. Prepended data is rejected.
- The archive comment must be empty.
- The archive must contain at least one regular file.
- Encryption, passwords, and AES entries are rejected. Confidentiality belongs to HTTPS and, if ever required, to a separate envelope, not to the package format.
- The only accepted compression methods are `stored` and `deflate`. Every other method is rejected before decompression.
- The central directory is authoritative for entry metadata. The local header must agree with it. The extractor must call the archive-level consistency check before writing any entry.
- Entry names are read from the raw central-directory bytes, never from the Unicode Path extra field or any other name override.
- Data descriptors are accepted only when the central directory declares the sizes; the extractor bounds the actual output independently.

### 5.2 Entry types and modes

- Only regular files and directories are allowed.
- Symbolic links, hardlinks, device nodes, FIFOs, sockets, and every other special type are rejected. ZIP represents a symbolic link as a regular entry whose Unix mode has the `S_IFLNK` type bits; those bits are rejected.
- File modes are normalized to `0644` or `0755`. Directory modes are normalized to `0755`.
- Setuid, setgid, and sticky bits are rejected even when the remaining permission bits are valid.
- A missing Unix mode is treated as `0644` for a file and `0755` for a directory.
- Executable files must be declared with `0755`; there is no implicit executable inference from the name or extension.
- On platforms without Unix modes, the mode is validated and then ignored for creation.

### 5.3 Entry paths

Every entry name must be valid UTF-8 and use only this ASCII set:

```text
A-Z a-z 0-9 . _ - /
```

In addition:

- The name is non-empty and at most 1,024 bytes.
- The name is relative and has no leading `/`.
- The name has no trailing `/` unless it is a directory entry.
- The name has no empty component, no `.` component, and no `..` component.
- The name has no `\`, `:`, NUL, or control character.
- Each component is at most 255 bytes.
- The path has at most 32 components.
- No component ends with `.`.
- No component is a Windows reserved device name, case-insensitively, with or without an extension: `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`, `CONIN$`, `CONOUT$`.
- Paths are unique after ASCII case folding. `A.txt` and `a.txt` collide.
- A path cannot be both a file and a directory, and a file cannot be an ancestor of another entry.

The ASCII-only rule eliminates Unicode normalization collisions on macOS and case-folding ambiguity on Windows. It is intentionally narrower than the set of names a filesystem accepts. A future schema version may extend it with an explicit normalization and folding rule; v1 does not.

### 5.4 Bounds

| Value | Bound |
|---|---|
| Package size | 8 GiB, from the manifest |
| Entries, including directories | 100,000 |
| Total declared uncompressed bytes | 16 GiB |
| Declared uncompressed bytes per entry | 4 GiB |
| Compression ratio per entry | 200:1 |
| Total compression ratio | 100:1 |
| Path length | 1,024 bytes |
| Path component length | 255 bytes |
| Path depth | 32 components |
| Extraction time | 600 seconds by default |

The ratio is `declared uncompressed / max(compressed, 1)`. A stored entry has ratio 1. A zero-byte entry has ratio 0. The total ratio uses the sum of entry compressed sizes, not the package size, so headers do not dilute it.

The caller may lower a bound. The caller may raise a bound only with an explicit policy decision; the implementation must validate that a raised bound still fits the declared package size and available disk. The default is normative for v1.

The extraction time bound is measured with a monotonic clock. It is checked before parsing, before each entry, and during each bounded read. A timeout aborts extraction and removes the partial staging tree.

### 5.5 Verification and staging sequence

1. Open the package file once.
2. Hash and size-check that open handle against the verified manifest.
3. Seek to the start of the same handle. The archive is parsed from the verified handle, not reopened by path.
4. Parse the central directory. Reject multi-disk, nonzero offset, non-empty comment, duplicate names, an empty archive, and an entry count above the bound.
5. Run the archive-level header-consistency check. Reject overlapping entries and local headers missing from the central directory.
6. First pass over entries: validate names, types, modes, compression, encryption, declared sizes, conflicts, totals, and ratios. Write nothing.
7. Create a fresh staging directory. The operation directory must not exist; a pre-existing file, directory, or symlink is rejected. Every ancestor from the profile base to the staging root must be a real directory, not a symlink.
8. Second pass: decompress and write each file with `create_new`, never following a pre-existing final path. Count actual bytes and hash each file. Reject a short or long entry.
9. Sync file data and, on Unix, directory entries. The staged tree must be durable before the journal records `staged`.
10. Walk the staged tree with `symlink_metadata`. Reject symlinks and non-regular files. Compare the actual file set, sizes, and modes with the plan. No extra or missing entry is allowed.
11. Append the `staged` journal record. Only then may activation move the tree.

Any failure removes the partial staging tree. The journal remains at `verified`; startup recovery discards the staging directory and marks the operation failed. No partial tree is promoted.

### 5.6 Journal integration

The staging function requires the last journal record for the operation to be `downloaded` or `verified` and its context to match the operation identity. It records `verified` after the package hash check when the last phase is `downloaded`, then records `staged` after the tree is durable and verified. It never writes `staged` before the tree passes every check.

## 6. Implementation gates

1. A fixed, non-production fixture package stages successfully on macOS with a manifest and trust store.
2. Traversal, absolute paths, drive and UNC prefixes, backslashes, NUL and control characters, empty components, trailing dots, and reserved device names are rejected.
3. Symbolic links, special-file modes, setuid, setgid, and sticky bits are rejected.
4. Duplicate names and ASCII case collisions are rejected.
5. Entry-count, per-entry size, total-size, ratio, path-length, component-length, depth, and timeout bounds are each tested at the boundary.
6. Header confusion is rejected: duplicate central-directory names, local/central disagreement, overlapping entries, unlisted local headers, prepended data, and a non-empty comment.
7. Encryption, unsupported compression methods, multi-disk archives, and empty archives are rejected.
8. A package whose bytes change after manifest verification is rejected before parsing.
9. A pre-existing staging directory or symlinked ancestor is rejected.
10. A failed extraction leaves no partial tree and the journal recovers to `failed`.
11. A staged tree contains exactly the planned files with the planned modes and hashes.
12. A real fixture product installs, updates, rolls back, and uninstalls on macOS, then Windows and Linux.

Gates 1–11 are in-process tests. Gate 12 is end-to-end acceptance. On 2026-11-07 the macOS in-process portion of gate 12 passes: `tests/fixtures/eqwec-hub` builds real v1 and v2 executables, signs manifests with a public non-production seed, installs v1, launches it, updates to v2, rolls back to v1, rejects a tampered package, and recovers an activation interrupted after the tree move. Uninstall, installer orchestration, download transport, platform packaging, and Windows/Linux fixture runs do not exist, so gate 12 remains `BLOCKED`.

Implementation evidence (2026-11-07): gates 1–11 are covered by 31 tests in `crates/eqwec-updater/src/staging.rs`; the macOS fixture portion of gate 12 is covered by 3 tests in `tests/fixtures/eqwec-hub/tests/hub_end_to_end.rs`; `cargo test --workspace --all-features --locked` passed 163 tests locally on macOS. Gate 12 remains `BLOCKED` overall.

## 7. Consequences

**Positive**

- The package container is defined before extraction code exists.
- The central directory allows most bombs to be rejected before decompression.
- The path and mode rules are simple enough to audit and to reproduce on every platform.
- The format uses an MIT-licensed pure-Rust parser and a pure-Rust deflate backend.
- Release engineering can inspect packages with standard tools.

**Negative**

- ASCII-only names reject localized resource names in v1.
- Deflate produces larger packages than zstd.
- ZIP header-confusion handling adds validation code and tests.
- The 16 GiB uncompressed and 100,000-entry bounds may need revision for very large products.
- Files are synced individually, which slows installs with many small files.

**Security**

- The package hash is checked on the same open handle that is parsed, closing the path-reopen window.
- Links and special files cannot be created.
- Path traversal, case collisions, and Windows device names are rejected before writing.
- Header disagreement, overlapping entries, and unlisted local headers are rejected.
- Decompression is bounded by declared size, actual size, ratio, entry count, and time.
- A failed extraction cannot leave a promoted tree.

**Performance**

- Hashing the package before parsing adds one sequential read.
- Deflate decompression is single-threaded in v1. No throughput claim is made until measured.
- Per-file fsync is the dominant cost for packages with many small files. A later version may batch or use a platform durability primitive, but correctness comes first.

**Licensing**

- `zip` is MIT; `flate2` is MIT or Apache-2.0; `zlib-rs` is Zlib; `crc32fast` is MIT or Apache-2.0; `indexmap` and `memchr` are permissively licensed.
- No codec, patent-encumbered format, or DRM dependency is introduced.
- Notices must be recorded before a release artifact ships.

**Maintenance**

- The extractor must track ZIP64 and header-consistency behavior in the `zip` crate.
- A new compression method or path rule is a schema change, not a refactor.
- Windows path behavior remains unverified until Windows tests run.

## 8. Reconsideration criteria

Revisit this ADR if:

- Package size or install time makes deflate unacceptable.
- A product needs localized or Unicode file names.
- The `zip` crate or `zlib-rs` becomes unmaintained or a security advisory has no fixed release.
- Header-confusion testing finds a class the current checks cannot reject.
- Windows or Linux extraction reveals a path rule that cannot be enforced consistently.
- A platform installer requires a different container or per-file metadata.
- End-to-end fixtures show the 16 GiB or 100,000-entry bounds are wrong for a real product.

## 9. Links

- ADR-0006: project serialization and storage.
- ADR-0007: packaging, updates, and signing.
- ADR-0011: Hub manifest, registry, journal, and installation layout.
- `docs/product-specs/eqwec-hub.md`
- `docs/product-specs/eqwec-hub-schemas.md`
- `docs/security/THREAT-MODEL.md`
- `docs/security/SECURITY-BASELINE.md`
- `docs/research/LEGAL-LICENSE-REVIEW.md`
