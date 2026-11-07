# ADR-0005: Audio backend and plugin hosting

- **Status:** ACCEPTED (with a mandatory real-time prototype gate)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0001, ADR-0010, `docs/architecture/SYSTEM-ARCHITECTURE.md`, `docs/performance/PERFORMANCE-STRATEGY.md`

## 1. Context

AudioCraft is a digital audio workstation. FilmCraft and EffectCraft also need audio playback, synchronization, and rendering. The audio path has stricter constraints than any other part of EQwec: it must run in real time, must not allocate or block in the callback, and must survive plugin faults without taking down the application.

The audio backend and plugin host are expensive to change after the graph, project format, and plugin database are built. The project has no audio prototype, so the decision must define the architecture and the evidence required before product work.

## 2. Requirements

- Low-latency, glitch-free playback and recording on Windows, macOS, and Linux.
- A sample clock that is the sole authority for scheduling and rendering.
- No allocation, blocking lock, file I/O, network, logging, or unbounded work in the real-time callback.
- Bounded lock-free or message-passing queues between the callback and UI/disk workers.
- Multitrack audio, MIDI, automation, and plugin hosting.
- Crash isolation for plugins; a plugin fault must not crash the DAW.
- Plugin scanning that is safe, bounded, and does not block the audio thread.
- Support for CLAP, VST3, and LV2 where legally and technically feasible.
- No AAX in the first release.
- Deterministic offline rendering that matches real-time playback.
- Project recovery and audio file integrity.
- Cross-platform device selection, sample-rate conversion, and channel layouts.

## 3. Alternatives considered

### A. CPAL

Pure Rust, MIT, cross-platform audio I/O over CoreAudio, WASAPI, ALSA, and other backends. Good starting point, but the callback behavior, device-change handling, and latency differ by backend. Requires a project-owned real-time-safe abstraction and platform testing.

### B. Direct platform APIs

CoreAudio (macOS), WASAPI (Windows), and ALSA/PipeWire/JACK (Linux) give the most control and the best latency. They require three implementations and platform-specific expertise. Use them behind the CPAL interface where CPAL is insufficient.

### C. JACK

Low-latency professional audio on Linux and macOS; LGPL-2.1+. Good for pro users and inter-application routing, but not a cross-platform default and adds a dependency and configuration burden. Offer as an optional backend.

### D. GStreamer

Strong pipeline and codec integration, but it is a media framework, not a DAW audio graph. Adds a large dependency and a second processing model. Not the audio engine.

### E. CLAP

MIT, open plugin standard with parameter automation, state, note expression, and a clear C ABI. Strong fit for a new host. Smaller installed base than VST3, but growing and legally clean.

### F. VST3

MIT since SDK 3.8; broad installed base. Requires Steinberg trademark compliance and a more complex API. Host as a secondary format.

### G. LV2

ISC, open standard, strong on Linux. Host later, after CLAP and VST3 are stable.

### H. AAX

Proprietary, paid, and tied to Avid's ecosystem. Rejected for the first release.

## 4. Evidence

- CLAP is MIT and designed as an open plugin standard.
- VST3 is MIT since SDK 3.8; Steinberg's trademark and logo usage rules still apply.
- LV2 is ISC.
- CPAL is MIT.
- JACK is LGPL-2.1+; dynamic linking and notices are required if used.
- Audio plugin binaries are native code with the user's privileges unless the host isolates them. A plugin crash can take down the host if hosted in-process.
- Real-time audio callbacks must not allocate, lock unboundedly, perform I/O, or wait on the UI thread; these are standard real-time constraints, not EQwec-specific measurements.
- No EQwec audio latency, xrun, or plugin-performance measurement exists. All numeric budgets are `BLOCKED` until the prototype.

## 5. Decision

### 5.1 Audio I/O

1. Use CPAL as the first cross-platform audio I/O implementation.
2. Put CPAL behind a project-owned `eqwec-audio-io` trait with explicit device, sample-rate, buffer-size, channel-layout, and callback contracts.
3. Use platform backends (CoreAudio, WASAPI, ALSA/PipeWire) behind the same trait where CPAL cannot meet latency or device requirements.
4. Offer JACK as an optional Linux/macOS backend, subject to LGPL compliance.
5. The audio callback owns the sample clock. It performs no allocation, blocking lock, file I/O, network, logging, or unbounded work.
6. Communicate with the UI and disk workers through bounded lock-free or message-passing queues. Define backpressure and drop policy explicitly.
7. Offline rendering uses the same graph and processing code as real-time playback, with a deterministic block size and no wall-clock dependence.

### 5.2 Plugin hosting

1. Host CLAP first, VST3 second, and LV2 later.
2. Host plugins out of process where feasible. A plugin crash must not take down the DAW.
3. Scan plugins in a separate process with timeouts, crash isolation, and bounded resource use. Never scan on the audio thread.
4. Maintain a plugin database with path, hash, vendor, version, format, scan result, and trust state.
5. Require explicit user consent before loading a newly discovered or changed plugin binary.
6. Do not claim a security sandbox until the platform-specific isolation is implemented and tested. Out-of-process hosting is crash isolation, not a complete security boundary.
7. Do not support AAX in the first release.
8. Do not bundle third-party plugins without a distribution license.

### 5.3 MIDI and synchronization

- Use `midir` for MIDI I/O and `wmidi` for messages, subject to admission review.
- Timestamp MIDI events against the sample clock; do not rely on wall-clock timestamps in the audio graph.
- Define how FilmCraft and EffectCraft synchronize audio and video: a shared transport clock, sample-accurate offsets, and explicit drift correction.
- Test device hot-plug, sample-rate changes, and buffer-size changes without crashing or leaking resources.

### 5.4 Project and rendering

- AudioCraft owns its project format (ADR-0006).
- Render output is deterministic for a given project, sample rate, block size, plugin set, and seed (where randomization is involved).
- Record plugin versions and states in the project; a missing plugin must produce a clear, recoverable error, not silent data loss.
- Preserve audio recordings and project data on uninstall by default.

## 6. Prototype gate

Before AudioCraft product work, a prototype must prove on Windows, macOS, and Linux:

1. A real-time graph with at least 16 audio tracks and 4 MIDI tracks at the declared buffer size.
2. Zero xruns in a defined stress test on the mid-range hardware profile; record the exact buffer size, sample rate, and workload.
3. No allocation in the audio callback, verified by instrumentation.
4. CLAP and VST3 plugin load, parameter automation, state save/restore, and crash isolation.
5. Plugin scan of a representative set with timeouts and no host crash.
6. Device hot-plug, sample-rate change, and buffer-size change.
7. Audio recording and playback with file integrity checks.
8. Offline render matching real-time playback within a defined tolerance.
9. Project save/reopen with plugin state and missing-plugin recovery.

The prototype is a gate, not a product. Results are recorded in `docs/performance/` and this ADR is updated.

## 7. Consequences

**Positive**

- A Rust-owned, real-time-safe audio architecture.
- CLAP-first hosting avoids the VST3 complexity for the core path and is legally clean.
- Out-of-process hosting contains plugin crashes.
- One graph for playback and render improves determinism.

**Negative**

- CPAL may not meet the lowest-latency requirements on every platform; platform backends are needed.
- Out-of-process plugin hosting adds IPC, shared-memory, and lifecycle complexity.
- CLAP has a smaller installed base than VST3; users may need VST3 for compatibility.
- Plugin sandboxing is platform-specific and may be incomplete.
- Real-time testing requires dedicated hardware and instrumentation.

**Security**

- Plugin binaries are untrusted native code. Out-of-process hosting, least privilege, and explicit consent are required.
- Scan results and plugin hashes are advisory; a signed plugin is not automatically safe.
- Do not load plugins from untrusted paths or network locations.
- Audio files and project files are untrusted input; bound parsers and fuzz them.

**Performance**

- The callback must remain allocation-free and blocking-free.
- Measure callback time, xrun count, plugin CPU, latency, and memory per track.
- Do not claim low latency without a recorded buffer size, sample rate, hardware, and workload.

**Licensing**

- CPAL MIT; CLAP MIT; VST3 MIT since 3.8; LV2 ISC; JACK LGPL-2.1+.
- VST trademark usage must follow Steinberg's guidelines.
- AAX is proprietary and excluded.
- Plugin binaries have their own licenses; do not redistribute them without permission.

**Maintenance**

- Plugin formats evolve; the host boundary must be versioned and tested.
- Platform audio backends require per-platform ownership.
- Real-time regressions are release blockers.

## 8. Reconsideration criteria

Revisit this ADR if:

- CPAL cannot meet the latency or device requirements on a target platform.
- Out-of-process plugin hosting cannot meet the real-time budget.
- CLAP adoption is insufficient and VST3 must become primary.
- A platform plugin format (AU on macOS) becomes a product requirement.
- The prototype fails the xrun, allocation, or crash-isolation gates.
- A plugin sandbox becomes feasible and materially improves security.
- FilmCraft/EffectCraft synchronization requires a different transport model.

## 9. Links

- `docs/architecture/SYSTEM-ARCHITECTURE.md`
- `docs/performance/PERFORMANCE-STRATEGY.md`
- `docs/security/THREAT-MODEL.md`
- `docs/research/LEGAL-LICENSE-REVIEW.md`
- ADR-0010
