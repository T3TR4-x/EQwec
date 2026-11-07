# ADR-0004: Media decode, encode, and codec policy

- **Status:** PROPOSED (legal and patent review required)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner, engineering agent, and legal counsel (required)
- **Supersedes:** none
- **Related:** ADR-0002, ADR-0003, ADR-0006, `docs/research/LEGAL-LICENSE-REVIEW.md`, `docs/security/THREAT-MODEL.md`

## 1. Context

FilmCraft, EffectCraft, PhotoCraft, AudioCraft, and Browser need to read and write media. Media support is not a single library choice: it combines container parsing, codec decoding, codec encoding, color management, hardware acceleration, and patent/licensing obligations.

The project cannot responsibly ship every codec on day one. It needs a default set that is legally clear, a gated path for common but patent-encumbered formats, and an architecture that can add codecs without rewriting product semantics.

## 2. Requirements

- Broad decode support for common containers and codecs used by professional media.
- Reliable, frame-accurate video decode and encode for FilmCraft and EffectCraft.
- Audio decode/encode for AudioCraft, FilmCraft, and EffectCraft.
- Image and RAW decode for PhotoCraft.
- Color management that does not assume 8-bit sRGB everywhere.
- Bounded parsing of untrusted media; no script or network execution.
- Hardware acceleration where available, with a CPU fallback.
- A license-compatible distribution model; no accidental GPL or proprietary contamination.
- A recorded codec set, build flags, notices, and source offer per release.
- No DRM in the first release.
- A credible path to H.264/AAC playback if the product requires it, subject to legal review.

## 3. Alternatives considered

### A. FFmpeg libraries (LGPL-only build)

Broadest format and codec coverage, mature, cross-platform, hardware acceleration through platform APIs. LGPL-2.1-or-later if GPL-only components are disabled. Requires dynamic linking or a compliant relinking path, notices, source offer, and exact build-flag records. Patent obligations are separate from copyright.

### B. FFmpeg as a subprocess

The application invokes a separately distributed FFmpeg binary for export, batch rendering, and format conversion. This isolates the LGPL boundary, simplifies relinking, and makes the build flags auditable. It is less suitable for low-latency playback and frame-accurate scrubbing, and it adds process-management and temp-file complexity.

### C. GStreamer

Strong pipeline model, hardware acceleration, and plugin ecosystem. LGPL-2.1+ core, but plugin and codec packaging varies; some plugins are GPL or have patent issues. Adds a large dependency graph and a second media pipeline model. Deferred.

### D. Platform media frameworks

AVFoundation (macOS), Media Foundation (Windows), and GStreamer/VA-API (Linux) provide hardware decode/encode and OS integration. They fragment behavior, require per-platform implementations, and may not support professional formats or codecs consistently. Use them for hardware acceleration behind a shared interface, not as the only media layer.

### E. Symphonia and format-specific Rust crates

Symphonia (MPL-2.0) covers common audio formats in pure Rust. Rust image crates cover PNG, JPEG, WebP, TIFF, and OpenEXR. They are safe and permissively licensed but do not cover professional video containers, codecs, or hardware acceleration. Use them where they fit, not as the whole media stack.

### F. libx264 / libx265

GPL-2.0/GPL-3.0 encoders. They cannot be shipped in a proprietary build without making the combined work GPL-compatible. Excluded from the default distribution.

## 4. Evidence

- FFmpeg is LGPL-2.1-or-later by default; using GPL-only parts makes the GPL apply. FFmpeg's legal page provides an LGPL compliance checklist.
- Chromium's proprietary codec support is limited to Chrome-branded builds; CEF's default binaries do not include H.264/AAC. Enabling them requires `proprietary_codecs=true` and `ffmpeg_branding="Chrome"` plus patent/distribution review.
- VST3 is MIT since SDK 3.8; CLAP is MIT; LV2 is ISC. These are plugin formats, not media codecs.
- LibRaw is LGPL-2.1 or CDDL-1.0; the LGPL option is preferred.
- OpenColorIO is BSD-3-Clause; LittleCMS is MIT.
- Symphonia is MPL-2.0.
- H.264, AAC, and HEVC involve patent pools and are not covered by a copyright license. AV1, VP9, Opus, FLAC, PNG, JPEG, and WebP are the royalty-free default candidates, subject to verification.
- Widevine, FairPlay, and PlayReady require separate agreements and provisioning.

## 5. Decision

### 5.1 Default media stack

1. **Video/container decode:** FFmpeg libraries under an LGPL-2.1-or-later build, dynamically linked, with GPL-only components disabled. Record the exact `configure` flags and resulting codec list.
2. **Export and batch rendering:** prefer FFmpeg as a subprocess for the first FilmCraft implementation. This isolates the LGPL boundary and simplifies notices and relinking. Use linked libraries where real-time playback requires them.
3. **Hardware acceleration:** platform APIs (VideoToolbox, Media Foundation, VA-API) behind a shared `eqwec-media` interface, with CPU fallback.
4. **Audio decode:** Symphonia (MPL-2.0) where it covers the format; FFmpeg for containers/codecs it does not.
5. **Audio output:** CPAL (MIT) with platform backends behind a real-time-safe interface.
6. **Images:** `image` for common raster formats; `exr` for OpenEXR; format-specific crates where control is needed.
7. **RAW:** LibRaw under LGPL-2.1, dynamically linked. Evaluate RawSpeed only if LibRaw cannot meet performance or camera coverage.
8. **Color:** LittleCMS (`lcms2`, MIT) for ICC; OpenColorIO (BSD-3-Clause) as an evaluation for ACES and complex pipelines. The photo pipeline uses a linear working space, bounded tiled processing, and an explicit display transform.

### 5.2 Codec policy

- **Default:** royalty-free formats: PNG, JPEG, WebP, AVIF, VP9, AV1, Opus, FLAC, Ogg Vorbis.
- **Legal-gated:** H.264/AVC, AAC, HEVC/H.265, ProRes, DNxHR/DNxHD. Do not ship them in the default build until counsel or an authorized licensing decision approves the distribution model.
- **Excluded:** libx264, libx265, and any GPL-only FFmpeg component in the proprietary build.
- **DRM:** no Widevine, FairPlay, or PlayReady in the first release. A product that requires streaming DRM must complete agreements, provisioning, robustness, and privacy review first.
- **Evidence:** every release records the codec set, build flags, notices, source offer, and patent decision. A build that silently enables a GPL or proprietary component is a release blocker.

### 5.3 Format policy

- Native project formats are owned by each product (ADR-0006).
- Interchange uses open standards where practical: PNG, JPEG, WebP, AVIF, OpenEXR, TIFF, SVG, PDF, WAV, FLAC, Opus, MP4/Matroska, OpenTimelineIO.
- Do not claim full compatibility with proprietary project formats (PSD, AI, AAF, FCPXML, etc.) without documented, tested fidelity and legal review.
- Decode-only support for a proprietary format does not imply encode support or round-trip fidelity.

## 6. Conditions and gates

1. **Legal gate:** codec patent and distribution review before any gated codec ships. `BLOCKED` until counsel or an authorized licensing decision exists.
2. **Compliance gate:** LGPL checklist, notices, source offer, and relinking method verified per artifact.
3. **Correctness gate:** frame-accurate decode/encode tests with representative fixtures; color round-trip tests; audio sync tests.
4. **Security gate:** bounded parsing, fuzzing, and process isolation for risky decoders.
5. **Performance gate:** measured decode/encode throughput, dropped frames, seek latency, and memory on the defined hardware profiles.
6. **DRM gate:** separate ADR and agreements; not part of the first release.

## 7. Consequences

**Positive**

- Broad format support with a legally clearer default.
- One media abstraction across FilmCraft, EffectCraft, PhotoCraft, AudioCraft, and Browser.
- Subprocess export simplifies LGPL compliance for the first release.
- Hardware acceleration can be added without changing product semantics.

**Negative**

- FFmpeg LGPL compliance is ongoing work: notices, source offers, build flags, relinking.
- Subprocess export adds latency and process-management complexity.
- H.264/AAC playback may be unavailable until legal review completes; this is a product limitation, not a hidden failure.
- Professional formats (ProRes, DNxHR, HEVC) are gated.
- Platform hardware paths require per-platform testing.

**Security**

- Media parsers are a primary attack surface; use bounds, fuzzing, and process isolation where feasible.
- Do not execute scripts, fetch external references, or auto-open media.
- Treat decoder crashes as contained failures, not application crashes.
- Verify package and codec provenance.

**Performance**

- Measure decode/encode throughput, seek latency, dropped frames, A/V sync error, and process-tree memory.
- Do not claim codec performance without a workload and environment.
- Hardware acceleration must fall back safely and produce equivalent output within a documented tolerance.

**Licensing**

- FFmpeg LGPL-2.1-or-later; no GPL components in the default build.
- LibRaw LGPL-2.1; Symphonia MPL-2.0; OpenColorIO BSD-3-Clause; LittleCMS MIT.
- Codec patents are separate; gated formats require counsel.
- DRM requires agreements.

**Maintenance**

- FFmpeg security updates must be tracked and rebuilt.
- Codec/format support grows by product need, not by checklist.
- A subprocess boundary can be replaced by linked libraries if playback requires it, but the compliance model must be re-reviewed.

## 8. Reconsideration criteria

Revisit this ADR if:

- Counsel determines that a gated codec can be shipped under an acceptable license.
- FFmpeg LGPL compliance proves impractical for the distribution model.
- Hardware acceleration or subprocess export cannot meet the playback/export budgets.
- A platform media framework becomes the better cross-platform path.
- DRM becomes a product requirement.
- LibRaw, Symphonia, or OpenColorIO fails a maintenance, security, or performance gate.
- A proprietary format requires a vendor agreement or a clean-room interoperability decision.

## 9. Links

- `docs/research/LEGAL-LICENSE-REVIEW.md`
- `docs/architecture/TECHNOLOGY-SELECTION.md`
- `docs/security/THREAT-MODEL.md`
- ADR-0003
- ADR-0006
