# Legal and License Review

| Field | Value |
|---|---|
| Status | Planning review. **Not legal advice.** No distribution is approved until qualified counsel and the project owner review the relevant items. |
| Date | 2026-11-07 |
| Scope | Project license, third-party dependencies, codecs, DRM, assets, trademarks, and distribution obligations |
| Decision owner | Project owner; counsel required for codec, DRM, and copyleft conclusions |
| Related | `docs/adr/0008-original-project-license.md`, `docs/architecture/TECHNOLOGY-SELECTION.md`, `docs/architecture/DEPENDENCY-MAP.md` |

## 1. Why this document exists

EQwec plans to distribute native desktop applications on Windows, macOS, and Linux. That creates obligations that do not exist for a private prototype:

- every third-party component must be identified, licensed, and noticed;
- some licenses require source disclosure, relinking, or reverse-engineering permissions;
- codecs and DRM may require patent licenses or commercial agreements;
- fonts, icons, sample media, and trademarks have separate terms;
- the project's own license determines what users and contributors may do.

This review records the current state and the questions that must be answered before a public release. It does not replace counsel.

## 2. Evidence standard

- **Verified:** stated by the official project, vendor, or license text cited in section 12.
- **Assumption:** a planning judgment that follows from verified facts but is not a legal conclusion.
- **UNVERIFIED:** not confirmed from a primary source; must be checked before admission.
- **BLOCKED:** cannot be resolved without counsel, a commercial agreement, or owner approval.

License identifiers use SPDX names where possible. A package may offer a choice of licenses; EQwec must record which option it relies on, not merely list all options.

## 3. Project license options (ADR-0008)

The project license is **not selected**. The following options were evaluated.

| Option | Obligations | Strategic effect | Assessment |
|---|---|---|---|
| MIT OR Apache-2.0 (dual) | Notice; Apache-2.0 patent grant and attribution; no source disclosure for binaries | Maximum compatibility with Rust crates and commercial adoption; allows proprietary forks; compatible with most permissive dependencies | **Recommended**, subject to owner approval |
| Apache-2.0 only | Notice; patent grant; attribution | Similar to dual, but less convenient for MIT-only consumers | Viable |
| MPL-2.0 | File-level copyleft; modifications to MPL files must be MPL; patent grant; notices | Allows proprietary larger works; requires sharing modifications to covered files; more complex for contributors | Viable if the owner wants controlled openness |
| GPLv3 | Derivative works must be GPLv3; source offer; patent and anti-tivoization terms | Strong user freedom; prevents proprietary forks; conflicts with proprietary codecs/DRM and some distribution models | Viable only if the whole product strategy is GPL-compatible |
| AGPLv3 | GPLv3 plus network-use source offer | Not useful for local desktop apps; adds confusion | Rejected |
| Proprietary / source-available | Custom terms; no redistribution without permission | Maximum commercial control; limits community and trust; complicates contributions | Possible, but not recommended for the ecosystem vision |

**Recommendation:** release EQwec-owned source under `MIT OR Apache-2.0`, with Apache-2.0 as the default patent-grant option, and keep the EQwec name, logos, and brand assets under separate trademark terms. Use a Developer Certificate of Origin (DCO) for contributions initially. Revisit a CLA only if the owner needs deliberate dual licensing or relicensing flexibility.

**Why not GPLv3 by default:** the browser and media products may need to ship or link proprietary codecs, DRM components, platform SDKs, or commercial libraries. GPLv3 would make those combinations legally difficult or impossible. The project can still publish GPLv3-compatible components separately, but the core license should not force that outcome before the codec/DRM strategy is settled.

**Why not MPL-2.0 by default:** MPL-2.0 is a credible choice for a desktop application, but it adds file-level source-disclosure obligations and is less familiar to Rust contributors than MIT/Apache-2.0. It remains the recommended alternative if the owner wants a copyleft boundary without GPL's linking effects.

**Owner decision required.** Until ADR-0008 is accepted, the repository has no public license and no permission to redistribute source or binaries.

## 4. License families and what they require

### 4.1 Permissive licenses

MIT, BSD-2-Clause, BSD-3-Clause, ISC, and Apache-2.0 allow use in proprietary and open-source products if notices and conditions are met. Apache-2.0 adds an express patent grant and requires preservation of notices; it is incompatible with GPLv2 but compatible with GPLv3. These are the preferred licenses for shared infrastructure.

### 4.2 MPL-2.0

MPL-2.0 is file-level copyleft. EQwec may combine MPL files with proprietary files in a larger work, but modifications to MPL-covered files must remain MPL and their source must be made available. MPL-2.0 includes a patent grant. It is generally compatible with GPLv2+/LGPL when the secondary-license mechanism applies; verify per component and per distribution.

### 4.3 LGPL-2.1 and LGPL-3.0

LGPL allows linking with proprietary code if the user can replace or relink the library, the library's source (including modifications) is available, and notices are provided. Static linking requires additional care (often object files or a relinkable form). LGPLv3 adds patent terms and anti-tivoization requirements. EQwec must choose dynamic linking or a separate-process architecture where practical, and document the compliance method.

### 4.4 GPL-2.0 and GPL-3.0

GPL requires the derivative work to be distributed under GPL and source to be offered. GPLv3 adds patent, anti-tivoization, and installation-information terms. GPL components cannot be combined with proprietary code in the same work without a compatible exception. GPL tools used only at build time do not automatically affect the distributed binary, but generated code and linked libraries can.

### 4.5 CDDL-1.0

CDDL is file-level copyleft and is not GPL-compatible. LibRaw offers LGPL-2.1 or CDDL-1.0; EQwec should choose LGPL-2.1 and comply with its relinking and notice requirements.

### 4.6 Public domain

SQLite is public domain. It has no attribution requirement, but EQwec should still record provenance and include it in the SBOM.

### 4.7 Proprietary and platform licenses

Platform SDKs, codecs, DRM, and commercial frameworks have their own terms. They may restrict redistribution, reverse engineering, benchmarking, or use in virtualized environments. Each requires a separate review; an OS API being free to call does not mean its runtime can be redistributed.

## 5. Dependency inventory

The following table is a planning inventory. "Verify" means the exact version and license must be confirmed at admission. A component is not approved until it appears in `Cargo.lock`, the license inventory, and the SBOM.

### 5.1 UI, rendering, and accessibility

| Component | License | Use | Obligations / notes | State |
|---|---|---|---|---|
| egui / eframe | MIT OR Apache-2.0 | Application UI | Notices; Apache patent grant if chosen | Candidate |
| AccessKit | MIT OR Apache-2.0; portions BSD-3-Clause | Accessibility tree and platform bridges | Notices; preserve Chromium-derived BSD notices | Candidate |
| wgpu | MIT OR Apache-2.0 | GPU abstraction | Notices | Candidate |
| Vello | Apache-2.0 OR MIT | 2D renderer evaluation | Notices; 0.x API risk | Evaluation |
| resvg / usvg | MPL-2.0; newer versions may relicense | SVG import/render evaluation | MPL file-level obligations; verify version | Evaluation |
| tiny-skia | BSD-3-Clause (verify) | CPU raster evaluation | Notices | Evaluation |
| kurbo | Apache-2.0 OR MIT (verify) | Bézier geometry | Notices | Candidate |
| lyon | MIT OR Apache-2.0 (verify) | Tessellation | Notices | Candidate |
| cosmic-text | MIT OR Apache-2.0 (verify) | Text shaping/layout | Notices | Candidate |
| HarfBuzz | MIT-style (verify) | Shaping fallback | Notices | Evaluation |
| FreeType | FTL or GPLv2 | Font raster fallback | Choose FTL; notices | Evaluation |
| fontdb | MIT (verify) | Font discovery | Notices | Candidate |
| Slint | GPLv3; royalty-free desktop/mobile/web; commercial | UI fallback | GPLv3 is viral; royalty-free/commercial terms require review | Fallback |
| Iced | MIT | UI evaluation | Notices | Rejected for v1 |
| Qt 6 | LGPLv3; GPLv3; commercial | UI fallback | LGPL relinking/notices or commercial license; some modules GPLv3-only | Fallback |
| Skia / skia-safe | BSD-3-Clause | Rendering fallback | C++ build; notices | Deferred |

### 5.2 Browser engine

| Component | License | Use | Obligations / notes | State |
|---|---|---|---|---|
| CEF | BSD-3-Clause | Browser engine | Include CEF LICENSE.txt and Chromium third-party notices; source/binary redistribution terms | Candidate |
| Chromium | BSD-3-Clause plus third-party licenses | CEF dependency | Large notice file; codec and DRM components separately licensed | Candidate |
| `cef` crate | Apache-2.0 OR MIT (verify) | Rust binding | Notices; pin version | Candidate |
| WebView2 | Proprietary Microsoft runtime | Windows fallback | Microsoft distribution terms; Evergreen runtime | Fallback |
| WKWebView | Apple OS framework | macOS fallback | Apple SDK terms; no runtime redistribution | Fallback |
| Servo | MPL-2.0 | Watch item | MPL obligations if used | Watch |
| Electron | MIT | Rejected | Not a browser; wrong security model | Rejected |

### 5.3 Media, image, and color

| Component | License | Use | Obligations / notes | State |
|---|---|---|---|---|
| FFmpeg | LGPL-2.1-or-later by default; GPL-2.0-or-later with `--enable-gpl` | Decode/encode/demux | LGPL compliance: dynamic link or subprocess, source offer, notices, no GPL components; exact build flags recorded | Candidate; legal gate |
| libx264 / libx265 | GPL-2.0 / GPL-3.0 | H.264/HEVC encode | Do not ship in the default proprietary build; separate legal review if offered | Excluded by default |
| OpenH264 | BSD-2-Clause plus Cisco patent terms (verify) | H.264 encode alternative | Patent license conditions; binary distribution terms | Evaluation |
| libaom / dav1d / SVT-AV1 | BSD-style (verify) | AV1 encode/decode | AOMedia patent policy; notices | Candidate |
| Symphonia | MPL-2.0 | Audio decode | MPL file-level obligations | Candidate |
| LibRaw | LGPL-2.1 or CDDL-1.0 | Camera RAW decode | Choose LGPL-2.1; dynamic link; notices | Candidate |
| RawSpeed | LGPL-2.1 (verify) | RAW decode alternative | LGPL compliance | Evaluation |
| OpenColorIO | BSD-3-Clause | Color management evaluation | Notices; C++ dependency | Evaluation |
| LittleCMS / `lcms2` | MIT | ICC color management | Notices | Candidate |
| `image` | MIT OR Apache-2.0 | Raster formats | Notices | Candidate |
| `exr` | BSD-3-Clause (verify) | OpenEXR | Notices | Candidate |
| `png`, `jpeg-decoder`, `image-webp` | MIT OR Apache-2.0 (verify) | Format codecs | Notices | Candidate |
| `ravif` / `rav1e` | BSD-3-Clause (verify) | AVIF encode | Notices; patent policy | Evaluation |
| zlib / `flate2` | Zlib OR Apache-2.0/MIT | Compression | Notices | Candidate |
| zstd | BSD-3-Clause OR GPLv2 | Compression | Choose BSD-3-Clause; notices | Candidate |

### 5.4 Audio and plugins

| Component | License | Use | Obligations / notes | State |
|---|---|---|---|---|
| CPAL | MIT | Cross-platform audio I/O | Notices | Candidate |
| CoreAudio / WASAPI / ALSA / PipeWire / JACK | OS or LGPL/MIT (verify) | Platform audio backends | Platform terms; JACK LGPL compliance if linked | Candidate |
| CLAP | MIT | Plugin format | Notices; open standard | Candidate |
| VST3 SDK | MIT since 3.8 | Plugin format | MIT notice; Steinberg trademark guidelines if VST marks are used | Candidate |
| LV2 | ISC | Plugin format | Notices | Later |
| AAX | Proprietary, paid | Plugin format | Not supported initially | Rejected |
| `midir` | MIT | MIDI I/O | Notices | Candidate |
| `wmidi` | MIT (verify) | MIDI messages | Notices | Candidate |
| `rtrb` | MIT OR Apache-2.0 (verify) | Lock-free audio queues | Notices | Candidate |
| `rubato` | MIT | Resampling | Notices | Candidate |

### 5.5 Data, crypto, IPC, and diagnostics

| Component | License | Use | Obligations / notes | State |
|---|---|---|---|---|
| SQLite | Public domain | Catalogs and caches | Provenance record; no attribution required | Candidate |
| `rusqlite` | MIT | SQLite binding | Notices | Candidate |
| `redb` | MIT OR Apache-2.0 (verify) | Embedded store alternative | Notices | Evaluation |
| `serde`, `serde_json` | MIT OR Apache-2.0 | Serialization | Notices | Candidate |
| `zip` | MIT | Single-file project package | Notices; bounded parsing | Candidate |
| `sha2`, `ed25519-dalek`, `ring` | MIT/Apache-2.0; BSD-3-Clause; ISC | Hashing and signatures | Notices; security review; no homegrown crypto | Candidate |
| `tracing`, `tracing-subscriber`, `tracing-appender` | MIT | Diagnostics | Notices; redaction required | Candidate |
| `interprocess` or platform IPC | MIT OR Apache-2.0 (verify) | Local IPC | Notices; peer credential validation | Candidate |

### 5.6 Build, test, and release tooling

| Component | License | Use | Obligations / notes | State |
|---|---|---|---|---|
| `cargo-nextest` | MIT OR Apache-2.0 | Test runner | Build-time tool | Candidate |
| `proptest` | MIT OR Apache-2.0 | Property tests | Build-time tool | Candidate |
| `cargo-fuzz` | MIT OR Apache-2.0 | Fuzzing | Build-time tool | Candidate |
| `criterion` | MIT OR Apache-2.0 | Benchmarks | Build-time tool | Candidate |
| `insta` | Apache-2.0 | Snapshot tests | Build-time tool | Candidate |
| `cargo-audit` | MIT OR Apache-2.0 | Advisories | Build-time tool | Candidate |
| `cargo-deny` | MIT OR Apache-2.0 | License/advisory policy | Build-time tool | Candidate |
| `cargo-about` | MIT OR Apache-2.0 | Notices generation | Build-time tool | Candidate |
| `cargo-cyclonedx` | Apache-2.0 | SBOM | Build-time tool | Candidate |
| WiX Toolset | MS-RL (verify) | Windows MSI | Build-time tool; check redistribution of custom actions | Candidate |
| Flatpak | LGPL-2.1+ (verify) | Linux sandboxed packaging | Build/runtime terms | Candidate |
| AppImage | MIT (verify) | Linux portable package | Build/runtime terms | Candidate |
| minisign | ISC (verify) | Artifact signatures | Build-time tool | Candidate |
| Playwright | Apache-2.0 | Browser/web tests | Build-time tool; browser binaries have separate terms | Candidate |

## 6. Compatibility analysis

### 6.1 Permissive core

A project licensed `MIT OR Apache-2.0` can incorporate MIT, BSD, ISC, Apache-2.0, MPL-2.0, LGPL, and (if the whole work is GPL-compatible) GPL components only under the applicable obligations. Permissive dependencies do not force source disclosure of EQwec code.

### 6.2 MPL-2.0 dependencies

MPL-2.0 files can be used in a proprietary application if:

- the MPL-covered files remain under MPL-2.0;
- modifications to those files are made available in source form;
- notices are preserved;
- the larger work does not impose additional restrictions on the MPL files.

EQwec should keep MPL dependencies behind a crate boundary and avoid modifying them unless necessary. If modified, publish the changes.

### 6.3 LGPL dependencies

LGPL-2.1/3 dependencies require a relinking path. Acceptable approaches:

1. Dynamic linking with the shared library shipped alongside the application.
2. A separate process (for example, FFmpeg CLI) with a documented interface.
3. Static linking only with object files or another compliant relinking mechanism.

The release must include the library's license, source offer (including modifications), build instructions, and any notices. Do not strip the ability to replace the library.

### 6.4 GPL dependencies

GPL components are excluded from the default proprietary build unless the entire distributed work is GPL-compatible. This affects:

- `libx264` and `libx265` (GPL);
- GPL-only FFmpeg components;
- GPL applications or libraries that cannot be isolated as separate processes without creating a combined work (the boundary depends on facts and jurisdiction).

A separate optional download of a GPL component may still create obligations; counsel must approve the model.

### 6.5 Codec patents

Copyright licenses do not grant patent licenses. H.264, AAC, HEVC, and some other codecs may require patent pool licenses even when the software is open source. The project must not assume that an LGPL/BSD codec implementation is patent-free.

## 7. Codec and DRM review

### 7.1 Default codec set

The default build should use royalty-free formats where the product allows:

| Format | Patent/royalty status | Use |
|---|---|---|
| PNG | Royalty-free | Images |
| JPEG | Patents expired (verify) | Images |
| WebP | Google royalty-free | Images |
| AVIF / AV1 | AOMedia royalty-free policy | Images/video |
| VP9 | Google royalty-free | Video |
| Opus | IETF royalty-free | Audio |
| FLAC | Royalty-free | Audio |
| Ogg Vorbis | Royalty-free | Audio fallback |

### 7.2 Legal-gated codecs

| Format | Risk | Required action |
|---|---|---|
| H.264/AVC | Patent pools; decode/encode licensing | Counsel review; record whether a license is needed; do not ship by default until resolved |
| AAC | Patent pool | Same as H.264 |
| HEVC/H.265 | Multiple patent pools; higher risk | Counsel review; likely excluded from the first release |
| ProRes | Apple proprietary format and licensing | Counsel and Apple terms; decode-only may still carry risk |
| DNxHR/DNxHD | Avid proprietary format | Counsel and Avid terms |
| MP3 | Patents expired (verify) | Verify; likely safe |
| JPEG XL | Standards/patent status must be verified | Do not claim royalty-free without evidence |

### 7.3 DRM

Widevine (Google), FairPlay (Apple), and PlayReady (Microsoft) require separate agreements, device provisioning, robustness rules, and privacy review. They are **not** part of the first release. A browser or video product that requires streaming-service DRM must complete this review before claiming compatibility.

### 7.4 Build flags and evidence

Every release must record:

- the exact FFmpeg version and `configure` flags;
- whether `--enable-gpl`, `--enable-nonfree`, or proprietary codecs were enabled;
- the resulting codec list;
- the corresponding notices and source offer;
- the patent-license decision for each shipped codec.

A build that silently enables a GPL or proprietary component is a release blocker.

## 8. Content, fonts, icons, and trademarks

### 8.1 Fonts

- Use fonts under the SIL Open Font License, Apache-2.0, or a compatible license.
- Preserve copyright and license notices; respect Reserved Font Names.
- Do not embed proprietary fonts without a distribution license.
- Record font files, versions, and licenses in the SBOM.

### 8.2 Icons and images

- Use original EQwec icons or assets under a compatible license.
- CC-BY requires attribution; CC0 is public domain; proprietary assets require a license.
- Do not copy icons, illustrations, or UI assets from competitors.
- Record provenance for every bundled asset.

### 8.3 Sample media and fixtures

- Use original, generated, or licensed test media.
- Do not commit copyrighted music, footage, photos, or fonts to the repository.
- Mark fixtures with their license and source.

### 8.4 Trademarks

- EQwec names and logos require a separate trademark policy; the source license does not grant trademark rights.
- Do not use third-party logos or names in a way that implies endorsement.
- VST is a Steinberg trademark; usage must follow Steinberg's guidelines.
- Chromium, CEF, Apple, Microsoft, Windows, macOS, and Linux names are subject to their owners' policies.
- Product names must be checked for conflicts before public release.

## 9. Distribution obligations and SBOM

Before any public release:

1. Generate an SBOM in SPDX or CycloneDX format for every artifact.
2. Generate a third-party notices file from the actual lockfile and bundled binaries.
3. Include CEF/Chromium, FFmpeg, Qt/Slint (if used), font, icon, and sample-media notices.
4. Provide LGPL source offers and relinking information where required.
5. Provide MPL source for modified MPL files.
6. Record the exact build flags, versions, and hashes.
7. Verify that no GPL/proprietary component is present in a build that claims to be permissive-only.
8. Keep the notices and SBOM with the release artifact, not only in the repository.
9. Review the obligations with counsel before the first public binary.

## 10. Open questions and blockers

| Item | State | Owner / unblock |
|---|---|---|
| Project license | `PROPOSED` | Owner approval (ADR-0008) |
| H.264/AAC patent licensing | `BLOCKED` | Counsel; decide whether to ship, license, or omit |
| HEVC, ProRes, DNxHR | `BLOCKED` | Counsel and vendor terms |
| DRM agreements | `BLOCKED` | Commercial agreements and privacy review |
| FFmpeg LGPL compliance model | `BLOCKED` | Choose dynamic linking vs subprocess; legal review |
| Qt/Slint commercial or copyleft choice | `BLOCKED` | Only relevant if selected as fallback |
| CEF/Chromium notices and codec build | `UNVERIFIED` | Prototype build and notice audit |
| Font/icon/sample-media licenses | `UNVERIFIED` | Asset inventory |
| Contributor policy (DCO vs CLA) | `PROPOSED` | Owner and counsel |
| Trademark policy and name clearance | `BLOCKED` | Owner and counsel |
| Export-control review | `UNVERIFIED` | Counsel; cryptography and DRM may be affected |
| Proprietary format interoperability (PSD, AI, etc.) | `UNVERIFIED` | Legal and technical review; no reverse engineering of protected formats |

## 11. Release rule

A release is forbidden if:

- the project license is not selected;
- a shipped dependency has no recorded license or notice;
- an LGPL/MPL/GPL obligation is unresolved;
- a codec or DRM is shipped without the required patent or commercial decision;
- the SBOM and notices do not match the artifact;
- a bundled asset has no provenance;
- a trademark use is unresolved.

These are legal gates, not engineering preferences. A waiver requires the project owner and counsel, with residual risk recorded.

## 12. Sources

All sources were accessed on 2026-11-07.

1. FFmpeg legal and license page. https://www.ffmpeg.org/legal.html
2. GNU LGPL 2.1. https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html
3. GNU GPL 3.0. https://www.gnu.org/licenses/gpl-3.0.html
4. Apache License 2.0. https://www.apache.org/licenses/LICENSE-2.0
5. Mozilla Public License 2.0. https://www.mozilla.org/en-US/MPL/2.0/
6. CEF license and README. https://github.com/chromiumembedded/cef/blob/master/LICENSE.txt
7. Chromium third-party licenses. https://chromium.googlesource.com/chromium/src/+/main/LICENSE
8. VST 3 licensing. https://steinbergmedia.github.io/vst3_dev_portal/pages/VST%2B3%2BLicensing/VST3%2BLicense.html
9. CLAP repository. https://github.com/free-audio/clap
10. LV2 specification. https://lv2plug.in/
11. LibRaw licensing. https://www.libraw.org/
12. OpenColorIO license. https://opencolorio.readthedocs.io/en/latest/aswf/license.html
13. SQLite copyright. https://www.sqlite.org/copyright.html
14. Slint licensing. https://slint.dev/pricing
15. Qt licensing. https://doc.qt.io/qt-6/licensing.html
16. Qt LGPL obligations. https://www.qt.io/development/open-source-lgpl-obligations
17. AccessKit license. https://github.com/AccessKit/accesskit
18. resvg/usvg license. https://github.com/linebender/resvg
19. Vello license. https://github.com/linebender/vello
20. wgpu license. https://crates.io/crates/wgpu
21. egui license. https://crates.io/crates/egui
22. The Update Framework specification. https://theupdateframework.github.io/specification/latest/
23. SPDX license list. https://spdx.org/licenses/
24. AOMedia patent policy. https://aomedia.org/license/
25. Via LA licensing (formerly MPEG LA). https://www.via-la.com/
26. Access Advance HEVC licensing. https://accessadvance.com/
27. W3C Patent Policy. https://www.w3.org/Consortium/Patent-Policy/
28. SIL Open Font License. https://openfontlicense.org/
29. Creative Commons licenses. https://creativecommons.org/licenses/
30. Steinberg VST trademark guidelines. https://steinbergmedia.github.io/vst3_dev_portal/pages/VST%2B3%2BLicensing/VST3%2BLicense.html
