# Performance Strategy

**Status:** approved methodology. No benchmark exists and no numeric budget is verified. All product performance claims are `BLOCKED` until measured.

## 1. Principles

- Measure before optimizing. Profiling evidence precedes a change.
- Use representative real workloads, not synthetic microbenchmarks alone.
- A performance claim requires hardware, OS, build profile, workload, command, baseline, observed result, and regression threshold.
- Optimize the largest verified bottleneck, not the most interesting one.
- Do not trade correctness, security, data integrity, or accessibility for speed.
- Do not disable a security control to reduce memory or startup time.

## 2. Metrics

| Category | Metrics |
|---|---|
| Startup | Cold start, warm start, time to interactive, time to first frame |
| Memory | Idle RSS, active working set, peak RSS, process-tree total, per-process breakdown, cache bytes |
| CPU/GPU | Utilization, frame time p50/p95/p99, GPU memory, device loss/recovery |
| I/O | Project open/save, media import, export throughput, disk footprint |
| Interaction | Input latency, tab switch, pan/zoom, scrub latency, command latency |
| Media | Decode/encode throughput, dropped frames, A/V sync error, audio xruns |
| Audio | Callback time, buffer underruns, plugin CPU, latency |
| Footprint | Download size, installed size, binary size, optional components |
| Reliability | Crash rate, recovery time, cache hit/eviction, memory growth over time |

## 3. Baseline profiles

Numeric budgets are set per product after the first reproducible baseline. The project uses three hardware classes:

| Class | Definition | Intended evidence |
|---|---|---|
| Entry | Modern 4-core CPU, 8 GB RAM, integrated GPU, SSD | Minimum supported experience; conservative budgets |
| Mid-range | 8-core CPU, 16-32 GB RAM, discrete or strong integrated GPU, SSD | Default development and release baseline |
| Professional | 12+ core CPU, 32-64+ GB RAM, discrete workstation GPU, NVMe | Heavy media, large documents, multi-track work |

Record the actual machine model, CPU, GPU, RAM, storage, OS build, driver, display scale, and power mode. A budget without this record is not reproducible.

## 4. Measurement method

- Use release builds for release claims. Debug-build numbers are not release evidence.
- Warm up caches and run multiple iterations; report median and p95/p99 where relevant.
- Measure the whole process tree, not only the main process.
- Separate cold and warm runs; state which is reported.
- Isolate the workload: no background sync, indexing, updates, or unrelated applications.
- Record the exact command and fixture hash.
- Compare like with like: same workload, same platform, same build profile, same display scale.
- Store raw results and the analysis, not only a summary number.
- If a result cannot be reproduced, mark the claim `BLOCKED` and remove it from release notes.

## 5. Provisional goals (hypotheses, not verified)

These are planning targets to be validated or rejected by measurement. They are not promises.

| Area | Provisional goal | Validation |
|---|---|---|
| Hub cold start | Interactive launcher in under 1 second on mid-range hardware with cached registry and network disabled | Measured baseline on each platform |
| Hub idle memory | No resident background daemon unless justified; bounded idle RSS | Process-tree measurement |
| Browser memory | Explore a 20-30% reduction versus a named reference browser under an equivalent workload | Reproducible benchmark with identical sites, extensions, profiles, and process-tree metrics |
| Browser tab suspension | Reclaim memory without losing session state or weakening isolation | Suspend/resume and restore tests |
| PhotoCraft | Bounded memory for large images via tiles; interactive pan/zoom at the declared document size | Frame-time and RSS fixtures |
| VectorCraft | Interactive pan/zoom and hit-testing on a large document; bounded tessellation cache | Frame-time and cache metrics |
| FilmCraft | Frame-accurate playback and export; no dropped frames in the declared preview workload | Playback and render traces |
| EffectCraft | Incremental evaluation with correct cache invalidation; no stale frames | Cache-hit and correctness tests |
| AudioCraft | Zero xruns at the declared buffer size on supported hardware; no allocation in the callback | Real-time trace and stress test |
| Install size | Report compressed and installed bytes; investigate unexplained growth | Package manifest and measured tree |

A goal that cannot be measured is not a goal. Replace it with a measurable criterion or remove it.

## 6. Regression policy

- A change that exceeds the approved regression threshold requires investigation before merge.
- A deliberate regression requires a documented tradeoff, owner approval, and a remediation plan.
- Performance tests run on a stable machine profile; CI variance is measured before enforcing a threshold.
- Do not accept a microbenchmark improvement that worsens the real workload.
- Track memory growth over long sessions, not only at startup.

## 7. Profiling toolkit (planned)

Tool choice depends on the platform and the selected technology stack. Candidate categories:

- CPU sampling profilers and flame graphs.
- Allocation and heap profilers.
- GPU capture and frame debuggers.
- OS process/memory tools.
- Audio callback and xrun instrumentation.
- Browser process-tree and renderer metrics.

The technology-selection ADR must name the actual tools and verify they work on each supported platform. Do not document a tool as available until it is installed and tested.

## 8. Current state

| Item | State |
|---|---|
| Benchmark harness | None |
| Baseline measurements | None |
| Approved numeric budgets | None |
| Performance gate | `BLOCKED` until runtime code and fixtures exist |
| Reference browser for comparison | Not selected; requires a reproducible protocol |
| Hardware profiles | Defined by class; actual machines not yet recorded |

No performance improvement has been demonstrated. Do not cite the provisional goals as results.
