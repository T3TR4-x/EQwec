---
name: eqwec-vectorcraft
description: "Use for any VectorCraft task covering vector document editing and SVG interoperability: artboards, layers and groups, Bézier paths and node editing, fills, strokes, gradients, boolean operations, hit-testing, snapping, text-on-path, units and precision, rasterization and export, and the native vector project format. Trigger when a change touches apps/vectorcraft, a VectorCraft-specific crate, vector fixtures, SVG import or export behavior, or shared eqwec-graphics, eqwec-ui, or eqwec-media contracts that affect VectorCraft. Do not use for PhotoCraft, FilmCraft, EffectCraft, AudioCraft, Hub, Browser, or CraftAnything-owned workspace work, for shared-crate changes with no VectorCraft behavior, or for unrelated repositories."
---

# VectorCraft

VectorCraft is the planned EQwec vector-graphics editor: artboards, Bézier path editing, fills, strokes, gradients, text-on-path, boolean operations, and SVG interoperability. This skill adds domain rules to `eqwec-engineering-core`; it never weakens core or quality rules.

**Status: planned.** No Cargo workspace, `apps/vectorcraft`, VectorCraft crate, native project schema, SVG parser, renderer, or validated command exists. Every path, crate name, format, and workflow below is a design requirement, not evidence of implementation. Do not describe planned behavior as shipped.

## Status and authority

- The repository is at version `0.0.0`; VectorCraft has no independent release. No feature is implemented or validated.
- Verify every proposed path and crate name against `docs/architecture/SYSTEM-ARCHITECTURE.md` and `docs/architecture/DEPENDENCY-MAP.md` when they exist. Report drift; do not create empty crates to match this skill.
- `eqwec-engineering-core` governs architecture, ownership, security, Git, versioning, and documentation. `eqwec-quality-control` governs gates and evidence. If this skill conflicts with core, core wins and the conflict is a skill defect.
- Discover real commands, schemas, and test results from `AGENTS.md` and CI. Do not invent commands, compatibility claims, performance numbers, or passing results.
- VectorCraft owns its native vector project format and vector semantics. Other products consume a documented export or interchange contract, never the live project.

## Activation

Use this skill when a task touches:

- `apps/vectorcraft` or a VectorCraft-specific crate, fixture, test, or release artifact.
- The vector document model, artboards, layers/groups, paths, nodes, handles, subpaths, styles, or native project schema.
- Bézier geometry, intersections, boolean union/subtract/intersect/exclude, stroke expansion, path simplification, or geometric validation.
- Hit-testing, selection, snapping, alignment, guides, transforms, viewport zoom/pan, or precision and units.
- Fills, strokes, gradients, patterns, clipping, masking, blend modes, or text-on-path.
- SVG import, SVG export, SVG feature-matrix decisions, or another vector interchange format.
- Rasterization, anti-aliasing, GPU tessellation, deterministic export, DPI, color management, or raster export formats.
- A shared `eqwec-graphics`, `eqwec-ui`, `eqwec-storage`, `eqwec-config`, or `eqwec-media` change that alters VectorCraft behavior or its contracts.
- VectorCraft security, performance, documentation, packaging, or release review.

Do not use it for:

- PhotoCraft raster editing, FilmCraft editing/playback, EffectCraft compositing, AudioCraft DSP, Hub installation/update, or Browser engine work.
- CraftAnything workspace or asset-exchange policy; use that skill for the exchange contract and this skill only for the vector document semantics it carries.
- A shared-crate change with no VectorCraft consumer or behavior change; core still governs it.
- Generic Rust refactoring, repository policy, or an unrelated repository.

Load `eqwec-engineering-core` first, and `eqwec-quality-control` before changing anything and again before claiming completion. Record the skill matrix; a missing mandatory skill is `BLOCKED`, not `NOT_RELEVANT`.

## 1. Ownership map

All product paths are planned. Proposed crate names are placeholders until the architecture documents ratify them.

| Path | State | Owner and responsibility |
|---|---|---|
| `apps/vectorcraft` | Planned | Product shell, editor UI orchestration, commands, document lifecycle, export jobs, packaging. |
| `crates/vectorcraft-doc` | Planned, name proposed | Document schema, artboards, layers, styles, stable IDs, canonical serialization, migrations. |
| `crates/vectorcraft-geom` | Planned, name proposed | Bézier math, intersections, tolerances, boolean adapter, topology validation. |
| `crates/vectorcraft-path` | Planned, name proposed | Node/handle editing, subpath operations, stroke expansion, path simplification. |
| `crates/vectorcraft-svg` | Planned, name proposed | Bounded SVG parser/writer, feature matrix, namespace and unit handling, sanitization. |
| `crates/vectorcraft-render` | Planned, name proposed | CPU reference rasterizer, tessellation, cache, export composition. |
| `crates/vectorcraft-text` | Planned, name proposed | Text layout, font resolution, text-on-path, outline/embed policy. |
| `crates/eqwec-core` | Shared, planned | Typed IDs, errors, result conventions. VectorCraft consumes; never forks. |
| `crates/eqwec-ui` | Shared, planned | Design tokens and UI primitives only. No vector workflow logic. |
| `crates/eqwec-graphics` | Shared, planned | GPU device/resource abstractions and renderer contracts. Consume; no product-only forks. |
| `crates/eqwec-storage` | Shared, planned | Tenant-scoped persistence, atomic writes, recovery primitives. |
| `crates/eqwec-config` | Shared, planned | Validated configuration and migration. |
| `crates/eqwec-logging` | Shared, planned | Structured diagnostics with redaction. |
| `crates/eqwec-media` | Shared, planned | Raster encode/decode boundary only where VectorCraft exports or imports raster images. |
| `docs/product-specs/vectorcraft.md` | Planned | Product scope, native format, SVG profile, supported platforms, budgets. |
| `tests/fixtures/vectorcraft/` | Planned | Deterministic geometry, SVG, font, rendering, adversarial, and round-trip fixtures. |

Ownership rules:

- Dependency direction is `apps/vectorcraft` → VectorCraft crates → shared crates. Shared crates must never depend on a product or app; no app depends on another app.
- A shared-crate change requires an impact assessment naming every consumer and tests that exercise VectorCraft and the other affected products. Do not add VectorCraft-only behavior to a shared crate.
- One crate per real boundary with an owner, public API, tests, and a second consumer or documented architectural reason. Do not scaffold a crate merely because this table names it.
- The native project format, SVG compatibility matrix, tolerance policy, and boolean algorithm decision are VectorCraft-owned contracts and require versioning.

## 2. Domain model and invariants

| Term | Meaning |
|---|---|
| Document | Canonical state: schema version, units, artboards, layer tree, styles, resources, settings. |
| Artboard | Bounded canvas with origin, size, unit, and optional background. It is not the viewport. |
| Layer/group | Ordered container with transform, opacity, blend mode, visibility, and lock state; groups nest with a bound. |
| Path | Compound geometry with stable `PathId`, fill rule, ordered subpaths, and a style reference. |
| Node/anchor | On-curve point with stable `NodeId` and incoming/outgoing handles. |
| Handle | Control point stored relative to its anchor; zero-length is valid and creates a zero-derivative endpoint, not a missing value. |
| Segment | Explicit line or cubic Bézier between consecutive nodes. Type is stored; no silent curve/line conversion. |
| Subpath | Ordered node sequence, explicitly open or closed. |
| Fill/stroke | Style properties and paint servers. Stroke is a style, not baked path geometry. |
| Gradient | Linear or radial paint with ordered stops, spread method, transform, and declared interpolation space. |
| Transform | Affine 2×3 matrix in document units, composed parent-to-child in a documented order. |
| Viewport | Zoom, pan, and device-pixel mapping. It is UI state and never document geometry. |
| Selection | Stable object IDs plus optional node/segment sub-selection. It carries no geometry. |
| Unit | Document unit and display unit, with an explicit conversion and export DPI. |

Critical invariants:

- **Canonical path data.** Serialization uses one command vocabulary and one numeric format, is idempotent, and preserves geometry within the declared model tolerance. Canonicalization must not reorder user-visible nodes or subpaths or renumber IDs.
- **Stable identity.** `DocumentId`, `ArtboardId`, `LayerId`, `PathId`, and `NodeId` survive save/reopen. Deleted IDs are never reused. Import assigns IDs by a documented deterministic rule, not randomness.
- **Open versus closed.** Closure is explicit. A closed subpath contributes a closing segment only when its endpoints differ. Fill of an open subpath may use an implicit closure; stroke never implicitly closes it.
- **Winding and fill rule.** Every compound path declares `nonzero` or `evenodd`. Winding and orientation are computed deterministically; boolean output must set a fill rule and orientation consistent with its holes.
- **Transforms versus baked geometry.** Path data is local; transforms stay live by default. Baking a transform is explicit, warned, undoable, and rejected for singular or near-singular matrices.
- **Stroke is not geometry.** Stroke expansion is an explicit operation for booleans, outlining, or export. Joins, caps, dashes, miter limit, and non-uniform scale behavior must be declared.
- **Boolean robustness.** A vetted algorithm or library is chosen in an ADR. Inputs are validated, degeneracies handled, output topology validated, and failure returns a typed error without destroying the inputs. No robustness claim without adversarial fixtures.
- **Scale-aware tolerance.** Absolute and relative tolerances are separate. One global epsilon is forbidden. Model equality, boolean snapping, hit-testing, and raster coverage use different tolerance classes.
- **Hit-testing space.** Tolerance is specified in screen pixels and converted to document space through the inverse viewport transform. Stroke hits account for half the stroke width and the declared join/cap geometry.
- **Selection and edit scope.** Selection is by stable IDs; node and segment sub-selection is explicit. Selection carries no geometry, operations apply in document order, and a stale or deleted ID fails the operation instead of silently retargeting.
- **Snapping.** Candidates come from visible grids, guides, artboard edges, and object geometry. Tolerance is in screen pixels converted through the inverse viewport transform; candidate priority and tie-break are deterministic; a snap preview never mutates the document before commit.
- **Undo/redo.** History is reversible commands or validated deltas, not full-document snapshots by default. Undo restores the exact prior state, including IDs and ordering; a new edit clears redo; history is bounded.
- **Non-destructive by default.** Live shapes, transforms, gradients, strokes, and text remain editable. Rasterize, outline, flatten, and bake are explicit, warned, undoable operations.
- **Round-trip fidelity.** Save/reopen preserves geometry, IDs, order, units, styles, artboard metadata, and unknown extension data. Any lossy conversion is disclosed in the UI and in the compatibility matrix.
- **No silent geometry loss.** Import, export, simplification, and repair never drop nodes, subpaths, or style information without an explicit policy, warning, or rejection.
- **Units are explicit.** Coordinates are never unitless by accident. Unit conversion, percentage resolution, and export DPI have documented rounding at the boundary, not in the model.
- **One writer.** A live document has one writer. Render and export workers consume immutable snapshots; do not hold a lock across a long render.

## 3. Geometry and numerics

- Store document geometry in `f64`. Use `f32` only for GPU uploads and renderer intermediates after an error analysis; never round-trip canonical path data through `f32`.
- Reject `NaN`, infinities, and coordinates outside a documented finite range at every parse and edit boundary. Guard squared-distance, determinant, and subdivision calculations against overflow.
- Use `eps = eps_abs + eps_rel * scale`, where `scale` is the local coordinate magnitude, segment length, or bounding-box diagonal appropriate to the predicate. Document each predicate's class and test it at small and large coordinates.
- Tolerance classes: model equality (tight, serialization), geometric predicate (intersection, coincidence), boolean snap (larger, only during normalization), hit-test (screen-derived), and raster coverage (backend-specific). Never reuse a hit-test tolerance for boolean topology.
- Classify degeneracies explicitly: zero-length segments, coincident endpoints, collinear overlaps, cusps, zero-length handles, duplicate nodes, tangent intersections, self-intersections, zero-area lobes, and holes touching the outer contour. Each operation declares accept, normalize, or reject.
- A zero-length subpath with round or square caps may render a dot or square; butt caps render nothing. This is rendering semantics, not a reason to delete the subpath.
- Boolean operations require an ADR covering algorithm, license, maintenance, numerical behavior, fill-rule handling, open-path policy, result orientation, determinism, and fallback. Validate results for closure, finite coordinates, self-intersection policy, duplicate nodes, and expected area relationship.
- Boolean failure must preserve the original paths and report the operation, inputs, tolerance, and failing predicate. Never return an empty path, a partially clipped path, or a "best effort" result as success.
- Normalize only inside the operation: snap coincident points, split intersecting segments, and order output deterministically. Do not mutate the source document during the operation.
- Path simplification is a user-visible command with an explicit tolerance, preview, and undo. It may remove nodes and change topology only with a warning. Never simplify automatically on import, save, export, or paste.
- Transform composition order is fixed and tested. Invert only when the determinant exceeds a documented threshold; otherwise reject the operation with a diagnostic.
- Text-on-path is not pure path geometry: glyph layout, font metrics, start offset, side, and baseline affect the result. Changes to the path must reflow deterministically or report why they cannot.
- Validate all geometry after import and before export. Validation failures are errors, not warnings that allow a corrupt document to be saved.

## 4. SVG and interchange

- Choose an explicit SVG profile in an ADR: SVG 1.1 plus a named SVG 2 subset, or a narrower profile. Never claim "full SVG" support. The compatibility matrix is the contract.
- The feature matrix must state, per feature, whether import preserves it, renders it, approximates it, or rejects it, and whether export emits it. Cover shapes, paths, groups/transforms, gradients, patterns, clip/mask, filters, text, images, `use`/`symbol`, CSS, animation, scripts, `foreignObject`, and external references.
- Parse with a namespace-aware, bounded XML parser. Disable DTDs, external entities, and network access. Reject undeclared prefixes, duplicate attributes, and invalid numeric values with a location-bearing diagnostic.
- Unknown elements and attributes are preserved as opaque, namespaced extension data or rejected explicitly. Never silently drop rendering-critical content. If appearance cannot be preserved, refuse the import rather than show a misleading document.
- Support presentation attributes and a documented inline-style subset with correct cascade precedence. Do not fetch external stylesheets, `@import`, images, fonts, or `xlink:href` targets. `javascript:` and script-bearing URLs are rejected.
- Resolve each element's transform in SVG order, and map the root `viewBox`/`preserveAspectRatio` to the document viewport. Percentages require a viewport; `em`/`ex` require a font context. Unresolvable units are rejected or handled by a documented fallback.
- Use an explicit unit table and DPI assumption. CSS `px` follows the SVG 96-per-inch convention but is not automatically the document unit. Physical units, percentages, and font-relative units are converted at the correct boundary with documented rounding.
- Text is font-dependent. Declare the font-resolution policy, missing-font behavior, fallback substitution, and whether export keeps text or outlines it. Do not promise visual fidelity without the font or a tested substitute.
- Filters, clipping, and masking are supported only by the declared subset. If a filter or mask is unsupported, preserve it and warn, or reject the import; do not ignore it and render a different image.
- Define round-trip fidelity levels: exact geometry and style, visually equivalent within tolerance, or lossy with disclosure. Test each level with independent parsers and at least two SVG viewers; a single viewer is not proof of interoperability.
- Export deterministically: stable element and attribute order, no random IDs or timestamps unless documented, normalized numbers, explicit units, and no host-specific paths. Record the writer version and profile in the output metadata where appropriate.
- VectorCraft owns the native project. CraftAnything or another product receives a documented SVG/export package, never a mutable project file or internal IDs it cannot validate.

## 5. Rendering and precision

- The CPU rasterizer is the correctness reference. GPU path rendering is optional and must match the CPU output within a documented tolerance on the same fixture; GPU-only behavior is a defect.
- Decide the anti-aliasing and compositing model in an ADR: coverage quality, gamma/linear blending, alpha convention, and color space. Never blend encoded and linear values in one operation.
- Choose GPU tessellation, stencil-and-cover, or another path-rendering strategy by measurement and parity tests. Handle device loss with a CPU fallback and never let a tessellation bug reach export silently.
- Stroke rendering must define joins (miter, round, bevel), caps (butt, round, square), dash pattern and phase, miter limit, zero-length subpaths, and behavior under non-uniform scale. Cosmetic or non-scaling strokes require an explicit feature decision.
- Gradients define stop ordering and duplicate-offset resolution, spread method, transform, focal point, degenerate radius, and interpolation color space. Clamp or reject out-of-range offsets explicitly; do not rely on renderer accidents.
- Blend modes and group isolation are explicit. If an export format or backend cannot represent a mode, warn or reject; never silently substitute normal blending.
- Zoom-independent quality: rerasterize at the current zoom and DPI. A cache key includes document revision, path/style hash, viewport transform, device scale, anti-aliasing mode, and backend. Never stretch a low-resolution cache into an export.
- Level of detail is allowed only for interactive preview. Export always uses full geometry and the declared quality; culling and approximation must not change output.
- Export maps document units to pixels with an explicit origin, scale, rounding rule, and pixel-grid alignment. Output dimensions and DPI are independent of window size and display scaling.
- Color management declares the document working space, display transform, export profile, and alpha handling. Do not assume sRGB silently; if a profile is missing, require a documented default and a user-visible notice.
- Raster export starts with a documented subset, normally PNG with declared bit depth and alpha. JPEG, WebP, or other formats require encoder, color, alpha, metadata, and determinism tests before they appear in the UI.
- Text export either embeds a licensed font, references it with a disclosed dependency, or outlines it explicitly. Outlining is a destructive visual choice and must be warned and undoable in the project.
- Performance: use a spatial index for hit-testing and culling, cache tessellation by content and transform, invalidate on geometry/style/backend change, bound caches by bytes, and never tessellate the whole document on every pan or zoom. Bound path complexity and node counts for interactive operations; degrade preview explicitly, but export must process the full path or return a typed resource error.

## 6. Threat boundaries and security

Imported SVG, clipboard vector data, fonts, images, project files, and metadata are untrusted. Fail closed.

| Threat | Required control | Failure behavior |
|---|---|---|
| Malicious SVG or path data | Bound file size, element count, nesting depth, path commands, coordinates, gradient stops, and parse time. Reject overflow and non-finite values. | Reject with a bounded diagnostic; no partial document. |
| External references | Do not fetch network, file, font, image, stylesheet, or entity references during import. Resolve only explicit, user-approved local assets. | Reject or leave unresolved with a visible placeholder; never silently fetch. |
| XXE and entity expansion | Disable DTDs and external entities; cap entity expansion and total parsed bytes; reject recursive definitions. | Abort parsing before allocation or expansion limits. |
| Decompression/path bombs | Bound `.svgz` and archive decompression, compression ratio, output bytes, and time; stream where possible. | Abort and clean temporary data. |
| Script and event injection | Never execute scripts, event handlers, `javascript:` URLs, or embedded active content. Strip or reject on import; never emit them on export. | Reject the active content; do not open a partially trusted document. |
| Font parsing | Treat fonts as untrusted binaries. Use a vetted parser, bound glyph/table sizes, isolate crashes, and never execute font instructions. | Reject the font and substitute only with explicit disclosure. |
| Clipboard/paste | Treat pasted SVG, paths, text, and images as untrusted imports. Revalidate geometry, units, styles, and references. | Reject invalid content; paste nothing rather than a corrupt path. |
| Plugin or extension injection | If extensions exist, use a capability manifest, trust class, and sandbox or out-of-process execution. No ambient filesystem or network access. | Refuse to load an unverified extension; preserve the document. |
| File path traversal | Normalize paths; reject absolute paths, `..`, drive/UNC prefixes, NUL bytes, symlink escapes, and case/Unicode collisions where relevant. | Reject the operation; never write outside the chosen root. |
| Non-atomic save or autosave | Write a restricted temp file in the target directory, flush, rename atomically, and retain a recoverable previous version. | Keep the old file; quarantine the temp; report failure. |
| Untrusted metadata | Treat titles, descriptions, author fields, and embedded thumbnails as display-only data. Sanitize, bound, and never render as markup. | Drop unsafe metadata with a warning; do not execute it. |
| Unsafe export overwrite | Require explicit overwrite consent, resolve symlinks, check the final path, and write atomically. | Refuse or preserve the existing file; never truncate it first. |

Additional rules: do not log document contents, clipboard payloads, personal paths, or font data by default. Do not add a "skip security checks" debug path to a production build. If a check cannot run, record `BLOCKED`; never call it passed.

## 7. Resource budgets and measurement

No numeric budget is approved yet. Set budgets in `docs/product-specs/vectorcraft.md` and the performance records before implementation. Every claim needs hardware, OS, build profile, workload, exact command, and observed result; otherwise mark it `BLOCKED`.

| Concern | Required measurement | Evidence |
|---|---|---|
| Cold startup | Time to an interactive empty document, per platform and build. | Baseline record; no network dependency. |
| Warm startup | Same workload after OS cache warm-up. | Baseline record. |
| Idle memory | RSS after a bounded idle period with a representative document. | Profiler and process tree. |
| Document open/save | Time and peak memory versus node, path, and embedded-asset count. | Fixture matrix and timing log. |
| Pan/zoom frame time | p50/p95/p99 frame time at declared zoom levels and document sizes. | Frame trace; no dropped interaction in the target workload. |
| Boolean latency | Time and peak memory per operation versus input complexity and degeneracy class. | Benchmark corpus, including failure cases. |
| Export throughput | Pixels or artboards per second at declared DPI, format, and color settings. | End-to-end export timing. |
| Install size | Compressed and installed bytes per platform, including fonts and optional codecs. | Package manifest and measured tree. |
| Memory per node/path | Bytes per node, segment, style, and cached tessellation. | Allocation measurement; bounds documented. |
| Cache bounds | Byte budget, hit rate, eviction rate, and invalidation behavior. | Cache metrics under large-document stress. |

A performance improvement claim needs a reproducible baseline and a real workload, not a microbenchmark. If a measurement cannot run in the environment, the gate is `BLOCKED` and the claim is removed from release notes.

## 8. Required end-to-end workflows

Fixtures must be deterministic, license-clean, and repository-owned. Build a geometry corpus (lines, cubics, cusps, zero-length handles, open/closed subpaths, holes, self-intersections), an SVG feature corpus, font fixtures with known licenses, adversarial files, golden renders, and large-document fixtures. Do not use personal artwork or unknown-license fonts.

| ID | Workflow | Success criteria |
|---|---|---|
| V-E2E-01 | Create shapes, paths, and artboards; move, scale, rotate, and reorder them. | Geometry is exact; transforms stay live unless baked; artboard units are explicit; save/reopen preserves IDs and order. |
| V-E2E-02 | Edit Bézier nodes and handles: add, delete, move, break, join, open, close, cusp, smooth. | Result is deterministic; node IDs are stable; open/closed semantics hold; undo restores the exact prior geometry. |
| V-E2E-03 | Apply union, subtract, intersect, and exclude to overlapping, tangent, coincident, holed, and near-degenerate paths. | Valid closed output where required; fill rule and orientation correct; area relation within tolerance; no duplicate nodes; failure is typed and preserves inputs. |
| V-E2E-04 | Save and reopen a native document containing artboards, groups, gradients, strokes, text, and unknown extension data. | IDs, order, units, styles, and geometry round-trip; canonical serialization is stable; no silent loss; reopening is deterministic. |
| V-E2E-05 | Import and export the SVG feature corpus, then compare with independent parsers and at least two viewers. | Supported features match the matrix; unsupported features are preserved or rejected with disclosure; no scripts or external fetches; export is deterministic. |
| V-E2E-06 | Create text on a path, edit the path, change font, and export. | Layout reflows deterministically or reports failure; missing fonts are disclosed; text remains editable or is explicitly outlined; export matches the declared policy. |
| V-E2E-07 | Render linear/radial gradients and strokes with every join, cap, dash, miter, and scale case. | CPU golden output matches within tolerance; GPU parity holds or CPU fallback is used; export DPI does not change style semantics. |
| V-E2E-08 | Pan, zoom, select, and hit-test a large document at multiple scales. | Interaction meets the declared frame budget; hit-testing is correct in screen and document space; memory and caches stay bounded; preview LOD never affects export. |
| V-E2E-09 | Perform a mixed edit sequence, undo, redo, then make a new edit. | Exact prior state is restored; redo is cleared correctly; history stays bounded; IDs and selection remain valid; save/reopen history policy is explicit. |
| V-E2E-10 | Kill the process during save and autosave, then recover. | Last consistent state is recovered; corrupt recovery data fails closed; the previous file is preserved; temp files are cleaned; no silent overwrite. |
| V-E2E-11 | Import malformed and hostile SVG: XXE, entity expansion, external refs, scripts, deep nesting, huge paths, non-finite numbers, and bombs. | Each is rejected within bounds; no network access or code execution; diagnostic identifies the class; no partial document is opened or saved. |
| V-E2E-12 | Export at declared units and DPI with fractional scaling and color settings. | Pixel dimensions, origin, rounding, alpha, profile, and metadata are exact; output is independent of display scaling; repeated exports are deterministic. |

Record fixture hash, command, environment, and `PASS`, `FAIL`, or `BLOCKED`. Until the workspace exists, every workflow is `BLOCKED`, not passed. A happy-path SVG file is not a feature matrix.

## 9. Failure modes, debugging, regression, and platforms

### Failure modes

- Degenerate geometry: zero-length segments, coincident nodes, cusps, collinear overlaps, self-intersections, and holes touching boundaries.
- Boolean failure: invalid topology, lost holes, duplicate nodes, wrong fill rule, non-deterministic output, or a silently empty result.
- Precision drift: scale-dependent epsilon, f32 contamination, accumulated transform error, or rounding inside the model.
- Font substitution and missing glyphs changing text-on-path layout or export appearance.
- GPU tessellation artifacts: cracks, seams, missing antialiasing, stencil overflow, or CPU/GPU divergence.
- Clipboard format differences: vector versus raster fallback, lost units/styles, or untrusted content bypassing import validation.
- Platform file dialogs, path semantics, sandbox bookmarks, and case/Unicode normalization.
- Color-management mismatch between document, display, and export; DPI scaling errors; SVG viewer differences.

### Debugging procedure

1. Reproduce with the smallest fixture; record document schema version, units, transform stack, fill rule, tolerance class, renderer, backend, and build.
2. Localize the boundary: parse → document model → geometry → style → tessellation/raster → display or export.
3. Dump canonical path data and validate finite coordinates, closure, winding, IDs, and deterministic serialization.
4. For booleans, reduce to the minimal input pair; record operation, tolerance, algorithm/library version, and expected area/topology; test against the adversarial corpus.
5. Check coordinate spaces: local path, layer transform, artboard, viewport, device pixels, and export pixels. A hit-test or tolerance in the wrong space is a defect.
6. Compare CPU reference and GPU output tile by tile; find the first divergent pixel or coverage boundary.
7. Verify save/reopen with an independent reader and byte-level canonical comparison; check unknown-content preservation and recovery files.
8. Record OS, GPU/driver, fonts, display scale, color profile, and viewer versions. Do not log document contents or personal paths by default.
9. Fix, add a regression test, and rerun the affected workflow matrix. Mark unavailable checks `BLOCKED`.

### Regression tests

- Unit: canonicalization idempotence, numeric formatting, ID stability, open/closed semantics, fill-rule winding, transform order, tolerance scaling, stroke joins/caps/dashes, gradient stops.
- Property: path parse/serialize round-trip, boolean output validity, undo/redo invariants, hit-test consistency across zoom, and deterministic export.
- Golden: CPU raster fixtures at declared DPI and color settings; GPU parity within a stated tolerance.
- Adversarial: SVG parser fuzzing, XXE/entity/network rejection, huge path and deep nesting, non-finite numbers, decompression bombs, and clipboard payloads.
- Round-trip: native project and SVG feature matrix with independent readers and multiple viewers.
- Platform: font substitution, DPI scaling, color profiles, file dialogs, clipboard formats, and GPU backends.
- Crash injection during save/autosave, followed by recovery verification.
- Every fixed defect gets a regression test before the fix is accepted.

### Cross-platform caveats

- **macOS:** Retina/backing scale, ColorSync profiles, sandbox and security-scoped bookmarks, system font availability, native dialogs, and Preview/Safari SVG differences.
- **Windows:** per-monitor DPI awareness, WCS/ICC profiles, drive and UNC paths, DirectWrite font substitution, clipboard formats such as EMF and SVG, and case-insensitive path collisions.
- **Linux:** X11 versus Wayland fractional scaling, colord/ICC, fontconfig substitution, portal file dialogs, case-sensitive paths, and GPU driver variation.
- **All platforms:** logical versus physical pixels; export must not depend on display scaling. SVG viewer behavior differs; compatibility claims are scoped to tested viewers and versions. Cross-compilation is not platform verification. If a platform check cannot run, mark it `BLOCKED`.

## 10. Release, documentation, and handoff

Release review is required when a change affects the native project schema, SVG profile or feature matrix, boolean algorithm, tolerance policy, renderer/backend, export formats or DPI behavior, text/font policy, supported platforms, security boundary, or performance budget.

Update the affected documents or record why no change is needed:

- `docs/product-specs/vectorcraft.md` — planned scope, native format, SVG profile, units, and support matrix.
- `docs/architecture/SYSTEM-ARCHITECTURE.md` and `docs/architecture/DEPENDENCY-MAP.md` — planned ownership and contracts.
- `docs/security/THREAT-MODEL.md` — planned SVG, font, clipboard, and export threats.
- Native format spec, SVG compatibility matrix, boolean-algorithm ADR, tolerance policy, performance records, and changelog.
- `README.md` status, `AGENTS.md` skill map, and release notes only after the required workflows pass.

VectorCraft has an independent version and stays `0.0.0` until its release gate passes. Release notes must state the SVG profile, supported features, known round-trip losses, tested viewers, platforms, and font limitations. Never describe an alpha or prototype as stable.

Handoff:

- `eqwec-engineering-core` — architecture, shared-crate ownership, security baseline, Git, versioning, and documentation lifecycle.
- `eqwec-quality-control` — pre-change, implementation, verification, performance, merge, and release gates. Provide the evidence matrix and exact commands actually run.
- CraftAnything skill — workspace and asset-exchange contracts; this skill supplies vector document semantics only.
- If this skill conflicts with core, core wins and the conflict is a skill defect.

## Red flags

- Claiming robust booleans without an ADR, adversarial fixtures, topology validation, and failure-path tests.
- Storing document geometry in `f32` without an error analysis or round-tripping canonical data through the renderer.
- Silently simplifying, quantizing, or reordering paths on import, save, paste, or export.
- Dropping unknown SVG elements, attributes, or namespaces without disclosure, or claiming "full SVG" from one happy-path file.
- Executing SVG scripts, event handlers, or `javascript:` URLs, or fetching external entities, stylesheets, images, or fonts on import.
- Parsing XML without disabling DTDs, external entities, and network access, or parsing without element/depth/size bounds.
- Saving non-atomically, truncating the target before writing, or losing the previous document on failure.
- Using one fixed epsilon everywhere, or a tolerance that does not scale with coordinates and segment length.
- Hit-testing in screen space with a document-space tolerance, or ignoring the viewport transform and stroke width.
- Baking transforms, strokes, text, gradients, or effects without explicit user action, warning, and undo.
- Renumbering or reusing node/path IDs on canonicalization, import, or save/reopen.
- Conflating open and closed subpaths, ignoring the fill rule, or returning boolean output with inconsistent winding.
- Expanding strokes silently, or treating a stroke as a zero-width path in boolean operations.
- Treating text as pure path geometry and assuming a font is available, or substituting fonts without disclosure.
- Allowing GPU and CPU renders to diverge without a parity test or a CPU fallback.
- Exporting dimensions or geometry that depend on window size, display scaling, or a stretched preview cache.
- Returning an empty or partial path as boolean success, or swallowing a geometry error to keep the UI responsive.
- Letting a corrupt or partially parsed document reach the canvas, save path, or autosave recovery.
- Claiming SVG round-trip fidelity without a feature matrix, independent parsers, and multiple viewers.
- Stating a performance or compatibility result without hardware, OS, build, workload, command, and observed evidence.
