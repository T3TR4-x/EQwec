---
name: eqwec-audiocraft
description: Use when working on AudioCraft or any audio path in the EQwec repository, including the DAW app, audio engine and graph, MIDI sequencing, timeline, plugin hosting, recording, mixing, automation, rendering, project format, or audio-specific shared-crate changes. Trigger on real-time callback, latency, xrun, sample-rate, device, plugin-crash, or render/export work. Do not use for unrelated products, generic UI-only changes, or non-audio shared-crate work with no AudioCraft consumer.
---

# EQwec AudioCraft

Product skill for AudioCraft, the EQwec digital audio workstation (DAW). It adds audio-domain constraints to `eqwec-engineering-core`; it never weakens core security, Git, versioning, or quality rules. If this skill conflicts with core, core wins and the conflict is a skill defect.

**Status: planning only.** No AudioCraft workspace, crate, backend, plugin host, or renderer exists in this repository. All paths and crates below are planned. Every statement is a requirement for the planned product, not a description of current behavior. Do not describe planned behavior as implemented. Discover build, test, and render commands from `AGENTS.md`, CI, or Cargo metadata when the workspace exists; never invent a command or record an unexecuted check as `PASS`.

## 1. Ownership and activation

| Path or crate | Ownership | Status |
|---|---|---|
| `apps/audiocraft` | App shell, command routing, session wiring, audio UI workflows, packaging entry point | Planned |
| `crates/eqwec-audio-engine` | Device/backend abstraction, stream lifecycle, callback boundary, transport clock, recording capture | Planned |
| `crates/eqwec-audio-graph` | Graph model, topological processing, buses, summing, latency compensation, real-time scheduling | Planned |
| `crates/eqwec-midi` | MIDI parsing, event model, scheduling, device adapters, piano-roll edit model | Planned |
| `crates/eqwec-timeline` | Tempo map, musical time, clips, patterns, arrangement, automation lanes | Planned |
| `crates/eqwec-plugins` | Plugin discovery and scanning, instance lifecycle, state, sandbox/host process, crash containment | Planned |
| `crates/eqwec-audio-render` | Offline render/export, tails, dither, peak/loudness measurement, render contract | Planned |
| `crates/eqwec-media` | Shared bounded decode/encode and media primitives | Planned shared |
| `crates/eqwec-storage` | Shared project persistence, atomic writes, recovery | Planned shared |
| `crates/eqwec-ui` | Shared design tokens and UI primitives; no DAW workflow logic | Planned shared |

Crate names are proposed; confirm them against `docs/architecture/SYSTEM-ARCHITECTURE.md` and `DEPENDENCY-MAP.md` when those documents exist. Do not create an empty crate to match this table.

Ownership rules:

- AudioCraft owns the app and audio-specific crates. Shared crates keep their own owners and public contracts. A product may depend on a shared crate; a shared crate must not depend on a product.
- A change to shared decode/encode, project persistence, or UI primitives that affects AudioCraft requires an impact assessment and a cross-product test.
- Keep DAW workflow, transport, graph, and plugin policy out of `eqwec-ui`.
- Scope project, cache, recording, and temporary paths to a tenant or profile. Never rely on a global default for tenant-owned data.
- Plugin SDKs, codecs, and bundled content are dependencies with recorded license, maintenance status, and replacement cost.

Activation triggers:

- Any change under `apps/audiocraft` or an audio-specific crate.
- Changes to the audio callback, graph, transport, latency compensation, MIDI timing, automation, plugin host, device backend, sample-rate conversion, render, project schema, recovery, or recording privacy.
- Latency, xrun, CPU, or memory claims; packaging, signing, sandbox, or installer changes that affect audio.
- Review of audio fixtures, performance baselines, or cross-platform audio behavior.

Non-triggers:

- Unrelated products, repositories, or non-audio work.
- Copy-only edits, unrelated documentation, or generic shared-crate changes with no AudioCraft consumer and no contract impact. Still run the core impact check.
- Pure visual token changes outside audio screens with no behavior change.
- Do not use this skill to justify audio changes outside its ownership; escalate to architecture instead.

## 2. Domain model and invariants

| Term | Required invariant |
|---|---|
| Sample clock | Monotonic 64-bit frame counter at the project sample rate. It is the sole authority for scheduling and rendering. Wall clock is for UI and diagnostics only. It changes only on an explicit transport seek or relocation. |
| Sample rate | The project rate is explicit and stored. The device rate may differ. Convert only at a declared boundary and record source, device, and project rates. Never convert silently; preserve pitch and duration. |
| Buffer size | The callback block size in frames. It may vary and may include partial blocks. The graph and every plugin must handle 0 through N frames. Do not assume a fixed block size. Apply control changes at block boundaries. |
| Audio graph | Directed acyclic graph of sources, processors, buses, sends/returns, plugin instances, meters, and output. Use deterministic topological order and explicit summing. Publish graph edits at a safe block boundary. Never mutate the live graph from the UI or the audio thread. |
| Tracks | Typed audio, MIDI, bus, return, or master tracks. Routing, gain, pan, mute, solo, and record arm are explicit. Document mute/solo semantics and avoid hidden global state. |
| Clips | Non-destructive references to media or MIDI with source offset, timeline start, duration, fades, and loop settings. Trimming and moving never rewrite source media. Missing media is an explicit, recoverable error. |
| MIDI events | Timestamped in musical ticks with a PPQ resolution and tempo map, then converted to sample frames per block. Dispatch sample-accurately. Pair note-on/note-off and send all-notes-off on stop, seek, and device loss. No stuck notes. |
| Piano roll | Editor view over a MIDI clip. Quantize, velocity, transpose, and resize are undoable commands. Edits do not mutate live playback buffers. Preview and audition are bounded. |
| Patterns and arrangement | Patterns are reusable content. Placement is an instance with explicit reference or copy semantics. Changing a shared pattern affects exactly the documented instances. Arrangement edits are undoable. |
| Automation | Parameter lanes with breakpoints and curves, evaluated per block with bounded interpolation. Smoothing prevents zipper noise. Use sample-accurate evaluation where the contract requires it. Automation is project data, not plugin-only state. |
| Plugin instances | Stable identifier, format, path, channel configuration, state blob, reported latency, and bypass state. State save/restore must round-trip. A plugin is untrusted native code. A missing or failed plugin is explicit and never silently replaced. |
| Latency compensation | Every node reports latency in samples. The host aligns parallel paths at mix and compensates record and monitor paths. Apply compensation changes at a safe boundary with bounded re-sync. Report input, output, graph, and total latency. Never ignore compensation. |
| Project state | Versioned canonical document containing sample rate, tempo map, tracks, clips, routing, automation, plugin state, and asset references. Save atomically. Validate before replacing the live session. Handle unknown fields by a documented policy. |
| Undo and redo | Command-based, bounded history with coalescing for continuous edits, and reversible state. Destructive recording creates a new take instead of overwriting. Bound plugin-state snapshots. Perform no undo work on the audio thread. |

### Real-time safety invariants

- No blocking allocation or deallocation in the callback. Preallocate buffers, scratch space, and event queues; use bounded pools and lock-free queues; retire objects to the control thread.
- No blocking locks in the callback. Use atomics, single-producer/single-consumer queues, or published immutable snapshots. No mutex, rwlock, or await on the audio path.
- No file or network I/O in the callback. Disk streaming and decode run on workers into preallocated ring buffers.
- No logging, formatting, syscalls, or other potentially blocking work. Write fixed-size telemetry records to an audio-safe ring and drain them on another thread.
- No panic or unwind across the callback or FFI boundary. Avoid panics on the audio path. If a boundary panic occurs, contain it, ramp to silence, and mark the stream failed; never emit garbage.
- Bound all work per callback by frame count, node count, event count, and plugin budget. Drain queues with fixed limits. No unbounded loops. Measure the worst case.
- Do not put an async runtime in the callback. Never block the audio thread on a control or worker thread.
- Enable flush-to-zero or denormals-are-zero where supported, or add a safe DC offset. Test silent tails in reverbs and filters.
- Fail silent. On underrun, plugin failure, or invalid state, mute or ramp to silence and preserve the session. Do not continue with corrupt state.
- Preserve sample-accurate event order. Do not schedule from wall clock.

## 3. Threat boundaries and security

- Treat audio, MIDI, project, plugin-state, metadata, and filename input as untrusted. Parse with bounded lengths, counts, and depth; reject malformed input; do not panic, allocate without limit, or execute content.
- Plugin sandboxing: native plugins are untrusted. Define process isolation or an equivalent containment boundary with least privilege, no network by default, path and architecture validation, and out-of-process scanning. Do not claim sandboxing that is not implemented.
- Plugin crash containment: a plugin crash, hang, or runaway allocation must not take down the DAW, corrupt the project, or lose recorded audio. Use an isolated host or watchdog, quarantine the plugin, bypass it, keep the session editable, and offer recovery. In-process hosting cannot contain a native crash; state that limitation explicitly.
- Path safety: canonicalize and constrain project assets, recordings, and plugin paths. Reject traversal, absolute-path injection, symlink escapes, reserved device names, and platform separator tricks. Create temporary files with restrictive permissions and clean them on failure.
- Sample-rate conversion: validate rates, channel counts, and sample formats. Use a tested, bounded resampler. Document quality and latency. Dither only on final bit-depth reduction. Never silently resample or pitch-shift.
- Malformed metadata: bound strings and tags, sanitize before display or logging, and prevent format-string, HTML/script, or terminal injection. Project and plugin metadata must not carry executable content.
- No silent upload: recordings, projects, MIDI, and plugin state stay local by default. Any upload, cloud render, telemetry, or crash report requires explicit consent and must exclude audio and project content unless the user selects it.
- Corrupt project data: fail closed. Validate schema, references, and integrity before mutating live state. Never partially load and then save over the original. Preserve the original, write a recovery copy, and report the diagnostic.
- Plugin state: bound the blob size, check schema and version, and never deserialize into executable behavior. Incompatible state fails explicitly and preserves the session.
- Permissions and sandbox: request microphone, file, and plugin access explicitly. Preserve platform sandboxing and hardened runtime. Never disable a security control for convenience.
- Secrets and privacy: keep keys and tokens out of projects and logs. Keep recording content out of diagnostics by default.

## 4. Interfaces and dependencies

### Audio backend

- Abstract CoreAudio on macOS, WASAPI on Windows, and ALSA and JACK on Linux, plus JACK where available on macOS.
- The backend owns device enumeration, stream open/close, the callback, sample format, clock, and hotplug events.
- Handle shared versus exclusive mode, OS resampling, variable buffer sizes, device rate changes, aggregate devices, and channel-count changes. JACK transport and latency are external; do not assume the DAW owns the clock.
- Report backend, device, buffer, sample rate, and latency in diagnostics.

### Plugin formats

- CLAP and VST3 are native ABI formats. Select formats per architecture and license review; do not claim support before the full workflow passes on each target.
- CLAP is an open format; verify the exact SDK license in the dependency map. The VST3 SDK is dual-licensed under GPLv3 or a Steinberg proprietary license, and VST is a Steinberg trademark. License review is mandatory before distribution; do not assume redistribution rights.
- Technical constraints include architecture and ABI match, channel layouts, variable block sizes, sample-accurate events, parameter changes, latency reporting, state size, GUI threading, and offline versus realtime behavior.
- Scan plugins out of process. Host untrusted plugins behind a containment boundary. A plugin failure is a recoverable session event, not a process-wide crash.

### MIDI I/O

- Provide adapters for CoreMIDI, Windows MIDI Services, the ALSA sequencer, and JACK MIDI where available.
- Convert device timestamps to the sample clock with bounded jitter. Handle hotplug, multiple ports, and stuck notes.
- MIDI 1.0 is the baseline. MIDI 2.0 or UMP support requires capability detection and explicit tests; do not imply it.

### Sample-rate conversion

- Use one tested resampler boundary for import, device mismatch, and render targets. Define quality and latency policy, bound buffers, prevent clipping, and disclose conversion in the UI and project metadata.

### Render and export contract

- Offline render is deterministic, block-size independent, and faster than real time where feasible.
- Apply latency compensation, automation, fades, and documented plugin offline behavior. Include the documented tail, produce the exact requested duration, prevent clipping, dither only at final bit-depth reduction, and write stable metadata.
- Compare online capture and offline render on the same fixture within a stated tolerance. A mismatch is a defect unless documented.

### Project format

- Versioned, documented, atomically written, and independent of absolute install paths. Store sample rate, tempo map, routing, clips, automation, plugin state, and relative asset references.
- Define migration and unknown-field policy. Reject executable content. Make the integrity and checksum policy explicit.
- Do not finalize an extension or schema in this skill; architecture owns the format.

### Threading contract

| Context | Allowed work |
|---|---|
| Real-time audio callback | Graph processing, plugin process calls, sample-accurate MIDI and automation dispatch, atomic meters, bounded event drain, silence on failure |
| Control thread | UI, graph edits, transport commands, load/save, undo/redo, plugin scan/load, render scheduling |
| Worker threads | Disk streaming, decode/encode, sample-rate conversion, waveform peaks, autosave, plugin scan and host process |
| Plugin host process | Untrusted plugin execution isolated from project and UI, with versioned IPC |

Never block the audio thread on control or worker work. Never let control code mutate live graph memory without a safe publication boundary.

## 5. Resource concerns

- Latency: report input, output, graph, plugin, and total round-trip latency in samples and milliseconds. Keep the monitoring path within a declared budget. Measure with loopback, not only backend-reported values.
- Buffer underruns and xruns: count callback deadline misses and backend underruns with audio-safe counters. Record device, backend, buffer, sample rate, graph, plugin set, and duration. Acceptance requires zero xruns under the declared workload; a dropout is a failure, not a tolerable artifact.
- CPU scheduling: run the audio thread at real-time priority where the OS supports it: CoreAudio, MMCSS on WASAPI, and rtkit or JACK on Linux. Avoid priority inversion, blocking, and unbounded plugin calls. Measure headroom on the worst-case graph, not an idle one.
- Priority inversion: never let the audio thread wait on a lock held by UI or a worker. Do not scan or load plugins synchronously on the audio thread. Use lock-free handoff and bounded retries.
- Denormals: use flush-to-zero, denormals-are-zero, or a DC offset. Test reverb and filter tails in silence. A denormal CPU spike is a defect.
- Memory allocation: preallocate callback buffers, event queues, scratch space, and plugin buffers. Bound caches and undo history. Monitor RSS and allocation counts. Require a plateau over a long session.
- Plugin CPU and memory: measure per instance, apply timeouts and watchdogs, and quarantine runaway plugins. One plugin must not starve the graph. Memory limits are best-effort unless isolation enforces them.
- Disk streaming: use worker read-ahead into ring buffers, handle slow or removed disks, and never read files in the callback. Missing media is explicit and recoverable.
- Sample cache: bound decoded audio and waveform-peak caches with eviction and invalidation on file change. Prevent unbounded growth in large sessions.
- Project load: validate and load progressively or off-thread, make it cancelable, and do not freeze the UI or mutate the live session before validation succeeds. Bound large-project memory.
- Installer size: measure package size. Keep plugin SDKs, codecs, and optional content out of the base installer where licensing or size requires it, and record the tradeoff.

Measurement:

- Xruns: run a sustained fixture at the target buffer, rate, and graph; check counters and the rendered or audible result; report the exact environment.
- Round-trip latency: use a physical or virtual loopback impulse, compare input to output, subtract declared backend and plugin latency, and report samples, milliseconds, and variance.
- CPU: record DSP load and worst-case callback duration over the session.
- Memory: record RSS, peak, and plateau over a long session.
- No performance claim without baseline, workload, environment, command, and result.

## 6. Required end-to-end workflows

Generate fixtures deterministically where possible. Commit only small approved binaries.

- 48 kHz, 24-bit stereo WAV with a 1 kHz sine at -18 dBFS for 10 seconds; an impulse; and a unity stereo identity.
- MIDI file at 120 BPM with notes on bar and beat boundaries and explicit note-offs.
- Project with audio tracks, a MIDI track, a bus, a send, automation, and at least one plugin.
- Crash and hang test plugins, a 44.1 kHz asset, a truncated or corrupt project, and a long-session script.
- Place fixtures under the owning app or crate test directory when the workspace exists. Do not add large media without approval.

| Workflow | Success criteria |
|---|---|
| Record or import audio | Arm a track, record 10 seconds, stop, and import a 48 kHz WAV. Verify duration, gain within ±0.1 dB, no clipping, a local recording, a new take on undo, and disclosed conversion when applicable. |
| Arrange tracks | Use two tracks, trim and move clips, add a crossfade, and change bus gain. Verify the known mix gain, no clicks at boundaries, latency-compensated alignment, and undo/redo restoration. |
| Sequence MIDI | Use 120 BPM notes, edit in the piano roll, and loop. Verify sample-accurate timing within ±1 sample, note-off delivery, no stuck notes, and a clean loop boundary. |
| Automate a parameter | Draw a volume ramp and a mute. Verify the curve and smoothing, no zipper noise, sample-accurate behavior where required, and persistence after reopen. |
| Save and reopen | Save, close, and reopen. Verify semantic equality of project data, plugin state, asset references, and automation. Verify atomic save and no partial overwrite. |
| Render and export | Render offline and compare with real-time capture. Verify exact duration, latency compensation, tail, no clipping, final-stage dither, deterministic repeats, and the stated tolerance. |
| Plugin crash | Crash and hang a plugin during playback. Verify the DAW survives, the session stays intact, the plugin is quarantined or bypassed, recovery is documented, and reports contain no audio. |
| Sample-rate mismatch | Use a 44.1 kHz asset and device with a 48 kHz project. Verify explicit conversion, correct pitch and duration, disclosure, and no silent mismatch. |
| Device disconnect | Unplug during playback and recording. Verify graceful stop, no hang, preserved recorded data, recovery on reconnect, and no stuck notes. |
| Project recovery | Kill the process during edit and during save. Verify autosave or recovery opens the last consistent state, a corrupt project fails closed, the original is preserved, and diagnostics are actionable. |
| Long-session stability | Run two hours of playback and recording with automation and plugins at the declared buffer. Verify zero xruns, a memory plateau, no timing drift, and bounded undo history. |

Timing, gain, and dropout verification:

- Timing: internal events are sample-accurate within ±1 sample at the project rate. Measure and compensate loopback offset. Require no cumulative drift.
- Gain: a known sine is within ±0.1 dB, unity paths null, and nothing clips unexpectedly.
- Dropouts: require zero xruns under the declared workload. Any dropout fails the run and must capture counters and reproduction data.
- Do not mark a workflow `PASS` from unit tests alone; run the end-to-end path.

## 7. Failure modes, debugging, regression tests, and platforms

### Failure modes

- Xruns and clicks: buffer too small, CPU spike, blocking plugin, disk stall, denormal spike, or priority inversion. Check counters and isolate the graph.
- Plugin hang or crash: an in-process crash can kill the DAW. Require isolation, a watchdog, quarantine, and session recovery.
- Sample-rate mismatch or clock drift: wrong pitch or duration and drift over time. Require explicit conversion and one clock or drift compensation.
- Device disconnect or hotplug: invalid streams, stuck notes, and lost recordings. Require graceful stop and recovery.
- Corrupt project or partial save: fail closed, preserve the original, and write a recovery copy.
- MIDI stuck notes or jitter: faulty timestamp conversion, missing note-off, or hotplug. Send all-notes-off and bound jitter.
- Undo or latency-compensation inconsistency: a graph-edit race or stale latency. Apply changes at a safe boundary.
- UI freeze: loading, scanning, or rendering on the UI thread. Move the work to a worker.
- Path, case, or Unicode defects: macOS normalization differences, Windows case-insensitive and reserved names, and Linux case sensitivity. Normalize and test.

### Debugging procedure

1. Reproduce with the smallest fixture. Record commit, OS, backend, device, buffer, sample rate, plugin versions, and build profile.
2. Classify the fault: correctness, real-time, plugin, device, storage, or UI.
3. Read xrun counters and callback timing. Bypass plugins and tracks to isolate, then binary-search graph nodes.
4. Use null tests, impulse or loopback measurement, and offline-versus-online render comparison for timing and gain.
5. For plugin faults, collect isolated-host diagnostics and quarantine the plugin. Do not attach audio or project content to reports by default.
6. For project faults, validate schema and references, preserve the original, and inspect the recovery copy. Do not overwrite.
7. Add a failing regression test before the fix. Verify no audio-thread allocation, lock, or blocking logging by instrumentation and review.
8. Re-run the full workflow and the cross-platform matrix. Record evidence.

### Regression tests

- Unit: graph topological order and cycle rejection, latency-compensation math, tempo map, automation interpolation, sample-rate conversion, project schema and migration, path validation.
- Integration: offline render fixtures, save and reopen, plugin lifecycle, MIDI scheduling, simulated device disconnect.
- Stress: small buffers, many tracks and plugins, long sessions, xrun counters, memory plateau.
- Adversarial: fuzz WAV, MIDI, project, and plugin-state parsers; malformed metadata; path traversal; truncated files.
- Real-time: debug allocator and lock instrumentation on the callback, panic containment, and bounded-queue overflow behavior.
- Cross-platform: run the matrix on each supported backend. Mark unavailable checks `BLOCKED`, not passed.

### Cross-platform caveats

- macOS: CoreAudio device and rate changes, aggregate devices, microphone permission, hardened runtime and notarization, sandbox container paths, universal binaries, and plugin architecture.
- Windows: WASAPI shared mode may resample, exclusive mode can change format, MMCSS, device hotplug, MIDI services, path length and Unicode, and plugin architecture.
- Linux: ALSA versus JACK versus PipeWire JACK compatibility, real-time scheduling through rtkit or `RLIMIT_RTPRIO`, no hard real-time guarantee, X11 and Wayland plugin GUIs, and Flatpak or Snap sandbox limits.
- Plugin formats: CLAP and VST3 differ in ABI, licensing, GUI threading, and offline behavior. ARM64 and x86_64 plugins do not mix. Native plugins are not sandboxed by default.
- Paths: separators, case sensitivity, Unicode normalization, reserved names, and symlinks. Never assume a path is portable.
- Clocking: multiple devices need one clock or drift compensation. JACK and aggregate devices change clock ownership. Do not silently drop samples.

## 8. Release, documentation, and handoff

Trigger this section when audio paths, project schema, plugin hosting, the render contract, supported devices or platforms, latency and xrun claims, recording privacy, packaging, or signing change.

Update as applicable:

- `README.md` status only when behavior is actually shipped. Never present planned AudioCraft features as available.
- `AGENTS.md` skill activation map and safe commands.
- Architecture documents and ADRs for the graph, threading, plugin isolation, project format, and render contract.
- Product spec and acceptance criteria, a performance baseline with environment, and threat-model and privacy notes for recordings and plugins.
- Changelog, version, release notes, installer and signing metadata, and the supported-platform matrix.
- Fixtures and regression tests. Mark unavailable platform checks `BLOCKED`.

Handoff:

- `eqwec-engineering-core` for architecture, security, Git, versioning, dependency, or documentation conflicts.
- `eqwec-quality-control` for pre-change, implementation, verification, performance, merge, and release gates. Provide evidence, not claims.
- Record the skill usage matrix. A missing mandatory skill is `BLOCKED`.

## Red flags

- Allocating, freeing, locking, logging, formatting, or performing file or network I/O in the audio callback.
- Unbounded plugin processing, event queues, graph edits, or undo history.
- Ignoring latency compensation or reporting latency without measurement.
- Sample-rate mismatch or OS resampling without disclosure.
- Treating a dropout or xrun as acceptable.
- A plugin crash, hang, or runaway allocation taking down the DAW or corrupting the project.
- Panic or unwind across the callback or FFI boundary.
- Wall-clock scheduling instead of the sample clock, or cumulative timing drift.
- UI or control code mutating the live graph without a safe boundary.
- Silent upload of recordings, projects, or plugin state.
- A partial save overwriting a good project, or a corrupt project loading without failing closed.
- Trusting plugin state or project metadata, unbounded parsing, or path traversal.
- Unmeasured denormal spikes or priority inversion.
- Claiming a platform, backend, plugin format, or feature is supported without an end-to-end test and license review.
- A performance or latency claim without baseline, workload, environment, and result.
