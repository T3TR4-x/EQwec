---
name: eqwec-effectcraft
description: Use for any EffectCraft work on compositing, animation, visual effects, node or layer graphs, keyframes, GPU/CPU rendering, cache invalidation, or FilmCraft interchange. Trigger when touching apps/effectcraft, EffectCraft-specific crates, the EffectCraft project or effect format, shared graphics/media/UI contracts consumed by EffectCraft, or EffectCraft fixtures, tests, performance, security, documentation, and releases. Do not use for FilmCraft-only editing and playback, PhotoCraft or VectorCraft workflows, AudioCraft DSP, or unrelated repository work.
---

# EffectCraft

EffectCraft is EQwec's planned compositing, animation, and visual-effects product. It owns the effect graph, its evaluation, and its rendered output. FilmCraft owns the editorial timeline. This skill adds EffectCraft domain rules; `eqwec-engineering-core` remains the project rulebook and `eqwec-quality-control` remains the gate.

## Current state

EffectCraft is planned. The workspace currently has no `apps/`, `crates/`, `Cargo.toml`, or architecture documents. Treat every path, crate, schema, and workflow below as a design requirement, not as evidence of implementation. Before coding, verify names against `docs/architecture/SYSTEM-ARCHITECTURE.md` and `docs/architecture/DEPENDENCY-MAP.md` when they exist. If they are missing or contradict the code, report the drift; do not invent a structure or a passing command.

## Activation and non-triggers

Use this skill when a change affects:

- `apps/effectcraft` or an EffectCraft-specific crate.
- The effect graph schema, node API, port types, effect type/version, presets, or project file.
- Compositing, masks, transforms, blend modes, color operations, keyframes, or time mapping.
- GPU/CPU evaluation, shaders, device resources, render scheduling, or cache invalidation.
- Effect packages/plugins or the FilmCraft interchange contract.
- EffectCraft fixtures, regression tests, performance claims, security review, documentation, or release metadata.

Do not use it for:

- FilmCraft-only timeline editing, playback, or export; use the FilmCraft skill when it exists.
- PhotoCraft layer editing, VectorCraft path editing, AudioCraft DSP, or Hub packaging, unless an EffectCraft manifest or contract changes.
- Shared-crate changes with no EffectCraft consumer or behavior change. `eqwec-engineering-core` still governs the change.
- Generic Rust refactors, repository policy, or documentation that does not describe EffectCraft.

## Ownership

All paths are planned. Proposed crate names must be reconciled with the architecture documents; rename here if those documents choose different names.

| Path | State | Ownership | Rule |
|---|---|---|---|
| `apps/effectcraft` | Planned | EffectCraft | Product shell, project lifecycle, commands, UI orchestration, render job control. Workflow logic does not leak into `eqwec-ui`. |
| `crates/eqwec-graphics` | Planned, shared | Shared infrastructure | GPU device/resource management and rendering abstractions. Consume; do not fork or add product-only behavior. |
| `crates/eqwec-media` | Planned, shared | Shared infrastructure | Bounded decode/encode, probing, and frame sources. Consume. |
| `crates/eqwec-ui` | Planned, shared | Shared infrastructure | Design tokens and UI primitives only. No EffectCraft workflow logic. |
| `crates/effectcraft-graph` | Planned | EffectCraft | Graph model, typed ports, validation, serialization schema, migrations. |
| `crates/effectcraft-eval` | Planned | EffectCraft | Scheduler, temporal evaluation, backend dispatch, CPU reference path. |
| `crates/effectcraft-keyframes` | Planned | EffectCraft | Curves, interpolation, and time mapping. |
| `crates/effectcraft-cache` | Planned | EffectCraft | Cache keys, frame/intermediate caches, eviction. |
| `crates/effectcraft-compositing` | Planned | EffectCraft | Buffer contract, masks, transforms, blend modes, color operations, compositing kernels. |

Shared ownership rules:

- A shared crate has one owner and a public API. A product does not fork, vendor, or add product-only behavior to it.
- A change to `eqwec-graphics`, `eqwec-media`, or `eqwec-ui` that affects EffectCraft requires an impact assessment naming every affected product and a cross-product test. A shared crate must not depend on a product.
- Do not create a crate to match this list. Create it when it has an owner, API, tests, and a second consumer or a documented architectural reason.
- Every project path, asset reference, cache entry, and IPC message carries an explicit tenant/profile scope.

## Domain model

### Document model

- Project: tenant-scoped assets, compositions, presets, and settings.
- Composition: one graph, one timeline, one output configuration. The timeline defines a rational timebase, duration, frame range, and frame rate. The output configuration defines resolution, color space and transfer, alpha mode, and pixel format.
- Layer: a named subgraph with a transform, opacity, blend mode, and optional mask, producing one composited output. A layer stack is a graph, not a second evaluation model.
- Node: stable ID, effect type and semantic version, typed ports, typed parameters, enabled/bypass state, and optional mask inputs.
- Port types: `Image`, `Mask`, `Scalar`, `Color`, `Vector2`, `Transform`. The set is closed and versioned; adding a type is a schema change.
- Parameter: a static typed value or a keyframe curve. Declare unit, range, default, and whether it is animatable.
- Image buffer: nonzero width and height, pixel format, color space and transfer, alpha mode, row stride, origin, and ownership. The buffer type carries these; comments and naming conventions do not.
- Keyframe curve: sorted unique times on the composition timebase, per-segment interpolation (`hold`, `linear`, `cubic Bezier`), handles, and declared extrapolation.
- Timeline integration: composition time derives from a rational frame rate and frame index. Retime and source-time mapping are explicit and serialized.

### Graph invariants

- **Acyclic.** Reject a cycle at edit time and at load time with the node path. Validate iteratively with bounded depth, node count, edge count, and port count; do not rely on a recursion limit to stop a hang.
- **Typed.** Every required input is connected to a port of the same type. Optional inputs have explicit defaults. Fan-out is allowed; implicit coercion is not. Invalid connections fail with node ID, port ID, and expected type.
- **Resolution contract.** Each node declares `SameAsInput`, `Fixed`, or `Scale`. An edge that changes resolution without an explicit resample node is invalid. Masks may use a lower resolution only with a declared resampling rule. Composition output resolution is authoritative; no hidden scaling.
- **Color contract.** Each node declares whether it preserves the working space or converts. Every conversion is an explicit operation with source and destination space and transfer. No implicit sRGB assumption, no display-profile conversion during render, no double conversion.
- **Temporal contract.** Each node declares `TimeInvariant`, `PerFrame`, or `NeedsContext(n)`. The context window is part of evaluation and caching. Frame index maps to time exactly; do not accumulate floating-point frame steps.
- **Stable identity.** Node, port, parameter, asset, and composition IDs survive save/reopen. Deleted IDs are not silently rebound. Unknown effect types load as placeholders that preserve parameters and block render; they are never dropped or replaced by a default effect.
- **Deterministic order.** Topological evaluation uses a stable tie-break, normally node ID. Thread completion order must not change accumulation order or output.

### Compositing semantics

- Transforms are affine, with a declared order (anchor, scale, rotate, translate), pixel-center convention, and sampling filter. Resampling is explicit and part of the cache key.
- Blend modes composite premultiplied alpha in the declared working space. Each mode has a defined formula. An unsupported mode fails validation; it does not silently fall back to normal.
- Masks are single-channel coverage with declared invert and bounded feather. Mask resolution and resampling are explicit. Mask edges must be tested at alpha boundaries.
- Color operations use explicit matrices, transfer functions, and bounded LUT domains. Declare the out-of-gamut policy; do not clip silently.
- Effect categories (source, filter, generator, transform, composite, transition) are metadata, not a second type system.

### Cache keys

A cache key includes every semantic input: engine build and effect type/version; node parameters (static values, or sampled curve value plus sample time); recursive input keys; output resolution; working color space and transfer; pixel format and alpha mode; sampling filter; mask inputs; random seed; and any quality setting that changes output. Assets are keyed by content hash, never by path, modification time, or size alone. A backend-specific result is either keyed with the backend/device identity or stored in a separate per-backend cache; never mix them.

## Security and threat boundaries

Treat the project file, assets, presets, effect packages, shaders, and any expression text as untrusted.

- **Project files.** Parse with a bounded reader and a strict versioned schema. Cap file size, nodes, edges, ports, keyframes, string length, asset references, and nesting. Reject duplicate keys, inconsistent lengths, unknown enum values, and unsupported schema versions with a typed error. Do not log raw project content.
- **Expressions and scripts.** Prefer no general-purpose scripting. If expressions exist, run them in a restricted interpreter with instruction, time, and memory limits; no filesystem, network, environment, process, or FFI access; no host-language `eval`; deterministic inputs only. A failed expression fails the node closed.
- **Shaders.** Shader source is executable code. Load only built-in shaders or shaders from verified, installed effect packages. A project file may reference an effect type/version and parameters, not arbitrary shader source. If user shaders are ever supported, require a separate threat model, strict validation, resource limits, and GPU-process isolation; do not ship that path by accident.
- **Plugins and effect packages.** Verify signature and integrity before loading. Pin ABI and effect versions. Declare capabilities and resource limits in the manifest; deny filesystem, network, and process access by default. Isolate crashes and hangs; a plugin failure must not corrupt the project or the host process.
- **Paths.** Resolve asset references against the managed project root or a content-addressed store. Reject `..`, absolute paths, Windows drive/UNC prefixes, NUL bytes, and symlink escapes. Create temporary files privately and clean them on failure.
- **Resource exhaustion.** Enforce budgets for graph size, texture dimensions, intermediate bytes, keyframe count, mask size, LUT size, shader size, and render time. Fail closed with a diagnostic that identifies the node and limit.
- **Failure behavior.** An invalid graph blocks render/export; it must not produce a partial output file. Write output to a temporary file and publish atomically only after success. Preserve the last good project and autosave; a failed load or render must not overwrite recoverable user data. Cancellation releases GPU and CPU resources.

## Interfaces and dependencies

### Graph serialization

Use a versioned schema with a migration chain. Serialize stable IDs, typed values, units, timebase, ports, edges, effect type/version, keyframes, and output settings. Do not serialize pointers, GPU handles, cache contents, or transient evaluation state. Use canonical field order and deterministic ordering for round-trip tests. Reject unknown schema versions; migrate known versions explicitly. Unknown fields follow the declared strictness policy, not an accident of the serializer.

### Node API

Separate validation from evaluation. A node declares ports, parameter schema, temporal class, resolution policy, color policy, and determinism class. Evaluation receives typed inputs, sampled parameters, time/frame, output resolution, working color space, a cancellation token, and a resource budget; it returns a typed buffer or a typed error. Nodes do not read global state, wall-clock time, locale, or environment. Errors carry node and port context.

### Image buffer contract

Declare the internal working buffer format once and use it in every internal node. Convert at decode/source and encode/output boundaries. The buffer carries dimensions, pixel format, color space and transfer, alpha mode, stride, origin, and ownership. GPU textures are exposed through `eqwec-graphics`; the CPU path uses the same logical contract so parity tests compare semantics, not memory layout. No implicit copy, no hidden color conversion, no uninitialized padding.

### GPU/CPU backend parity

Every effect that claims backend parity needs a CPU reference implementation or a documented reference fixture, plus a parity test on each supported backend. Compare in the working color space with an alpha-aware metric; define tolerance per operation, not one blanket number. Reject NaN and Inf. A GPU path that cannot run reports a capability error and may fall back only through an explicit, tested path.

### Determinism boundary

- Deterministic by contract: graph validation, topological order, keyframe sampling at a given rational time, cache keys, serialization, CPU evaluation, and repeat CPU renders.
- Deterministic in practice on one device: GPU evaluation for a fixed engine build, backend, device, and driver. Do not promise it across drivers.
- Implementation-defined: cross-vendor GPU results, floating-point rounding, shader compiler optimizations, and texture-filtering precision. These require tolerance-based comparison, not byte equality.
- Never acceptable: unseeded randomness, wall-clock time, hash-map iteration order, uninitialized memory, locale-dependent formatting, or thread-race-dependent accumulation.

### Keyframe interpolation

Sample at an exact rational time. Define segment boundaries, handle clamping, overshoot policy, and extrapolation before the first and after the last keyframe. Interpolation must not depend on UI frame rate, evaluation order, or whether the frame was cached. Clamp only where the parameter schema declares a range.

### Cache invalidation contract

An upstream semantic change invalidates that node and its transitive dependents; unaffected branches keep their entries. A cache hit must be indistinguishable from recomputation. Eviction changes performance, never output. Persistent caches include engine/effect versions and content hashes and are discarded on incompatible upgrades. Detect cache corruption by checksum or validation and treat it as a miss, not as data.

### FilmCraft interchange

FilmCraft owns the editorial timeline document. EffectCraft owns the composition graph and its rendered result. Exchange only through a versioned, documented contract: an exported frame sequence or a referenced render with declared time mapping, resolution, color space, alpha, and frame range. Do not share mutable project files, undocumented sidecars, or global state. Unsupported effects or parameters are reported explicitly; never silently dropped. Round-trip tests preserve stable IDs, parameter values, time mapping, color space, alpha, and frame boundaries.

## Resources and performance

- **GPU memory.** Budget VRAM explicitly. Pool and reuse intermediate buffers, cap texture dimensions, and evict before allocation failure. Handle device loss and out-of-memory as recoverable errors with a diagnostic. Do not allocate one full-frame buffer per node without a measured reason.
- **Intermediate buffers.** Bound total intermediate bytes per frame. Prefer tiles or regions of interest when a node supports them; otherwise document the full-frame cost. Never let graph size or resolution create unbounded allocation.
- **Scheduling.** Evaluate in topological order with a bounded worker pool. Prioritize the visible frame; run cache warming and background renders at lower priority. Cancel stale work on seek, edit, or close, and release resources on cancellation.
- **Incremental evaluation.** Propagate a dirty set from changed parameters, assets, resolution, or color settings. Recompute only transitive dependents. Reuse time-invariant node results across frames. Verify branch reuse with instrumentation counters in tests, not by inspecting the screen.
- **Cache size and eviction.** Define memory and disk caps and an eviction policy. Frame caches are bounded by bytes and frame count. Eviction is safe at any time. Persistent cache indexes are bounded at startup.
- **Render time, startup, installer.** Measure first-frame latency, steady-state frame time, full-sequence time, cold start, and package size against a recorded baseline. Cache compiled shaders. Load effect packs lazily. Do not bundle large sample media in the installer. A performance claim without hardware, build profile, workload, and result is `BLOCKED`.

## Required workflows

Fixtures are planned. Use small deterministic assets: a 64×64 RGBA gradient, an alpha ramp, a checkerboard, color bars, a two-node and a five-node graph, and a 10-frame animation. Keep golden outputs versioned with the engine/effect version that produced them.

### 1. Build a graph

Create a composition, add a source, blur, transform, and blend node, connect typed ports, and validate. Success: validation passes; topological order is stable; the rendered fixture matches the golden within the declared tolerance; a wrong-type connection and a cycle are each rejected with node/port context.

### 2. Animate a parameter

Keyframe opacity and position with hold, linear, and Bezier segments over the fixture frame range. Sample at exact keyframe times and midpoints. Success: values match the expected curve within the declared numeric tolerance; boundary and extrapolation behavior matches the contract; save/reopen preserves keyframe times, handles, and interpolation.

### 3. Render a frame sequence

Render the fixture range at a fixed timebase and output configuration. Success: every frame is produced exactly once with the expected name and no gaps; output files are published atomically; a repeated render on the same backend matches the first (bit-exact for CPU, within tolerance for GPU); cancellation leaves no final partial files.

### 4. Save and reopen

Save, close, reopen, and compare. Success: semantic equality of compositions, graph topology, IDs, parameters, curves, masks, output settings, and asset references; canonical serialization is byte-stable if the format promises it; unknown effect types remain as placeholders with parameters intact.

### 5. Verify cache invalidation

Render, record cache counters, change one upstream parameter or asset, and render again. Success: the changed node and its dependents recompute; sibling branches hit cache; changing resolution or color space invalidates all affected entries; reverting the change does not serve a stale result. Add a test that fails if any key component (resolution, color space, filter, seed, asset hash) is omitted.

### 6. Compare GPU and CPU output

Render the same graph and fixture on both backends. Success: no NaN or Inf; max absolute error and a high percentile stay within the per-operation tolerance in the working color space; alpha edges and mask boundaries are included; a failure names the node, frame, backend, device, and driver. Do not widen tolerance to make a real defect pass.

### 7. Round-trip with FilmCraft

Export an EffectCraft composition through the documented interchange, import it into FilmCraft, edit the surrounding timeline, and return it. Success: IDs, parameters, time mapping, resolution, color space, alpha, and frame boundaries survive; unsupported features are reported; the rendered result matches the reference within tolerance; no undocumented file is required.

### Required edge cases

- **Cycle.** Adding an edge that closes a cycle is rejected at edit time; a crafted cyclic project fails to load with a typed error and does not hang or crash; other compositions remain usable.
- **Missing asset.** A moved or deleted asset produces a recoverable placeholder and a diagnostic; render/export is blocked with an actionable error; saving does not erase the original reference.
- **Resolution change.** Changing composition or node resolution invalidates affected caches, respects fixed versus relative resolution policies, and stays within memory budgets.
- **Color-space change.** Changing working or output color space invalidates caches and applies exactly one explicit conversion; no stale or double-converted frame is served.

## Failure modes and debugging

Common symptoms and first checks:

| Symptom | Likely cause | Check |
|---|---|---|
| Black or transparent frame | unconnected required input, failed asset, NaN, wrong alpha | validate graph, asset state, buffer stats |
| Wrong gamma or hue | missing or double color conversion, display profile used for render | input/output color spaces, conversion nodes, cache key |
| Stale frame after edit | incomplete cache key, broken dirty propagation | cache counters, key contents, asset hash |
| Preview/export mismatch | different resolution, color space, backend, or quality setting | both render configurations |
| Hang | undetected cycle, unbounded recursion, shader compile wait, lock | graph validation, shader log, thread dump |
| OOM or device loss | unbounded intermediates, no pooling, driver defect | memory budget, buffer count, driver version |
| Nondeterministic output | unseeded random, wall clock, unordered iteration, race | seed, ordering, thread count, repeat run |
| Shader compile failure | unsupported feature, backend syntax, driver bug | capability report, shader log, second backend |

Debugging procedure:

1. Reproduce with the smallest fixture and record project hash, engine build, backend, device, driver, OS, resolution, color space, frame index, and effect versions.
2. Validate the graph and print the topological order and cache keys for the failing node.
3. Bypass nodes to localize the first incorrect buffer; compare dimensions, format, alpha mode, and color space at each edge.
4. Compare against the CPU reference or a known golden frame.
5. Inspect cache hit/miss counters and asset content hashes before suspecting the renderer.
6. Check the bounded diagnostic log for shader compilation, device loss, and resource-limit errors.
7. Reproduce on a second backend or device to separate logic defects from driver defects.
8. Add the regression test before the fix; keep the fixture and the failing configuration.

Regression tests must cover: cycle rejection and cyclic-file load, type mismatch, missing asset, resolution and color-space invalidation, cache-key completeness, keyframe boundaries and extrapolation, alpha compositing and mask edges, GPU/CPU parity, repeat-render determinism, save/reopen, FilmCraft round-trip, fuzzed graph/project parsing, and resource-limit failures.

### Cross-platform caveats

- Detect GPU features at startup: compute support, float render targets, max texture size, linear filtering of float textures, timestamp queries, and required shader stages. Fall back only through a tested path; report unsupported capabilities.
- Shader compilation differs by backend and driver. Validate shaders, cache compiled artifacts by driver/backend, and surface compile logs without leaking project data.
- Floating-point results differ with FMA, precision, denormal handling, and fast-math. Do not enable fast-math for the reference path; compare with tolerance and record it.
- Driver bugs exist. Isolate workarounds behind a versioned capability table with a test and a removal condition.
- Color management differs by platform. Render in explicit project color spaces; never use the display profile as the render space.
- Filesystem case sensitivity, path separators, and Unicode normalization affect asset lookup. Use content hashes and canonical relative paths.
- Test on every supported OS/architecture and on at least one GPU family per supported backend. Mark untested combinations `BLOCKED`.

## Release and documentation

Update this skill, the architecture documents, the project-format documentation, the effect-version changelog, and the compatibility notes when changing the graph schema, node API, port types, effect type/version, keyframe semantics, cache-key semantics, color/resolution contract, or FilmCraft interchange. A schema change needs a migration and a round-trip test. An effect behavior change needs an effect version bump and preset compatibility assessment. Release notes must not describe planned or prototype behavior as shipped.

Handoff:

- `eqwec-engineering-core` for architecture, ownership, shared-crate policy, security baseline, Git, versioning, and documentation lifecycle conflicts.
- `eqwec-quality-control` for pre-change, implementation, verification, performance, merge, and release gates, including `PASS`/`FAIL`/`BLOCKED` evidence.
- The FilmCraft skill for timeline document ownership and editorial behavior; the PhotoCraft and VectorCraft skills for shared graphics/media changes that affect them.
- If this skill conflicts with core security, versioning, or Git rules, core wins and this skill is defective.

## Red flags

- A cache hit after an upstream parameter, asset, resolution, color-space, filter, or seed change.
- GPU/CPU divergence hidden by a loose tolerance or a tolerance widened without a recorded reason.
- Unbounded intermediate buffers, texture sizes, graph size, keyframe count, or cache growth.
- A graph cycle that hangs validation or render instead of returning a typed error.
- A render called reproducible when it uses unseeded randomness, wall-clock time, or unordered accumulation.
- Cache keys missing color space, resolution, pixel format, filter, effect version, or asset content hash.
- A project file that can load arbitrary shader source, execute a script, or reference a path outside the project root.
- Silent substitution of a missing effect, unsupported blend mode, or failed asset with a default.
- Preview and export using different resolution, color space, backend, or quality settings.
- A partial output file published as success, or a failed render overwriting the last good project.
- A performance or determinism claim without hardware, driver, workload, build profile, and recorded result.
