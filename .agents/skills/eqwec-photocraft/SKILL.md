---
name: eqwec-photocraft
description: "Use for any PhotoCraft task covering layer-based, non-destructive image editing and photography: the photo document model, adjustments and filters, masks, selections, brushes, history, RAW decode, color management, tiled rendering, native project format, and export. Trigger when a change touches `apps/photocraft`, a PhotoCraft-specific crate, photo project or export formats, photo fixtures, performance budgets, or a PhotoCraft user workflow. Do not use for VectorCraft, FilmCraft, EffectCraft, AudioCraft, Hub, or Browser work, for shared-crate changes with no photo-specific behavior, or for unrelated repositories."
---

# PhotoCraft

PhotoCraft is the EQwec layer-based, non-destructive image editor for photography. This skill adds domain constraints to `eqwec-engineering-core`; it never weakens the core rules.

**Status: planned.** No PhotoCraft feature, crate, project format, or command is implemented or validated. Treat every path and interface below as planned until the workspace and architecture documents prove otherwise. Do not describe any capability as shipped.

## Activation

Use this skill when a task:

- touches `apps/photocraft` or a PhotoCraft-specific crate;
- changes the photo document model, layer stack, adjustment graph, masks, selections, brushes, history, RAW boundary, color management, project format, or export;
- changes photo fixtures, golden images, performance budgets, or PhotoCraft release notes;
- reviews a PhotoCraft diff or verifies a PhotoCraft workflow.

Do not use it when a task:

- belongs to VectorCraft, FilmCraft, EffectCraft, AudioCraft, Hub, or Browser;
- changes only a shared crate with no photo-specific behavior — use core and the shared crate contract instead;
- is unrelated to the EQwec repository.

Load `eqwec-engineering-core` first, and `eqwec-quality-control` before changing anything and again before claiming completion. If this skill conflicts with core, core wins and the conflict is a skill defect.

## 1. Ownership and paths

As of this writing, `apps/` does not exist and `crates/` contains only `eqwec-core`. Every path below is **planned**. Confirm names against `docs/architecture/SYSTEM-ARCHITECTURE.md` and `docs/architecture/DEPENDENCY-MAP.md`. Do not create an empty crate merely to match this table.

| Path | Status | Owner and responsibility |
|---|---|---|
| `apps/photocraft` | Planned | Product entry point, editor UI, workflow orchestration, packaging |
| `crates/eqwec-graphics` | Shared, planned | GPU device/resource management, rendering abstractions |
| `crates/eqwec-media` | Shared, planned | Bounded decode/encode, media pipeline primitives |
| `crates/eqwec-storage` | Shared, planned | Tenant-scoped persistence, atomic project writes |
| `crates/eqwec-ui` | Shared, planned | Design tokens and UI primitives, not photo workflow logic |
| `crates/eqwec-photo-doc` | Planned, name proposed | Document model, layer stack, project schema |
| `crates/eqwec-photo-adjust` | Planned, name proposed | Adjustment/filter graph, CPU reference implementations |
| `crates/eqwec-photo-mask` | Planned, name proposed | Mask buffers, feather, density, invert, compositing |
| `crates/eqwec-photo-select` | Planned, name proposed | Selection model, vector paths, raster coverage, mask conversion |
| `crates/eqwec-photo-brush` | Planned, name proposed | Brush engine, stroke capture, coalescing |
| `crates/eqwec-photo-history` | Planned, name proposed | Command and graph-delta undo/redo, memory budget |
| `crates/eqwec-photo-raw` | Planned, name proposed | RAW decoder boundary and camera metadata mapping |
| `crates/eqwec-photo-color` | Planned, name proposed | Color-space model, ICC transforms, display conversion |

Ownership rules:

- Shared crates are owned by `integration`, not PhotoCraft. Propose shared API changes through an ADR and an impact assessment naming every consumer.
- Dependency direction is `apps/photocraft` → PhotoCraft crates → shared crates. Shared crates must not depend on a product; no app depends on another app.
- PhotoCraft owns the native PhotoCraft project format. Other products consume a documented export or interchange contract, never the live project.
- Every document path, recent-file entry, cache key, and recovery file carries an explicit profile or tenant scope. No global default for tenant-owned data.
- One crate per real boundary with an owner, API, tests, and a second consumer or documented architectural reason.

## 2. Domain model and invariants

### Hierarchy

- Document → layer stack → layers and groups → adjustment/filter nodes → masks → selections.
- Layer kinds: pixel, adjustment, fill, group. Groups nest with a bounded depth. A layer carries a stable `LayerId`, visibility, opacity, blend mode, mask references, and color-space tag.
- The adjustment/filter stack is an ordered graph, not a list of baked results. Nodes declare typed, versioned parameters, input/output ports, and a deterministic seed where randomness exists.
- Masks attach to a layer or node. Selections are independent document objects used as an edit scope or converted to a mask.
- The document holds canvas size, resolution, working color space, bit depth, layer stack, graph, history policy, and color-managed display configuration.

### Non-destructive edit graph

- Evaluate the graph on demand and cache node results. Invalidate by content hash and parameter version, not by guess.
- Never bake adjustments into layer pixels by default. `Flatten`, `rasterize`, and `apply` are explicit, warned, undoable operations.
- Keep the imported source intact. Crop, transform, and canvas resize store parameters and a canvas rectangle instead of overwriting source pixels.
- Separate `assign profile` (metadata reinterpretation) from `convert to profile` (pixel transform). Never conflate them.
- Save the graph, parameters, and masks — not only the rendered result.

### Color, depth, alpha, gamma

- Declare one working color space per document. Composite in linear light. Convert to display space only in the final display transform.
- Decode gamma-encoded input to linear before compositing and encode once at display or export. Never apply a transfer function twice or mix encoded and linear values in one operation.
- Evaluate adjustments in f32; document storage may be u8, u16, or f32. Export preserves the chosen depth.
- Use one alpha convention per buffer contract. Default: premultiplied alpha in linear working space for pixel and composite buffers; straight alpha only at import/export boundaries; mask coverage is straight and unassociated.
- Every buffer carries a color-space ID and alpha mode. No implicit conversion or reinterpretation at a function boundary.
- Handle missing or unknown profiles explicitly. Assuming sRGB requires a user-visible notice; never do it silently.
- Dither on bit-depth reduction where banding would otherwise be visible.

### Tiles and scanlines

- Default to fixed tiles (proposed 256×256) with an apron/halo for filters. Use scanline buffers only below a documented image-size threshold.
- Tile grid origin is top-left in document coordinates. Edge handling is deterministic. Zero row padding; never expose uninitialized memory.
- A filter requests its declared apron and reads no farther. Filters that need neighbors without an apron produce seams and are defects.

### Undo/redo

- Model history as commands (user intent) or graph deltas (node and parameter changes), not full-image snapshots by default.
- Coalesce brush strokes and slider drags into one user-visible step. Bound history by bytes and entry count.
- A new edit invalidates redo. History must not resurrect deleted layer IDs or stale document versions.
- Default policy: do not persist undo history in the project. Reopening starts with a clean history and clean dirty state. Change this only by documented format decision.

### Identity and ownership

- Use stable `DocumentId`, `LayerId`, `NodeId`, `MaskId`, and `SelectionId` with generation counters. Never reuse IDs.
- Scope IDs and storage paths to a profile or tenant. No global mutable document registry.
- One writer owns a live document. Render workers receive immutable snapshots. Do not hold a lock across a long render or an `await`.
- A document is either fully loaded and validated or not opened. No half-valid state is rendered or saved.

### Never destructive by default

- Never overwrite the original imported image or RAW file.
- Never bake adjustments, filters, masks, selections, transforms, or crop into pixels without explicit user action.
- Never discard the previous saved project on a failed save.
- Export always writes a new target and never mutates the document.

## 3. Threat boundaries and security

- Treat raster images, RAW files, project files, sidecars, LUTs, presets, brushes, plugins, fonts, and clipboard data as untrusted.
- Bound decode before allocation: maximum width, height, pixel count, decompressed bytes, frame count, metadata size, and decode time. Reject overflow in `width × height × channels × bytes_per_sample`.
- Defend against decompression bombs by streaming, checking declared against actual size, and aborting at the limit. Do not trust headers alone.
- Reject malformed dimensions: zero, negative, overflowing stride, mismatched channel count, absurd DPI, inconsistent EXIF orientation.
- Enforce path safety: traversal, absolute-path injection, Windows drive and UNC prefixes, symlink escape, and NUL bytes. Re-validate external links on every open and require explicit consent.
- Treat plugins and third-party filters as untrusted code. Allowlist or verify signatures, scope capabilities, run out of process where feasible, and bound CPU, memory, and time. No implicit filesystem or network access.
- Validate GPU inputs: texture dimensions and formats, binding sizes, uniform ranges, and sampler state. Shaders come from the product-owned set, never from project files. Handle device loss and fall back to the CPU renderer.
- Do not upload images, thumbnails, filenames, EXIF, or project data silently. Network features are opt-in, disclose the endpoint and payload, and telemetry excludes pixels and paths.
- On corrupt project data, fail closed. Do not apply a partial layer stack. Offer read-only recovery when possible, preserve the last recoverable version, and never replace corrupt data with an empty document.
- Write atomically: temp file in the same directory, fsync where required, rename, and keep a recoverable previous version. Clean temp files on failure.
- Isolate decoder FFI, document every pointer-ownership and thread-safety invariant, and fuzz the boundary.

## 4. Interfaces and dependencies

Define and version these contracts before implementing features. No code below is validated.

- **Image buffer:** dimensions, stride, sample format, channel count, color-space ID, alpha mode, tile or scanline layout, ROI view, and mutability. No implicit copy or conversion.
- **Color space:** profile identity or bytes, transfer function, primaries, white point, rendering intent, transform cache key, and black-point compensation policy.
- **Tile cache:** byte budget, pin/unpin, deterministic eviction, thread-safe access, and metrics.
- **Adjustment graph:** typed versioned parameters, ports, deterministic evaluation, serialization roundtrip, and a CPU reference implementation.
- **Mask:** coverage resolution and scale, bit depth, coordinate space, feather/density/invert semantics, and deterministic resampling.
- **Selection:** vector path plus raster coverage, antialias policy, coordinate space, and explicit conversion to a mask. Marching ants are UI only.
- **Project format:** versioned schema, checksums, unknown-field policy, forward compatibility, migration, atomic save, recovery journal, and embedded versus referenced assets.
- **Export:** documented subset of PNG, JPEG, TIFF, and WebP; bit depth, alpha, ICC embedding, metadata policy, and deterministic encoder settings.
- **RAW boundary:** one adapter over a vetted decoder, bounded resources, metadata-only versus full decode, camera profile and white-balance mapping, and an explicit unsupported-camera error.
- **Renderer parity:** the CPU renderer is authoritative. The GPU renderer must match it within a documented tolerance on the same fixture. GPU is optional; CPU fallback always works.

Dependency rules:

- Keep dependency direction acyclic.
- Route third-party codecs through `eqwec-media` or the RAW adapter; do not scatter decoder calls through the UI.
- Record each dependency's purpose, license, maintenance status, and replacement cost.
- Do not add a dependency for a few lines that the standard library or an existing crate already provides.

## 5. Resource concerns

- Render high-resolution images by tile on demand and by ROI. Do not allocate a full-resolution duplicate per adjustment or layer.
- Memory-map source files read-only only when safe; decode to tiles for editing and copy before mutation. Do not assume a stable mapping for removable or network files.
- Budget the tile cache by bytes, not entry count. Pin visible and active tiles, add hysteresis, bound prefetch, and expose metrics.
- Budget GPU memory explicitly. Reuse staging buffers, choose texture formats deliberately, evict under pressure, and recover from device loss.
- Bound undo history by bytes and coalescing. Prefer deltas and commands over snapshots.
- Keep startup lazy: no full-library scan, no eager full-resolution decode, and a bounded thumbnail cache.
- Export with streaming or tiled encode, a bounded queue, cancellation, and progress reporting.
- Cache ICC transforms and LUTs per source, destination, and intent. Avoid per-pixel profile lookups; transform whole tiles.
- Keep RAW codecs, GPU backends, and optional assets as documented components. Measure installer size impact.
- Avoid duplicate full-image buffers. Use ROI views, copy-on-write, and reference counts. Measure peak RSS, not only steady state.

## 6. Required end-to-end workflows

Create deterministic, license-clean fixtures. Do not use personal photographs or images with unknown provenance.

Fixtures: 8-bit RGBA gradient PNG with known values; 16-bit TIFF with alpha and a known profile; JPEG with EXIF orientation and no profile; procedurally generated large image (proposed 20000×20000); redistributable camera RAW; project containing a layer, adjustment, mask, and selection; truncated project and truncated image; ICC v2 and v4 profiles; high-frequency tile-boundary pattern.

| ID | Workflow | Success criteria |
|---|---|---|
| W1 | Open an image | Dimensions, orientation, profile, and bit depth match the fixture; source hash unchanged; no unintended full-image duplicate |
| W2 | Apply adjustments | Add exposure, contrast, and curve nodes; toggling each returns original pixels within tolerance; parameters persist; evaluation is deterministic |
| W3 | Add and edit a mask | Paint, feather, invert, and delete; composite changes only where coverage exists; mask survives save/reopen; no pixel destruction |
| W4 | Use a selection | Rectangle and lasso, feather, convert to mask; selection does not alter pixels; conversion is deterministic |
| W5 | Save and reopen native project | Layer IDs, order, graph, parameters, masks, selections, and color settings match; pixel result matches exactly or within documented tolerance |
| W6 | Export a standard format | Independent decoder reads the file; dimensions, profile, alpha, and depth are correct; original file untouched; same settings produce the same output |
| W7 | RAW workflow | Decode a supported fixture, adjust white balance and exposure, export; orientation and metadata follow policy; unsupported camera fails clearly without a crash |
| W8 | Large-image tiled workflow | Navigate, edit, and export under the memory ceiling; no tile seams; cancellation leaves a consistent document |
| W9 | Undo and redo | A mixed sequence restores the exact prior graph and pixels; a new edit invalidates redo; memory stays bounded; dirty state is correct |
| W10 | Autosave and recovery | Simulated crash mid-edit recovers the last consistent state; corrupt recovery data fails closed; no silent overwrite |
| W11 | Color-managed export | Convert to a target profile and embed it; soft proof is not baked; independent conversion matches within tolerance; assign and convert remain distinct |

For every workflow record the fixture hash, the repository-documented command, environment, observed result, and `PASS`, `FAIL`, or `BLOCKED`. Discover commands from `AGENTS.md` or CI; do not invent them. Until the workspace exists, execution is `BLOCKED`, not passed.

## 7. Failure modes, debugging, regression tests, cross-platform

### Failure modes

- Decoder rejects a valid file or accepts malformed input.
- Out-of-memory or tile-cache thrash under a large document.
- GPU device loss, or GPU/CPU output divergence.
- Missing, wrong, or double-applied ICC profile.
- Tile seams and edge-dependent filter output.
- Undo corruption, stale IDs, or unbounded history.
- Project version mismatch or failed migration.
- Export writes the wrong profile, alpha mode, or bit depth.
- Unsupported RAW camera or decoder-version difference.
- Disk full, interrupted save, or crash during autosave.

### Debugging procedure

1. Reproduce with the smallest fixture and record the document graph, schema version, and settings.
2. Localize the boundary: decode, graph evaluation, mask/selection, compositing, display transform, or encode. Test each with a known buffer.
3. Render the same node on the CPU reference and compare tiles; find the first divergent tile or pixel.
4. Check the color-space chain and alpha mode at every boundary. A mismatch is a defect, not a rounding difference.
5. Check filter apron and edge handling for seam defects.
6. Check cache budget, eviction, peak RSS, GPU memory, and device-loss logs.
7. Verify save atomicity and recovery behavior; inspect the project schema version and checksums.
8. Record OS, GPU backend and driver, ICC profiles, decoder version, and fixture hash. Do not log pixels, filenames, or EXIF by default.

### Regression tests

- Golden-image tests against the CPU reference with a documented tolerance; exact hashes only on lossless deterministic paths.
- Property tests for graph serialization roundtrip, undo/redo invariants, and selection-to-mask determinism.
- Fuzzing for image decoders, the RAW adapter, the project parser, and LUT/preset parsers.
- Adversarial tests for huge dimensions, decompression bombs, truncated files, path traversal, and malformed ICC data.
- Tile-seam tests at every edge and corner, including the image border.
- Color-transform tests across working, display, and export spaces.
- Crash injection during save and autosave, followed by recovery verification.
- Endianness variants for 16-bit samples and TIFF byte order.

### Cross-platform caveats

- **GPU backends:** Metal on macOS, DX12 on Windows, Vulkan on Linux. Detect features, keep a CPU fallback, and set parity tolerance per backend.
- **Color profiles:** ICC v2 and v4, display profile changes, macOS ColorSync, Windows WCS, Linux colord. Test with and without a display profile.
- **File dialogs:** macOS sandbox bookmarks, Windows drive and UNC paths, Linux portals. Validate returned paths; do not assume POSIX semantics.
- **RAW codecs:** decoder version and camera support differ by platform and build. Pin the decoder and never claim camera support beyond tested fixtures.
- **Endianness:** TIFF byte order, 16-bit sample order, and GPU upload paths. Test little- and big-endian fixtures where supported.
- **Filesystem:** case sensitivity, Unicode normalization (macOS NFD), long paths, permissions.
- **DPI and scaling:** logical versus physical pixels; export resolution must not depend on display scaling.

## 8. Release, documentation, and handoff

Triggers:

- Update `README.md` status only after the required workflow passes. Never describe a planned capability as shipped.
- Update the product spec, architecture and dependency map, threat model, project-format spec, export matrix, performance baseline, and changelog when affected.
- Record version impact. PhotoCraft has an independent version and stays `0.0.0` until a release gate passes.
- Release notes must state supported formats, platforms, RAW cameras, color-management limits, and known defects.
- Packaging must account for optional codecs and GPU components, their licenses, and installer size.

Handoff:

- `eqwec-engineering-core` for architecture, ownership, security policy, Git, versioning, and documentation conflicts.
- `eqwec-quality-control` for pre-change, implementation, verification, performance, merge, and release gates.
- Record the skill usage matrix. A missing mandatory skill is `BLOCKED`, not `NOT_RELEVANT`.

## Red flags

- Adjustment, filter, mask, selection, or crop baked into pixels without explicit user action.
- Working and display color spaces conflated, or sRGB assumed silently.
- Alpha mode or gamma changed silently at a boundary.
- Tile seams, missing aprons, or edge-dependent output.
- Unbounded undo history or a full-image snapshot per step.
- GPU and CPU renders diverge beyond tolerance, or a GPU-only path has no fallback.
- Original file overwritten on save or export; a failed save destroys the previous project.
- Corrupt project partially applied or replaced by an empty document.
- Decoder trusts header dimensions or declared decompressed size.
- Plugin or project-supplied shader runs with ambient filesystem, network, or GPU access.
- Image, thumbnail, filename, or EXIF uploaded without explicit opt-in.
- Full-image buffer duplicated per layer or adjustment; memory measured only at steady state.
- A feature described as implemented without a passing end-to-end workflow.
