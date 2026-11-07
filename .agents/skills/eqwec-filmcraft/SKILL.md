---
name: eqwec-filmcraft
description: Use when implementing, reviewing, or testing FilmCraft non-linear editing, timeline and project format, playback, A/V sync, media import, proxy/cache, render graph, render queue, or export, or when changing shared eqwec-media, eqwec-graphics, eqwec-storage, or eqwec-ui behavior that affects FilmCraft. Do not use for other EQwec products, repository policy alone, or changes with no FilmCraft path or media/render contract impact.
---

# EQwec FilmCraft

FilmCraft is the non-linear video editor: import, arrange, trim, composite, play back, and render video with synchronized audio.

**Status:** FilmCraft is planned. The apps and product crates named below do not exist yet; the Phase 2 workspace exists. This skill states required design and verification, not implemented capability. Verify every path against `docs/architecture/SYSTEM-ARCHITECTURE.md` and `docs/architecture/DEPENDENCY-MAP.md`; if they disagree with this skill, stop and report the drift.

## Scope and activation

### Owned paths and crates

All paths are planned until the workspace exists.

| Path | Role | State |
|---|---|---|
| `apps/filmcraft` | Product entry point, composition root, editor UI workflow wiring | Planned |
| `crates/eqwec-film-timeline` | Project, sequence, track, clip model; edit commands; undo/redo; time math | Planned |
| `crates/eqwec-film-sources` | Media source registry, probing, fingerprints, relink and offline state | Planned |
| `crates/eqwec-film-render` | Render graph, compositing, frame evaluation, deterministic mode | Planned |
| `crates/eqwec-film-proxy-cache` | Proxy generation, render cache, cache keys and eviction | Planned |
| `crates/eqwec-film-export` | Export presets, muxing contract, render queue, resume and recovery | Planned |
| `crates/eqwec-media` | Shared bounded decode/encode primitives; FilmCraft is a consumer | Planned shared |
| `crates/eqwec-graphics` | Shared GPU device, resource, and rendering abstractions | Planned shared |
| `crates/eqwec-storage` | Atomic project persistence and scoped storage | Planned shared |
| `crates/eqwec-ui` | Shared tokens and UI primitives; no timeline or edit logic | Planned shared |

`eqwec-core`, `eqwec-security`, `eqwec-logging`, `eqwec-config`, and `eqwec-ipc` are dependencies when needed, not FilmCraft-owned.

### Shared ownership rules

- FilmCraft owns film semantics: timeline rules, project schema, edit commands, compositor policy, export presets, and A/V sync behavior.
- Shared crates own generic mechanisms. Do not put clip, sequence, or transition types into `eqwec-media` or `eqwec-graphics`.
- A shared-crate change requires an impact assessment naming every consumer (AudioCraft, EffectCraft, PhotoCraft, VectorCraft, Hub) and contract tests for each affected consumer.
- `eqwec-storage` provides atomic, tenant/profile-scoped persistence; it must not encode the FilmCraft project schema. FilmCraft owns the schema and migrations.
- `eqwec-ui` provides primitives; timeline interaction, playback control, and edit commands stay in FilmCraft.
- Every project, cache, proxy, and export path carries an explicit tenant or profile scope. No global default for user data.
- Do not create a crate to match this table unless it has an owner, public API, tests, and a second consumer or documented architectural reason.

### Triggers

Load this skill when a change touches:

- Any FilmCraft path, project format, schema version, or migration.
- Media import, probe, decode, encode, mux, demux, or relink behavior.
- Timeline edits: place, trim, split, ripple, slip, slide, roll, transitions, keyframes.
- Playback, preview quality, dropped frames, audio clock, or A/V sync.
- Render graph, compositing, color transform, render cache, proxy generation, or render queue.
- Export presets, container/codec settings, output validation, or render recovery.
- Hardware decode/encode, GPU shader paths, codec availability, or licensing for the film pipeline.
- FilmCraft fixtures, end-to-end tests, performance baselines, or release notes.

### Non-triggers

- PhotoCraft image editing, VectorCraft path editing, AudioCraft DAW work, or EffectCraft compositing that does not change a FilmCraft or shared media/render contract.
- Repository policy, Git workflow, or CI changes that do not affect FilmCraft paths, formats, or contracts; use `eqwec-engineering-core`.
- A purely cosmetic `eqwec-ui` change that does not alter FilmCraft workflow, timing, data, or rendering.
- Unrelated repositories and generic Rust questions.

## Domain model and invariants

### Model

Canonical chain: **project → sequence → tracks → clips → media sources → effects**.

- **Project** — top-level document. Owns sequences, the media source registry, export presets, and project settings. Carries schema version and stable project ID.
- **Sequence** — editing timeline with exact rational frame rate, frame size, color configuration, audio sample rate, channel layout, and start timecode. Owns tracks and sequence-level effects.
- **Track** — ordered video, audio, or subtitle lane with enabled, locked, muted/solo, and compositing or gain properties. Track order defines compositing and mix order.
- **Clip** — placement instance referencing a media source. Stores source in/out, timeline start, duration, speed, linked-group ID, effects, transition participation, and keyframes. A clip never owns media bytes.
- **Media source** — stable source ID, content fingerprint, probed stream metadata, color metadata, rotation, audio layout, original location, optional managed copy, and optional proxy reference.
- **Effect** — typed filter or compositor node with validated parameters and keyframed values. Built-in, signed third-party, and untrusted effects are distinct trust classes.
- **Render job / export preset** — project-adjacent, versioned records. A job snapshots an immutable project revision, range, and preset; it never renders live mutable state.

### Time and frame invariants

- Store all edit times as exact rational values or integer frame/sample counts. Do not use floating point for edit decisions, comparisons, or serialization.
- A sequence frame rate is exact rational (for example `24000/1001`, `25/1`, `30000/1001`). Frame `N` covers the half-open interval `[N/rate, (N+1)/rate)`.
- Timecode is a display format, never the storage key. Drop-frame applies only to valid rates such as 29.97 and 59.94; non-drop rates such as 23.976 stay non-drop. Record the mode explicitly.
- A media source keeps its own timebase. Convert to sequence time only at an explicit boundary with a documented rounding and tie-break policy. A variable-frame-rate source is not silently treated as constant frame rate.
- Audio uses the project sample clock. Edit boundaries, fades, and keyframes that affect audio are quantized to samples. Resample once at the output boundary, not per clip.
- A/V sync is defined on the common rational timeline and measured at output. Linked audio and video share edit operations; per-clip clock drift is a defect.
- Keyframes use rational time in the parameter's declared time domain, are sorted, clamp at boundaries, and declare interpolation. Duplicate times are rejected or resolved by an explicit rule.

### Edit invariants

- Clip intervals are half-open `[start, end)` with positive duration. `end = start + duration`. Source in/out must lie within available media.
- Clips on one track do not overlap except inside a declared transition region. Transition handles must exist in the source; a transition that exceeds available handles fails closed.
- Split preserves source continuity: the first clip's source out equals the second clip's source in at the split point, and timeline positions remain adjacent.
- Trim adjusts source in/out and timeline boundary according to the declared mode. Ripple shifts downstream clips on the affected track and linked tracks by the exact removed duration; unrelated tracks do not move.
- Linked clips move, trim, split, and ripple together unless the user explicitly unlinks them. Unlink is an undoable edit.
- Proxy media is derived. It never replaces the source identity, never changes edit decisions, and never becomes canonical project state.
- Render output for frame `N` must be evaluated from timeline frame `N`, not from an accumulated counter or a wall-clock position.

### Canonical project state and undo/redo

- One canonical in-memory state per open project. UI, render, and background jobs read snapshots; they do not mutate state directly.
- Every mutation is a typed command or transaction. A transaction either commits fully or leaves the previous state unchanged. No partial edit on error or cancellation.
- Canonical serialization is normalized: stable IDs, deterministic ordering, no UI-only state, no derived caches, no absolute paths where a scoped relative path is possible. The same state serializes to the same bytes except for documented volatile metadata.
- Project files are schema-versioned and strictly parsed. Unknown fields are rejected unless a documented forward-compatible extension exists. Migrations are explicit, tested, and never destructive without a backup.
- Save is atomic: write a restricted temporary file, flush, rename over the target, and retain a recoverable previous version. A crash must not leave a truncated project presented as valid.
- Undo restores the exact prior canonical state, including IDs and ordering. Redo is cleared by a new edit. History is bounded. External media changes and proxy deletion are not undoable edits.
- Saving does not silently discard undo history unless the product documents that rule. Crash recovery is separate from undo.

## Threat boundaries and security

Treat every imported file as hostile: containers, codecs, metadata, subtitles, LUTs, fonts, project files, proxy manifests, and effect packages.

- **Bounded decode.** Enforce documented caps on file size, stream count, resolution, frame count, bitrate, decode queue depth, memory, and wall-clock decode time. Reject beyond the cap; never truncate silently or reserve attacker-controlled sizes.
- **Malformed timestamps.** Reject NaN, infinity, negative values where invalid, overflow, non-monotonic DTS beyond the documented reorder window, and edit lists or durations that escape the declared media. Any clamping must be recorded and tested.
- **Path safety.** Resolve media and relink paths against declared project and media roots. Reject traversal, absolute-path injection, Windows drive and UNC prefixes, NUL bytes, and symlink escapes. Detect case-insensitive and Unicode-normalization collisions where the platform requires it.
- **Project files.** Bound size and nesting, validate schema before applying, and never execute embedded scripts, shell commands, or plugin code on open. A corrupt or inconsistent project fails closed.
- **Corrupt project recovery.** Preserve the last recoverable version and quarantine the corrupt file. Never overwrite it with an empty project or delete it automatically. Present a bounded diagnostic and explicit recovery choices.
- **Plugins and effects.** Effects are code. Use a trust class and capability manifest. Third-party effects run out of process or sandboxed where feasible, with explicit install consent and no implicit network or arbitrary filesystem access. A plugin crash must not corrupt the project or take down the editor.
- **GPU shader input.** Treat shader source, LUTs, and GPU buffers as untrusted. Validate shader compilation, bound uniforms, textures, and workgroup sizes, reject out-of-bounds access, and handle device loss. Do not expose other applications' surfaces or read back unrelated memory.
- **No silent upload.** Never upload media, proxies, thumbnails, waveforms, project contents, paths, or derived metadata without explicit, informed opt-in. The media pipeline is offline by default. Plugin network capability is separately gated.
- **Source protection.** Never modify or delete source media during import, proxy generation, caching, relink, or export. Export writes to a new path through an atomic finalize step unless the user explicitly confirms overwrite.
- **Diagnostics.** Log bounded IDs, sizes, error classes, and hashes, not raw media payloads, full personal paths, or project contents.

## Interfaces and dependencies

### Decode and encode boundary

The media boundary returns typed decoded frames and audio blocks, never bare byte buffers with implicit meaning. Each frame declares pixel format, bit depth, primaries, transfer function, matrix, range, alpha mode, stride, rotation, color metadata, rational timestamp, and duration. Each audio block declares sample format, sample rate, channel layout, frame count, and timestamp.

- Define seek semantics: keyframe-accurate versus frame-accurate, with the cost and error class explicit.
- Decoders are owned per worker or stream; do not share one decoder across threads unless the contract documents it.
- Queues are bounded. Full means backpressure or a recorded preview drop, never unbounded memory growth.
- Cancellation and shutdown release decoder, GPU, and file resources and never commit a partial frame or file.
- Errors are typed: unsupported feature, corrupt media, resource limit, license unavailable, device lost, cancelled.

### Codec licensing

- Maintain a codec and container inventory with license, patent obligations, build flags, distribution scope, and hardware dependency.
- Do not bundle an encoder or decoder whose license is unreviewed. GPL components must not contaminate a non-GPL product build.
- Export presets expose only encoders that are actually licensed and available in that build. Do not silently substitute another codec, quality, or color path when one is unavailable.
- Hardware vendor SDKs and system codecs have separate terms. Record them before enabling a platform path.
- A licensing change is a release-blocking dependency change, not a build detail.

### Color management

- Declare input, working, display, and output color spaces. Do not assume sRGB or Rec.709.
- Carry primaries, transfer function, matrix, range, and HDR metadata through decode, render, and mux. Avoid double transforms and missing tags.
- Define alpha policy: straight or premultiplied, and where conversion happens.
- LUTs are untrusted files: bound size, validate domain, and test interpolation.
- CPU and GPU color paths must agree within a stated tolerance; a mismatch is a defect, not a rounding preference.

### Audio sample clock

- Playback has one clock master. Define whether video follows audio or audio follows video and how device drift is compensated.
- The real-time audio callback performs no allocation, blocking lock, file I/O, or unbounded work.
- Resampling happens once at a declared boundary. Compensate resampler latency and document priming and padding.
- Channel layout and sample format conversions are explicit; a mono/stereo/5.1 mismatch is not silently upmixed.

### Render graph, cache, and proxy

- The render graph is a DAG of sources, transforms, composites, and effects with explicit color and alpha. Evaluation is deterministic for a given graph, inputs, and renderer version.
- A frame request key includes sequence revision, frame index, range, resolution, bit depth, color configuration, effect parameters and keyframes, proxy policy, renderer/backend version, and every other output-affecting input.
- The cache key is versioned and collision-resistant. Missing a key input causes stale frames; including unstable data causes misses. Negative results are cached with bounds.
- Cache growth, proxy storage, and temporary render files have quotas, eviction, and cleanup. Eviction never removes source media.
- A proxy records source identity, source fingerprint, generation settings, tool version, and color treatment. Changing the source or generation settings invalidates it. Missing proxies fall back explicitly and visibly.

### Export contract

- Range semantics are half-open `[in, out)`. Frame count and first PTS are derived exactly from the sequence timebase.
- Output declares container, codec, profile, resolution, frame rate, bitrate or quality, color tags, audio codec, sample rate, channel layout, and priming/padding.
- Write to a temporary path and finalize atomically. A failed, cancelled, or interrupted export must not leave a file presented as complete.
- Validate the muxed output: decode it independently, check frame count, PTS monotonicity, duration, audio sample count, and metadata.
- Resume uses a checkpoint tied to the immutable job snapshot. Restarting must not duplicate frames, append to a stale file, or report success for a partial result.
- Record whether the output is deterministic or hardware-dependent, and which backend produced it.

### Hardware versus software

- Probe hardware decode and encode capabilities at runtime. Unsupported profiles fall back explicitly and record the fallback; they do not silently change output.
- Hardware paths may differ in color conversion, rounding, bit depth, and B-frame behavior. Do not claim bit-identical output without a test that proves it.
- Deterministic mode uses the software reference path, or a hardware path proven reproducible on the declared matrix. If reproducibility is not proven, label the output hardware-dependent.
- A deterministic claim names the platform, driver, backend, build, and fixture set it covers.

## Resource concerns

Define a budget and a measurement for each concern; an unmeasured claim is not evidence.

| Concern | Required measure | Failure behavior |
|---|---|---|
| Frame processing latency | p95/p99 per preview frame at target rates | Drop preview frame, record counter; never block UI or audio |
| Export throughput | Output frames per second and realtime factor against baseline | Investigate regression; keep queues bounded |
| Playback stability | Dropped-frame rate, audio underruns, A/V drift | Degrade preview or proxy explicitly; never hide drift |
| Cache efficiency | Hit rate, eviction rate, memory and disk tiers | Bound growth; stale entries invalidate by key |
| Memory | Peak RSS, frame pool, queue depth | Backpressure or fail with a typed resource error |
| GPU memory | VRAM budget, texture pool, device-loss events | Evict, reduce preview quality, or fail export safely |
| Disk I/O | Sequential playback read, scrub random read, write throughput | Preflight free space; fail export without partial success |
| Proxy storage | Quota, cleanup policy, orphan detection | Evict proxies only; never source media |
| Project load time | Time versus clip/keyframe/media count; lazy probe | Report budget breach; do not block UI silently |
| Installer size | Codec, GPU, and model payload; optional packs | Measure per platform; do not bundle unlicensed payloads |
| Codec licensing | Cost and obligations per shipped codec | Block release until resolved |

Playback may drop preview frames; export must not drop frames. Both paths must have explicit backpressure.

## Required end-to-end workflows

Fixtures are deterministic, small enough for CI, contain no secrets, and carry a manifest with hashes, expected frame numbers, sample offsets, and color tags.

- `FX-CORE` — 10 s constant-rate 24 fps clip with burned-in frame number and timecode; 1 kHz bursts at known frames; stereo 48 kHz.
- `FX-RATES` — 25 fps and 30000/1001 clips, 44.1 kHz and 48 kHz audio, and a one-hour low-resolution timeline for drift.
- `FX-IMAGE` — 100-frame image sequence with known frame content.
- `FX-COLOR` — Rec.709, Display P3, and HDR PQ/HLG tagged clips plus a bounded LUT.
- `FX-HOSTILE` — truncated container, absurd dimensions and stream counts, negative/NaN timestamps, path-traversal project, unsupported codec, and missing media.
- `FX-LARGE` — 500 clips and 2,000 keyframes for load, cache, and responsiveness measurement.

For each workflow, record the project revision, preset, backend, platform, and fixture hashes.

1. **Import media.** Import `FX-CORE`; probe and fingerprint it. Pass: duration, rate, dimensions, pixel format, color tags, sample rate, and channels match the manifest; one source per content hash; source bytes are unchanged; `FX-HOSTILE` is rejected with a typed error and no partial source; no network access occurs.
2. **Place clips.** Place video and linked audio at frame 100 with exact source in/out. Pass: timeline start, duration, and source range match; no overlap; linked group shares edits; invariants hold.
3. **Trim, split, ripple.** Trim head and tail, split at a frame, ripple-delete a middle range. Pass: source continuity holds at the split; downstream and linked clips shift by the exact removed duration; unrelated tracks do not move; undo restores the exact prior state hash.
4. **Transition and keyframe.** Overlap two clips with sufficient handles, add a cross-dissolve, keyframe opacity or transform. Pass: the rendered midpoint matches the expected blend within the declared tolerance; keyframed values at exact frames match the interpolation rule; a transition beyond available handles is rejected; undo is exact.
5. **Save and reopen.** Save, close, reopen. Pass: atomic write and previous-version backup exist; canonical serialization round-trips; IDs and ordering are stable; no source media is modified; the documented undo policy holds; a deliberately corrupt project fails closed and the last recoverable version remains available.
6. **Render a playable video.** Export a known range. Pass: frame count equals `out - in`; the burned-in number in rendered frame `N` equals timeline frame `N`; PTS mapping is exact; the audio impulse lands at the expected sample with drift at most one sample; an independent decoder plays the file; color and audio tags match the preset; no partial file is presented as complete.
7. **Proxy workflow.** Generate a proxy, edit with proxy playback enabled, then render from source. Pass: proxy use is recorded; final output matches the source-based render within the declared tolerance; source bytes are unchanged; changing the source invalidates the proxy; deleting the proxy does not touch the source; a missing proxy falls back explicitly.
8. **Background render.** Queue a render while editing. Pass: the job uses an immutable snapshot; UI frame latency stays within budget; progress and cancel work; app restart resumes or safely restarts; no partial final file; revision and preset are recorded.
9. **Interrupted render recovery.** Terminate the process at a checkpoint, then restart. Pass: no partial file is marked complete; temporary files are quarantined or cleaned; resume or restart produces a valid output exactly once; no duplicated frames or side effects.
10. **Missing media.** Move a source file, reopen the project. Pass: clips show offline; no silent substitution occurs; relink uses fingerprint or explicit user choice; a wrong file is rejected or clearly warned; export blocks unless an explicit offline policy is chosen; saving preserves recoverable state.
11. **Mixed frame rates and sample rates.** Build a one-hour timeline from `FX-RATES` and render start, middle, and end ranges. Pass: rational frame mapping has no accumulated error; audio is resampled once; sample offsets match the manifest; A/V drift stays within one sample; frame numbers at all three ranges are exact.

## Failure modes and debugging

| Symptom | Likely cause | First check |
|---|---|---|
| Frame `N` shows `N±1` or timecode disagrees | Frame-index/timecode or out-point confusion, drop-frame misuse | Half-open intervals, timecode mode, source-rate conversion |
| Audio drifts progressively | Per-clip resampling, device clock mismatch, uncompensated latency | Sample offset at start and end, resampler latency, clock master |
| Stutter or dropped preview frames | Decode, GPU, cache, or queue saturation | Frame-time percentiles, queue depth, cache hit rate, proxy use |
| Color shift or crushed levels | Missing or double transform, wrong range, alpha error | Metadata chain, transform count, CPU/GPU parity |
| Hardware path fails or differs | Unsupported profile, driver bug, fallback | Capability probe, fallback record, software reference |
| Stale frame after an edit | Cache key missing an input | Key composition and invalidation |
| Project corrupt or truncated after crash | Non-atomic save, missing flush | Temp/rename, backup, journal, recovery report |
| Partial export shown as success | Finalize contract violated | Temp path, rename, status transition |
| Missing media silently replaced | Relink or path fallback bug | Fingerprint check and explicit relink |
| Proxy deletion loses source | Source/proxy path confusion | Path ownership and deletion guard |
| Plugin crash kills the editor | In-process untrusted effect | Trust class, sandbox, out-of-process isolation |
| OOM or GPU OOM | Unbounded queue, cache, or texture pool | Caps, eviction, backpressure |
| Disk full mid-export | No preflight or unsafe write | Free-space check, typed failure, cleanup |
| Relink escapes media roots | Path traversal or symlink escape | Canonical path validation |

### Debugging procedure

1. Reproduce on the smallest fixture and record project revision, preset, platform, backend, driver, and codec path.
2. Classify the boundary: probe/import, decode, timeline model, render graph, color, audio clock, mux/export, or storage/recovery.
3. Compare against the software reference path. Check frame hashes at exact frame indices, PTS/DTS, sample offsets, and waveform correlation.
4. Disable one variable at a time: effects, hardware decode/encode, proxy, cache, color transform, background job.
5. Inspect bounded logs and counters. Do not dump media contents, full paths, or project payloads.
6. Write the regression test first and confirm it fails on the defect, then fix and confirm it passes.
7. Mark any platform check that cannot run as `BLOCKED`; never infer a pass from another platform.

### Regression tests

- Property tests for rational time arithmetic, interval invariants, edit sequences, and time conversion monotonicity.
- Round-trip serialization, schema migration, and undo/redo state-hash equality.
- Golden frame and audio checks for every fixture. CPU deterministic output is exact; hardware output uses a declared tolerance and is not labeled deterministic.
- Adversarial and fuzz tests for containers, project files, LUTs, subtitles, timestamps, and path handling.
- Kill/interrupt, disk-full, missing-media, relink, and corrupt-project recovery tests.
- Long-duration mixed-rate drift tests and playback/export performance baselines.
- Cross-platform tests recorded per platform; unavailable platforms are `BLOCKED`, not skipped silently.

## Cross-platform caveats

- **Hardware decode/encode:** VideoToolbox on macOS, Media Foundation/DXVA on Windows, VAAPI or vendor paths on Linux. Availability, profiles, and output differ by driver and device. Always keep a software path and record fallback.
- **Codecs:** system codecs, bundled libraries, and hardware encoders differ per platform and build. Verify license and availability before exposing a preset.
- **Color:** ICC profiles, Display P3, HDR PQ/HLG, and OS color management change display and export behavior. Verify tags and transforms on each platform.
- **Paths:** separators, case sensitivity, Unicode normalization, long paths, symlinks, network volumes, and atomic rename semantics differ. Test relink and save on each filesystem class.
- **Audio:** CoreAudio, WASAPI shared/exclusive, PipeWire, and ALSA have different clock and device-change behavior. Bluetooth latency and sample-rate changes must not break sync.
- **GPU:** shader compilers, precision, VRAM limits, and device-loss behavior vary. Test deterministic claims per driver.
- **Sandboxing:** macOS TCC, packaged Windows apps, and Linux sandboxes restrict file and device access. Request the minimum permission and fail with a clear diagnostic.

## Release and documentation triggers

Update the affected documents, or record why no change is needed, when:

- A workflow above changes status or acceptance criteria.
- The project schema, migration, or compatibility statement changes.
- A codec, container, hardware path, or license is added or removed.
- The supported platform matrix or performance baseline changes.
- Proxy, cache, render queue, or export behavior changes.
- User-visible recovery, relink, or export semantics change.

Required artifacts before a FilmCraft release claim:

- Accurate `README.md` status; never describe planned capability as shipped.
- `AGENTS.md` skill map and this skill updated.
- Architecture, dependency map, ADRs, and threat model updated where affected.
- Codec license inventory and supported platform matrix current.
- User documentation for import, proxy, export, recovery, and missing media.
- Changelog, independent version, and release notes that distinguish deterministic from hardware-dependent output.
- End-to-end workflow evidence for the claimed platforms, with unrun checks marked `BLOCKED`.

## Handoff

- Hand off to `eqwec-engineering-core` for architecture, shared-crate ownership, security policy, versioning, Git, or cross-product contract conflicts.
- Hand off to `eqwec-quality-control` before implementation and before any completion, performance, merge, or release claim. Provide the gate matrix and evidence.
- If this skill conflicts with `eqwec-engineering-core`, the core skill wins; record the conflict as a skill defect.

## Red flags

- Frame-index, timecode, and drop-frame confusion; off-by-one at half-open out points.
- A/V drift hidden by per-clip resampling, resampler latency, or an unstated clock master.
- A nondeterministic GPU render presented as deterministic.
- Deleting, moving, or overwriting source media during proxy creation or cache cleanup.
- Unbounded render cache, proxy store, decode queue, or frame pool.
- Codec licensing or patent obligations ignored; unlicensed encoder exposed in a preset.
- Floating-point edit times or accumulated frame counters.
- Cache key missing color, backend, preset, proxy, or keyframe inputs.
- Partial export presented as a successful file.
- Corrupt project overwritten with an empty state instead of preserving the last recoverable version.
- Relink or import accepting traversal, absolute injection, or symlink escape.
- Plugin or effect network access, telemetry, or media upload without explicit consent.
- Hardware fallback that silently changes output, color, or quality.
- Frame accuracy or A/V sync claimed without a burned-in-frame and sample-offset test.
