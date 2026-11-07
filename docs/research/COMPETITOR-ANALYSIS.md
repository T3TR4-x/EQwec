# Competitor Analysis

**Status:** Planning research. This document compares documented competitor behavior; it is not a product commitment, a legal opinion, or a compatibility guarantee.

**Date:** 2026-11-07

**Scope:** EQwec Hub, Browser, PhotoCraft, VectorCraft, FilmCraft, EffectCraft, CraftAnything, and AudioCraft.

**Evidence rule:** `OBSERVED` means the behavior is stated in vendor documentation, a vendor manual, release notes, or a public vendor feature page. `ASSUMPTION` means an inference made by this analysis and is marked inline. No performance comparison is made: vendor speed claims are excluded unless a cited benchmark states its workload and environment, and none were used here.

**Accessibility rule:** Only documented vendor statements are recorded. `UNVERIFIED` means no official conformance statement was located during this review; it does not mean the product is inaccessible.

## 1. Method and limits

- Sources are official vendor pages, manuals, release notes, and specifications. Each source is listed in Section 15 with access date `2026-11-07`.
- Some Adobe Help Center pages and some vendor documentation returned HTTP 403 to automated retrieval. Facts from those areas are limited to details corroborated by accessible official pages; unconfirmed details are omitted or marked `ASSUMPTION`.
- “Supports format X” below means the vendor documents import or export of X. It does not mean EQwec has verified round-trip fidelity, layer preservation, color accuracy, or version compatibility.
- Pricing and licensing change frequently. Prices are included only where the vendor’s official page states them at access time; they are comparison context, not EQwec pricing.
- Product naming follows current vendor usage. “Affinity” refers to the single Canva-owned application that replaced the former Affinity Photo, Designer, and Publisher products.
- This is a desktop-professional comparison. Mobile-first editors, browser-only editors, and consumer cloud editors are out of scope unless they establish a workflow that EQwec must match.

**Assumptions register**

- `A1`: Users prefer local-first operation and portable data over account-bound sync. This is an inference from the prominence of export/import features in Hub and Browser products; it is not a measured user preference.
- `A2`: Proprietary format round-trip will remain partial without vendor cooperation. This follows from documented limits such as Illustrator’s artboard-scoped SVG export and PSD’s size limits, but the exact loss depends on each file.
- `A3`: Vendor accessibility conformance reports are inputs to EQwec testing, not proof that EQwec’s own interface will be accessible.
- `A4`: Performance comparisons are meaningful only with a stated workload and environment; EQwec will not publish comparative performance claims without that evidence.
- `A5`: The “professional workflow” differences described here are hypotheses for user research, not validated personas.

Matrix capability rows are `OBSERVED` only where a cited vendor source documents the capability. Interaction characterizations, workflow labels, and comparative judgments are `ASSUMPTION` unless the vendor states them explicitly. Accessibility entries are `UNVERIFIED` unless a conformance document is cited. Opportunities, risks, and recommendations are analysis.

## 2. Cross-family summary

| Family | Mature interaction pattern | Recurring weakness | EQwec opening |
|---|---|---|---|
| Hub | Library plus install/update loop, optional cloud saves or community content | Platform silos, weak rollback, account coupling, incomplete lockfiles | Signed offline-first installs, atomic rollback, declarative lockfile |
| Browser | Tabs, profiles/containers, sync, extensions, resource management | Data portability at vendor exit, extension trust, memory/energy tradeoffs | Portable profile archive, explicit sunset guarantee, transparent resource policy |
| Photo | Layer stack or parametric catalog, non-destructive edits, RAW pipeline | Proprietary exchange formats, subscription coupling, weak round-trip fidelity | Dual-mode document, PSD compatibility tiers, open adjustment sidecar |
| Vector | Bézier editing, artboards/pages, SVG and PDF interchange | SVG feature loss, proprietary native formats, print/web divergence | SVG-native core with explicit fidelity reporting and print color |
| Film | Timeline editing, ingest/proxy, render queue, interchange | Format fragmentation, storage pressure, collaboration lock-in | OTIO-native timeline, deterministic render graph, content-addressed proxies |
| Effect | Layer/composition or node graph, templates, expressions | Render lock-in, steep node workflows, subscription lapse limits | Hybrid layer/node authoring with an open template format |
| CraftAnything | Asset libraries, metadata, references, provenance | Ecosystem silos, link rot, weak review workflows | Content-addressed exchange and provenance, not another editor |
| Audio | Session/arrangement views, mixer, plugin hosting, automation | Plugin compatibility, real-time risk, proprietary projects | CLAP-first sandboxed hosting, real-time-safe graph, open archive |

The repeated lesson is that professional users do not buy a feature list; they buy a dependable document lifecycle. Import, edit, recover, collaborate, export, and migrate are the product. EQwec should compete on that lifecycle rather than on a longer checklist.

## 3. Hub

Hub is EQwec’s trust boundary: it installs, updates, repairs, and removes software. A defect here can execute untrusted code, destroy user data, or strand a machine. The comparison therefore weights signing, rollback, offline behavior, and export more heavily than catalog size.

| Product | Licensing model | Documented capabilities | Interaction / workflow | Formats / interchange | Documented accessibility |
|---|---|---|---|---|---|
| Steam | Free client; store purchases | Library, automatic updates, Steam Cloud saves, Workshop community content, friends, screenshots | Library-centric GUI with background updates | Steam Cloud API; Workshop items; proprietary client data | `UNVERIFIED` for the client |
| GOG Galaxy | Free client | Unified library through official and community integrations, cloud saves, rollback, custom views | Library-centric GUI with integrations | Integrations, cloud saves, proprietary client data | `UNVERIFIED` |
| WinGet | Free, open-source Microsoft tool | Install, upgrade, uninstall, export, import, pin, repair, configure; many installer types | CLI-first with manifest repository | YAML manifests; export/import files | `UNVERIFIED` |
| Microsoft Store | Free client; paid apps and subscriptions | App discovery, install, update, library, family controls, packaged Win32 and MSIX apps | GUI store; updates through the Store | MSIX/APPX and packaged Win32; Store-managed updates | Microsoft documents Windows accessibility; Store client conformance `UNVERIFIED` |
| JetBrains Toolbox | Free client; product subscriptions | Install, update, roll back, keep multiple product versions, project list, license management | GUI/CLI hybrid; per-product install channels | Proprietary install metadata; product-specific binaries | `UNVERIFIED` |
| Homebrew | Free, open-source | Declarative Brewfile install/upgrade, dump, cleanup, services; formulae, casks, MAS, WinGet, VS Code, Cargo, npm, uv, Flatpak | CLI-first, project-oriented | Brewfile (Ruby DSL); explicitly no lockfile | `UNVERIFIED` |
| Flatpak | Free, open-source | Sandboxed desktop apps, rootless install, decentralized remotes, delta updates, runtimes | CLI and desktop software centers | Flatpak refs, runtimes, portals | `UNVERIFIED` |
| Adobe Creative Cloud | Subscription | Launches and updates apps, syncs assets, manages fonts, Stock, Behance, collaboration | GUI-first account hub | Proprietary; Adobe cloud services | Adobe publishes accessibility reporting, but the desktop hub is `UNVERIFIED` |

**Observed patterns**

- The durable core is library plus update loop; discovery and social features are secondary. Steam, GOG Galaxy, WinGet, Homebrew, and Adobe CC all organize around “what is installed and what changed” [S1][S3][S4][S5][S7].
- Declarative export is valued, but fidelity is uneven. WinGet documents export/import, and Homebrew documents `brew bundle dump`; Homebrew explicitly states it has no lockfile [S4][S5]. Its FAQ also states that it does not support installing arbitrary older versions [S77].
- Sandboxing is architectural, not a settings toggle. Flatpak documents rootless installation, sandboxed apps, and portal-mediated access [S6].
- Community content needs a trust model. Steam Workshop supports ready-to-use and curated item models, which implies review, ownership, and versioning decisions rather than a simple download button [S2].
- Account coupling is the normal failure mode: cloud saves, asset sync, and license checks can make a local install depend on a remote identity.

**Documented limitations**

- Homebrew is a rolling-release package manager: it does not install arbitrary older versions and will not add a Brewfile lockfile [S5][S77].
- Flatpak cannot run SUID binaries, requires a host `/proc`, and cannot provide kernel modules; these are deliberate sandbox boundaries, not bugs [S78].
- Steam Cloud and Workshop impose API and quota rules on developers; “cloud sync” is not an unbounded filesystem [S1][S2].
- Adobe CC ties app lifecycle and asset sync to an Adobe account and subscription [S7].

**EQwec opportunities**

- Make the install transaction the differentiator: signed manifest, content digest, atomic staging, verification before activation, rollback, repair, and a machine-readable audit log.
- Offer a declarative environment export with a real lockfile: exact versions, digests, source, license, and platform constraints. This is a direct improvement over Brewfile’s documented lack of lockfile support [S5].
- Support offline install and offline repair from a signed local bundle, with no account required for local use.
- Publish a per-app permission and sandbox manifest, and show users exactly what an installer will touch before it runs.

## 4. Browser

Browser is EQwec’s highest-maintenance product. The differentiator is not rendering; it is safe application logic, profile isolation, data portability, and a credible security-update cadence on top of a maintained Chromium foundation.

| Product | Licensing model | Documented capabilities | Interaction / workflow | Formats / interchange | Documented accessibility |
|---|---|---|---|---|---|
| Arc | Free | Sidebar tabs, Spaces, Profiles, pinned tabs and folders, Split View | Sidebar-first, workspace-oriented | Vendor data export before sunset | `UNVERIFIED` |
| Brave | Free, open-source | Shields, vertical tabs, sidebar, sync, Tor private windows | Tab strip plus sidebar, privacy controls | Bookmarks, history, sync | `UNVERIFIED` |
| Chrome | Free | Memory Saver levels, inactive-tab appearance, site exclusions, Energy Saver, profiles | Conventional tab strip, profile switching | Bookmarks; sync | Chrome documents accessibility features; conformance is `UNVERIFIED` |
| Edge | Free | Vertical tabs, workspaces, sleeping tabs, profiles, collections, sidebar | Conventional tab strip plus vertical tabs and workspaces | Favorites, collections; sync | Microsoft documents Windows accessibility; Edge conformance `UNVERIFIED` |
| Firefox | Free, open-source | Vertical tabs and sidebar in Firefox 136; Containers isolate cookies, localStorage, indexedDB, and cache by `userContextId` | Conventional tabs, sidebar, container assignment | Bookmarks, history; sync | `UNVERIFIED` |
| Safari | Bundled with Apple platforms | Profiles, tab groups, iCloud sync, Apple Intelligence topic grouping | Conventional tabs, profile windows, tab groups | Bookmarks, history, iCloud tabs | Apple documents system-level accessibility; app-specific conformance is `UNVERIFIED` |
| Vivaldi | Free | Two-level tab stacks, hibernation, tiling, workspaces, sessions, web panels, profiles, mail/calendar/feed reader, command chains | Highly configurable tab strip, panels, workspaces | Sessions, bookmarks, notes | `UNVERIFIED` |

**Observed patterns**

- Tab overload is the main interaction battleground: vertical tabs, two-level stacks, workspaces, sessions, hibernation, and tab groups all attack the same problem from different angles [S9][S12][S14][S15].
- Profile and container isolation is becoming table stakes. Safari documents separate profiles; Firefox documents container-scoped storage; Chrome documents multiple profiles [S11][S13][S14].
- Resource management is now a visible product feature: Chrome exposes Memory Saver levels, inactive-tab appearance, site exclusions, and Energy Saver [S10].
- Vendor exit is a real product risk. Arc’s published sunset date is 2026-12-22, with account and data deletion and a data-export path; its successor carries Spaces, Profiles, sidebar tabs, pinned tabs, folders, tab groups, and Split View [S8].

**Documented limitations**

- Arc’s sunset notice makes the cost of account-bound browsing data concrete: users must export before deletion, and the successor is a different product [S8].
- Firefox Containers isolate site data by container identity; the documentation describes storage isolation, not a separate operating-system process boundary [S13].
- Chrome Memory Saver discards inactive tabs to reclaim memory; users must exclude sites where discard is unacceptable [S10].
- Safari profiles separate history, cookies, website data, extensions, tab groups, and favorites, but passwords stored in iCloud Keychain remain available across profiles [S14].

**EQwec opportunities**

- Ship a portable profile archive: tabs, workspaces, history, bookmarks, extensions list, permissions, and settings in a documented, versioned format with integrity checks.
- Publish a sunset guarantee: if the product or a cloud service ends, a local export tool and format specification remain available.
- Make tab discard and resource policy transparent and per-profile, with an allowlist and a visible “why was this tab discarded?” explanation.
- Build an extension permission dashboard with signed updates, capability scopes, and an audit trail.
- Treat keyboard-first tab management and screen-reader-tested profile switching as release gates, not later polish.

## 5. Photo

PhotoCraft must choose a document model before it chooses features. The market splits between layer-stack editors and parametric catalog editors; both claim non-destructive editing, but they store different things and fail in different ways.

| Product | Licensing model | Documented capabilities | Interaction / workflow | Formats / interchange | Documented accessibility |
|---|---|---|---|---|---|
| Photoshop | Subscription | Layers, masks, smart objects, adjustment layers, Camera Raw, compositing, text, filters | Layer stack, canvas-first, panels | PSD/PSB; PDF, TIFF, PNG, JPEG and others | Adobe publishes an ACR for Photoshop |
| Lightroom Classic | Subscription | Non-destructive develop adjustments, catalog, masking, print, slideshow, web | Catalog-first, module-based | Proprietary catalog; XMP sidecars; PSD/TIFF/JPEG export | Adobe publishes an ACR for Lightroom |
| Lightroom (cloud) | Subscription | Cloud-synced non-destructive editing, AI masking, presets, web/mobile/desktop clients | Cloud library, edit across devices | Proprietary cloud library; DNG/JPEG/TIFF export | Adobe publishes an ACR; cloud client conformance `UNVERIFIED` |
| Affinity | Free with Canva account | Vector, pixel, and layout studios in one app; non-destructive adjustments; AI features gated behind paid Canva | Studio switching, canvas-first | `.af`; PSD, SVG, PDF and image export | `UNVERIFIED` |
| GIMP | GPL free software [S22] | Layers, masks, selections, brushes, non-destructive filters in 3.0, PSD export | Layer stack, canvas-first | XCF; PSD, PNG, JPEG, TIFF and others | GIMP publishes a VPAT [S76] |
| Krita | GPL free software | Brush engines, wrap-around painting, animation, PSD support, OCIO/OpenEXR HDR, Python | Canvas-first, brush-centric | `.kra`; PSD, OpenEXR, PNG and others | `UNVERIFIED` |
| darktable | GPLv3 free software | Scene-referred workflow, filmic tone mapping, module ordering, database-backed non-destructive editing | Darkroom plus lighttable, module pipeline | RAW plus XMP sidecars; TIFF, PNG, JPEG export | `UNVERIFIED` |
| Capture One | Subscription and perpetual options | Tethering, custom camera profiles, sessions and catalogs, assisted masking, color editing | Session or catalog, capture-first | Proprietary session/catalog; TIFF, PSD, JPEG and others | `UNVERIFIED` |

**Observed patterns**

- The layer-stack model (Photoshop, GIMP, Krita, Affinity) exposes editable objects and a visible stack; the parametric model (Lightroom Classic, darktable, Capture One) stores instructions and a history over a source image [S17][S20][S23][S24].
- Non-destructive editing is now expected across price points, but the implementation differs: smart objects and adjustment layers versus a database of instructions [S17][S20][S24].
- RAW processing, color management, and tethering are professional differentiators more than brush count [S24][S25].
- PSD remains the de facto exchange format even though it is proprietary and does not represent every editor’s model [S16].
- GIMP 3.2.8 was the current release at access time; the 3.0 release introduced the non-destructive filter model discussed here [S20][S21].

**Documented limitations**

- PSD is limited to 30,000 pixels in either dimension and 2 GB; PSB raises the dimension limit to 300,000 pixels but remains an Adobe-defined format [S16].
- Affinity requires a Canva account for activation, and its AI features are gated behind a paid Canva plan; local editing works offline after activation [S18][S19].
- Lightroom Classic’s catalog is the source of truth for edits; the files on disk are not the complete document [S17].
- GIMP 3.0 documents improved PSD export and non-destructive filters, but the vendor does not claim lossless PSD round-trip fidelity [S20].

**EQwec opportunities**

- Build a dual-mode document: a layer stack for compositing and a parametric instruction history for adjustments, both stored in one open project archive.
- Define PSD compatibility tiers (view, edit, round-trip) and publish a fidelity report per file: layers, masks, smart objects, text, adjustment layers, color profiles.
- Store adjustments in an open sidecar with stable IDs, so another EQwec app or a future version can re-render without the original application.
- Make tethering and capture crash-safe: write-ahead capture, checksummed originals, and session recovery.
- Publish an accessibility conformance report and test masking and color picking with keyboard-only and screen-reader workflows.

## 6. Vector

VectorCraft’s promise is SVG interoperability, but professional vector work also needs print color, artboards, text, and non-destructive effects. The comparison shows how often “SVG support” means “export a flattened approximation.”

| Product | Licensing model | Documented capabilities | Interaction / workflow | Formats / interchange | Documented accessibility |
|---|---|---|---|---|---|
| Illustrator | Subscription | Artboards, Bézier editing, mesh, symbols, effects, text, print production | Canvas-first, panels, artboard navigation | `.ai`, `.pdf`, `.eps`, `.ait`, `.svg`, `.svgz` | Adobe publishes an ACR; its Illustrator report documents major gaps for screen-reader use |
| Affinity | Free with Canva account | Vector, pixel, and layout studios; non-destructive adjustments; SVG and PDF export | Studio switching, canvas-first | `.af`; SVG, PDF, EPS and image export | `UNVERIFIED` |
| Inkscape | GPL free software | SVG-native editing, Live Path Effects, node editing, extensions, 1.4.x releases | Canvas-first, XML-aware, keyboard-heavy | SVG, SVGZ, PDF, EPS, PNG and others | `UNVERIFIED` |
| CorelDRAW | Subscription and one-time purchase | Vector illustration, page layout, PowerTRACE, mesh fill, multipage documents, broad format support | Canvas-first, dockers, page navigation | Native CDR plus PDF, SVG, AI, EPS and others | `UNVERIFIED` |

**Observed patterns**

- SVG is the interchange lingua franca, but it is not a lossless professional document format. Illustrator documents that SVG export saves the active artboard only and rasterizes mesh objects [S26].
- Artboards or pages plus print color (CMYK, spot colors, overprint) separate professional vector tools from web SVG editors [S26][S27].
- Live effects and boolean operations are often non-destructive and are exactly the features most likely to be lost on round-trip [S28].
- Licensing remains a differentiator: CorelDRAW still offers a one-time purchase alongside subscription, while Affinity moved to a free app with a Canva account [S18][S27].

**Documented limitations**

- Illustrator’s SVG export is artboard-scoped and can rasterize mesh content; it is not a full document export [S26].
- Inkscape’s native model is SVG plus its own extensions; files that depend on Live Path Effects or Inkscape-specific metadata may not render identically elsewhere [S28].
- CorelDRAW’s broad format compatibility is proprietary and version-dependent; the vendor’s list is not a guarantee of editable round-trip [S27].
- Affinity’s `.af` document is proprietary, and the application requires a Canva account for activation [S18][S19].

**EQwec opportunities**

- Make SVG the native document format, with explicit “web-safe” and “print” export profiles and a machine-readable loss report.
- Implement CMYK, spot colors, overprint, and PDF/X export as first-class features, not plugins.
- Store non-destructive booleans and path effects in the project archive so they remain editable after reopen.
- Provide an import fidelity dashboard for AI, SVG, PDF, and EPS: what was preserved, approximated, rasterized, or dropped.
- Support accessible SVG output: title, description, reading order, and focusable elements, plus keyboard-only node editing.

## 7. Film

FilmCraft is an NLE, so the timeline is the product. The market differs less on codecs than on timeline interaction, ingest/proxy discipline, collaboration, and how editorial data leaves the application.

| Product | Licensing model | Documented capabilities | Interaction / workflow | Formats / interchange | Documented accessibility |
|---|---|---|---|---|---|
| DaVinci Resolve | Free edition plus paid Studio edition | Media, Photo, Cut, Edit, Fusion, Color, Fairlight, Deliver pages; editing, color, VFX, audio, delivery | Page-based, track timeline, node color | Project database; AAF, XML, EDL and media formats | `UNVERIFIED` |
| Final Cut Pro | One-time purchase or Apple Creator Studio subscription | Magnetic timeline, Magnetic Mask, automatic captions, Visual Search, Transcript Search | Magnetic timeline, clip connections, roles | FCPXML; ProRes and other media | Apple documents system-level accessibility; app-specific conformance is `UNVERIFIED` |
| Premiere Pro | Subscription | Text-Based Editing, ingest and proxy workflows, Productions, Team Projects, color and audio | Track timeline, panels, text-based editing | FCP XML, AAF, EDL, XML and media formats | Adobe publishes an ACR; documentation notes that full functionality may not be available with assistive technology |
| Avid Media Composer | Subscription; Enterprise tier for some shared-storage workflows | Database-driven projects, bin locking, optional ScriptSync and PhraseFind add-ons, Pro Tools interoperability | Bin-and-clip model, track timeline, script-based editing | AAF, OMF, EDL and media formats | `UNVERIFIED` |
| Shotcut | Free, open-source | FFmpeg format support, MLT XML projects, proxy editing, filters, 10-bit/HDR | Track timeline, panel-based | MLT XML; FFmpeg-supported media | `UNVERIFIED` |
| Kdenlive | Free, open-source | Proxy editing, nested timelines, Glaxnimate integration, AI speech-to-text, effects | Track timeline, project bins | `.kdenlive` XML/MLT; FFmpeg-supported media | `UNVERIFIED` |

**Observed patterns**

- Timeline interaction is the real product: magnetic timelines, track-based timelines, and database-driven bin workflows produce different editing habits and different training costs [S29][S30][S34].
- Ingest and proxy workflows are mandatory for professional media; Premiere documents them as a first-class path, and Shotcut and Kdenlive expose proxy editing directly [S31][S33][S36].
- Interchange is plural: FCPXML, AAF, OMF, EDL, XML, and OTIO each cover a different slice of editorial data [S30][S32][S34][S53].
- Collaboration models diverge sharply: Productions and Team Projects, Avid bin locking, and shared-storage rules are not interchangeable [S34][S35].
- Captioning and transcript-based editing are moving from accessibility features to core editing features [S30][S32].
- Project files are not portable by default: Kdenlive documents its own project and asset-management model, and each NLE expects its own media-relink workflow [S37].

**Documented limitations**

- FCPXML is Apple-defined and versioned; it is not a universal project format, and round-trip to other NLEs is partial [S30]. `ASSUMPTION`: the partial round-trip is a consequence of differing timeline models, not a defect in any one product.
- From 2026.8, Avid’s third-party storage bin locking requires Media Composer Enterprise; shared-storage workflows are a licensing boundary, not only a technical one [S35].
- DaVinci Resolve’s free edition has documented format, frame-rate, and resolution limits; Studio adds 10-bit support, higher frame rates, AI tools, and more effects [S29].
- Shotcut and Kdenlive inherit codec and container support from FFmpeg/MLT builds, so “supports format X” depends on the shipped build [S31][S36].

**EQwec opportunities**

- Use OTIO as the internal timeline model and as an interchange path, with lossless round-trip fixtures for clips, tracks, transitions, markers, and metadata [S53].
- Build a deterministic render graph with resumable jobs, content-addressed inputs, and a render manifest that can be reproduced on another machine.
- Make proxy generation content-addressed and relinkable, so moving or renaming media does not destroy the edit.
- Provide a collaboration model with explicit locks, change sets, and conflict resolution rather than silent last-writer-wins.
- Treat transcript-based editing as both a workflow feature and an accessibility path.

## 8. Effect and compositing

EffectCraft must choose between layer-based compositing, node-based compositing, or a hybrid. The choice determines the learning curve, the template model, and the render architecture.

| Product | Licensing model | Documented capabilities | Interaction / workflow | Formats / interchange | Documented accessibility |
|---|---|---|---|---|---|
| After Effects | Subscription | Layer/composition model, precomps, render order, Essential Graphics/MOGRT templates, Render Queue, 3D renderers, Content-Aware Fill, expressions | Layer stack, timeline, composition nesting | Proprietary AEP; MOGRT; media and image sequences | Adobe publishes an ACR for After Effects |
| Nuke | Commercial subscription plus free non-commercial edition | Node graph, 200+ nodes, multi-channel workflow, resolution-independent processing, deep compositing, Python, OCIO | Node graph, script-based, shot-centric | Proprietary scripts and node files; OpenEXR and media formats | `UNVERIFIED` |
| Fusion | Included in DaVinci Resolve; paid Studio standalone | Node-based compositing, 2D/3D, keying, tracking, particles, deep compositing | Node graph, page inside Resolve | Fusion comps; OpenEXR and media formats | `UNVERIFIED` |
| Natron | GPL free software | Node-based compositing, OpenFX plugins, keying, tracking, 2D/3D | Node graph | `.ntp`; OpenEXR and image sequences | `UNVERIFIED` |
| Blender Compositor | GPL free software | Render layers and passes, node graph, compositing inside Blender [S43][S44] | Node graph, scene/render-layer inputs | `.blend`; OpenEXR and image sequences | `UNVERIFIED` |
| Cavalry | Free Starter plus paid Professional subscription | Procedural 2D animation, scene tree, time and graph editors, behaviours, JavaScript, Lottie export | Scene tree plus timeline plus graph editor | Proprietary project; Lottie, image and video export | `UNVERIFIED` |

**Observed patterns**

- Layer/composition tools optimize for motion graphics and handoff to editors; node tools optimize for shot work, multi-channel images, and repeatable recipes [S38][S40].
- Templates are the automation surface for teams. After Effects documents Essential Graphics and MOGRT templates as an editable handoff format [S39].
- Deep compositing, multi-channel handling, and color management are the professional boundaries between a motion-graphics tool and a shot-compositing tool [S40][S42].
- Free editions are common for evaluation, but their limits differ: Nuke Non-commercial restricts output and plugins, and Cavalry offers a free Starter tier [S41][S45].

**Documented limitations**

- Nuke Non-commercial limits output to HD, disables some nodes, blocks third-party plugins, encrypts files, and restricts the usable version window [S41][S42].
- After Effects’ render order and precomposition nesting create hidden complexity: the visible layer stack is not always the whole evaluation order [S38].
- Cavalry’s project format is proprietary. Its free/paid feature boundary must be verified per release; a specific subscription-lapse behavior is `UNVERIFIED` and is not used as evidence here.
- Node graphs scale by convention, not automatically: naming, grouping, and script hygiene are user responsibilities [S40].

**EQwec opportunities**

- Build a hybrid authoring model: a layer-based surface for motion graphics, backed by an inspectable node graph for compositing and automation.
- Publish an open template format with typed parameters, versioning, and a sandbox for expressions; do not require a proprietary runtime to open a template.
- Make OCIO color management and deep-image compositing core architecture, not later plugins.
- Provide a render graph with explicit cache keys, invalidation rules, and farm distribution, so a render is reproducible from a manifest.
- Support keyboard graph navigation and screen-reader descriptions of node parameters as accessibility requirements.

## 9. CraftAnything

CraftAnything has no direct one-to-one competitor. The useful comparison is a set of adjacent models: asset browsers, shared libraries, editorial interchange, scene description, and provenance. Each solves one slice of the cross-application problem.

| Analogue | Licensing model | Documented capabilities | Interaction / workflow | Formats / interchange | Documented accessibility |
|---|---|---|---|---|---|
| Adobe Bridge | Free | File browsing, XMP metadata, keywords, collections, smart collections, batch rename, Creative Cloud Libraries access, export presets | File-browser-centric, metadata panels | XMP; Adobe and common media formats | `UNVERIFIED` |
| Creative Cloud Libraries | Subscription | Shared graphics, videos, colors, text styles, brushes, and layer styles across Adobe apps; sync; share links; up to 10,000 assets per library | Panel inside creative apps, library-centric | Proprietary library assets; Adobe cloud sync | `UNVERIFIED` |
| Figma libraries | Free and paid plans | Publish components, styles, and variables; notify subscribers; review and accept updates; permissions; version history | Design-tool panels, library publishing | Proprietary Figma files; components and styles | `UNVERIFIED` |
| Eagle | Paid license (verify) | Image asset management, tags, folders, smart folders, browser extension, metadata | Library-centric GUI | Proprietary library; common image/video formats | `UNVERIFIED` |
| PureRef | Free/donationware (verify) | Reference-image board, image arrangement, note overlays, always-on-top | Canvas board | Proprietary board; image import | `UNVERIFIED` |
| OpenTimelineIO | Open-source project | Editorial interchange API and format for clips, timing, tracks, transitions, markers, and metadata; media referenced externally | API and file format, pipeline-oriented | `.otio` JSON; references external media | Not applicable |
| OpenUSD | Open-source project | Scene description composition through layers, references, and variants; not merely a single file format | Scene graph and composition arcs, pipeline-oriented | `.usd`, `.usda`, `.usdc` and related layers | Not applicable |
| C2PA | Open technical specification | Content provenance and authenticity through manifests, assertions, claim signatures, and ingredients | Embedded or sidecar provenance, verification-oriented | C2PA manifests associated with assets | Not applicable |

**Observed patterns**

- Asset libraries succeed when they live inside the creation tool. Bridge is a browser, but Creative Cloud Libraries are panels inside Adobe apps; Figma libraries are published from the design tool itself [S48][S49][S50][S51].
- Metadata and provenance outlive file formats. XMP keywords and C2PA manifests remain useful even when the authoring application changes [S47][S56].
- Shared-library updates need review and acceptance. Figma documents subscriber notifications and an explicit review/accept step, which prevents silent breakage [S52].
- Interchange formats work when their scope is narrow. OTIO covers editorial timing; USD covers scene composition; neither tries to be a universal document [S53][S54].
- Provenance is opt-in and trust-dependent. C2PA defines manifests and signatures, but adoption requires signing keys, verification policy, and privacy decisions [S56].

**Documented limitations**

- Creative Cloud Libraries are account-bound and Adobe-ecosystem-bound; they are not a neutral cross-vendor exchange format [S49].
- Figma library publishing is plan-gated and platform-bound; consumers must use Figma-compatible workflows [S50][S51].
- OTIO references media externally and does not package the media itself [S53].
- USD is a composition system with layers, references, and variants; treating it as a simple asset container loses its semantics [S54][S55].
- C2PA does not by itself establish truth; it records signed provenance claims, and verification depends on trust anchors and implementation quality [S56].

**EQwec opportunities**

- Make the workspace manifest reference canonical documents by content-addressed ID, with provenance, license, and link health; do not make CraftAnything the owner of every document.
- Add a review/accept workflow for shared assets with semantic diff: what changed, who changed it, and which consumers are affected.
- Provide handoff packages that combine media references, timeline or scene references, and a C2PA-compatible provenance manifest.
- Surface link rot and missing dependencies before export, with relink and repair actions.
- Explicitly do not build a universal editor; integrate with each product’s native document model.

## 10. Audio

AudioCraft’s defining constraint is real-time safety. A feature that is acceptable in an editor can be unacceptable in an audio callback. The comparison therefore separates workflow features from host architecture.

| Product | Licensing model | Documented capabilities | Interaction / workflow | Formats / interchange | Documented accessibility |
|---|---|---|---|---|---|
| FL Studio | One-time purchase with lifetime free updates | Channel Rack, Playlist, Piano Roll, Mixer with edition-dependent track count, pattern-based sequencing | Pattern and clip workflow, mixer-centric | `.flp`; WAV, MIDI and plugin formats | `UNVERIFIED` |
| Ableton Live | Perpetual licence with paid upgrades | Session and Arrangement views sharing tracks, Max for Live (Suite or add-on), Capture MIDI, audio-to-MIDI, tuning systems, MIDI Tools | Session clip launching plus linear arrangement | `.als`; WAV, MIDI and plugin formats | `UNVERIFIED` |
| Logic Pro | One-time purchase or Apple Creator Studio subscription | Quick Swipe Comping, Flex Time/Pitch, Live Loops, Spatial Audio with Dolby Atmos ADM export, Session Players, Stem Splitter, Mastering Assistant | Track timeline plus Live Loops grid | `.logicx`; WAV, MIDI, ADM/BWAV | Apple documents system-level accessibility; app-specific conformance is `UNVERIFIED` |
| Pro Tools | Subscription or perpetual with support plan | Track-based editing, mixing, AAX plugin hosting, immersive audio, cloud collaboration | Edit and mix windows, track timeline | Pro Tools sessions; AAF, OMF, WAV, MIDI | `UNVERIFIED` |
| Bitwig Studio | 12-month upgrade plan | Clip launcher and arranger, hybrid tracks, unified modulation, The Grid, plugin sandboxing, CLAP support | Clip launcher plus arranger, modular routing | Proprietary project; WAV, MIDI and plugin formats | `UNVERIFIED` |
| REAPER | $60 discounted licence / $225 commercial licence | Flexible routing, 64-bit processing, ReaScript/JSFX, portable install [S65] | Track timeline, routing matrix, highly configurable | `.rpp`; WAV, MIDI and plugin formats | `UNVERIFIED` |
| Ardour | GPL free software | LADSPA/LV2/AU/VST2/VST3 hosting, JACK/MIDI/LTC sync, unlimited tracks and busses, Lua scripting | Track timeline, mixer, routing | `.ardour`; WAV, MIDI and plugin formats | `UNVERIFIED` |
| Cubase | Commercial licence with versioned releases | Edition-dependent Expression Maps, Note Expression, Chord Pads, VST3 hosting, 64-bit plugins only, Modulators, pattern sequencer, DAWproject interchange | Track timeline, arranger, mixer, expression editing | `.cpr`; DAWproject, WAV, MIDI and plugin formats | `UNVERIFIED` |

**Observed patterns**

- Two workflow poles dominate: clip/session launchers (Ableton Live, Bitwig Studio, FL Studio) and linear arrangement tools (Logic Pro, REAPER, Ardour, Cubase). Several products now offer both views over shared tracks [S59][S60][S62].
- Plugin hosting is a platform feature, not a checkbox. Bitwig documents plugin sandboxing and CLAP support; Ardour documents LADSPA, LV2, AU, VST2, and VST3 hosting [S63][S67][S68].
- MIDI expression and articulation management matter for orchestral and scoring work; Cubase’s Expression Maps and Note Expression are explicit examples [S69][S70].
- Licensing models vary widely: lifetime updates, 12-month upgrade plans, perpetual licences with paid upgrades, and free/open-source software all coexist [S57][S58][S61][S64][S66][S67].

**Documented limitations**

- FL Studio’s trial cannot reopen saved projects, which makes evaluation a one-way door for project data [S57][S58].
- Bitwig’s 12-month upgrade plan means newer versions require an active plan; the licence does not grant indefinite future upgrades [S64].
- Cubase hosts 64-bit plugins only; older 32-bit plugins are not hosted natively [S69].
- No performance claim is made here. Real-time latency, xrun behavior, and plugin counts require a cited benchmark with a defined workload and environment, and none were used.

**EQwec opportunities**

- Host CLAP as the preferred open plugin format: MIT-licensed, ABI-stable, C-only ABI, with per-note modulation and MPE support [S71].
- Sandbox plugins and isolate crashes, with a clear recovery path and per-plugin resource accounting.
- Build a real-time-safe graph: lock-free parameter changes, bounded queues, explicit backpressure, and user-visible xrun reporting.
- Provide an open project archive plus stem export and DAWproject interchange, so a session is not trapped in one application [S69].
- Implement articulation/expression maps and MPE as first-class MIDI features, not post-processing.
- Make transport, mixer, and piano-roll workflows keyboard-first and screen-reader-testable.

## 11. What not to copy

**Branding and trade dress.** Do not reuse competitor names, logos, icons, product naming, or distinctive visual identities. Do not imitate Adobe’s panel layout, Apple’s magnetic-timeline branding, Ableton’s Session View identity, or Vivaldi’s visual system in a way that suggests affiliation. Comparative advertising that uses competitor marks requires legal review in each jurisdiction.

**Proprietary code.** Do not copy, decompile, or incorporate competitor source code, binaries, or SDKs without a license. Interoperability work must use publicly documented specifications or a clean-room method approved by counsel; public documentation does not grant an implementation license.

**DRM circumvention.** Do not implement or ship tools that bypass DRM, code signing, license checks, or activation. DRM support requires separate agreements and is out of scope for the first release (ADR-0004).

**Proprietary assets and content.** Do not bundle or redistribute Adobe Stock assets, Creative Cloud Libraries content, brushes, templates, LUTs, sample packs, presets, or project files obtained from a competitor. Do not reverse-engineer proprietary file formats beyond publicly documented specifications. A format’s public documentation does not grant a license to its trademarks, sample content, or implementation patents.

**Protected interactions.** Do not assume a workflow is free to copy because it is visible in a demo. Magnetic timelines, node-graph interactions, radial menus, gesture sets, command chains, and distinctive panel behaviors may be protected by patents, design rights, or trade dress. This is a legal-review item, not a finding that any specific interaction is patented. The safe path is to solve the user problem with an independently designed interaction and documented prior art.

**Lifecycle traps.** Do not copy account deletion at product sunset, cancellation friction, trial files that cannot be reopened, or silent cloud dependency. Arc’s export-before-deletion deadline and FL Studio’s non-reopenable trial projects are documented examples [S8][S57]. A claim that a specific product makes paid features read-only after lapse is `UNVERIFIED` and is not used here.

**Unverifiable marketing.** Do not copy “faster than X” claims, “industry-standard” claims, or compatibility badges without a reproducible benchmark or a published test matrix. A compatibility claim is a test result, not a slogan.

## 12. Interoperability and legal/technical work

Compatibility is a spectrum. EQwec should publish three tiers for every format: **Documented** (vendor documentation exists), **Tested** (EQwec fixtures pass round-trip tests for a defined feature set), and **Certified** (a named third-party workflow has been validated). No format should be described as “fully compatible.”

| Format or standard | Ownership / status | Intended EQwec use | Work required |
|---|---|---|---|
| PSD / PSB | Adobe proprietary, publicly documented | Photo import/export | License review, layer/mask/text fixtures, fidelity tiers |
| AI / AIT | Adobe proprietary | Vector import | PDF-compatible import path, raster fallback, no native-edit promise |
| `.af` | Affinity proprietary | No import commitment | Document as unsupported unless a licensed path exists |
| SVG / SVGZ | W3C open standard | Vector native and exchange | SVG 2 subset, text-on-path, filters, loss reporting |
| PDF / PDF/X | ISO open standard | Print and vector export | PDF/X validation, font embedding, color and overprint |
| FCPXML | Apple proprietary | Film interchange | Versioned schema adapters, timeline fixtures |
| AAF / OMF / EDL | Industry formats, Avid lineage | Film and audio interchange | Adapter layer, round-trip tests, media relink |
| OTIO | Open-source project | Internal timeline and editorial exchange | Schema policy, media reference rules, versioning |
| USD | Open-source project | 3D and scene exchange | Composition semantics, performance, asset resolution |
| OpenEXR | Open-source project | Image and compositing exchange | Multi-part, deep data, color management |
| ADM / BWAV | EBU/ITU specifications | Spatial audio export | Metadata validation, renderer compatibility |
| VST3 | Steinberg | Plugin hosting | SDK license review, conformance tests |
| AU | Apple | macOS plugin hosting | SDK and license review, validation |
| AAX | Avid | Pro Tools plugin hosting | SDK and license review, validation |
| CLAP | MIT open standard | Preferred plugin hosting | Conformance tests, sandbox integration |
| H.264 / H.265 / ProRes / Dolby | Patent and licensing regimes | Media import/export | Legal review, platform codec strategy, royalty budget |
| C2PA | Open technical specification | Provenance and authenticity | Signing/trust model, privacy review, verifier UX |

**Technical work that is not optional**

- Build a fixture corpus with real, licensed test media for every claimed format. A format is not “supported” until the fixture passes on Windows, macOS, and Linux.
- Separate parse, validate, and render stages. Untrusted media must be parsed with bounded memory and time, and malformed files must fail closed.
- Keep format adapters behind a versioned interface. A vendor schema change should not require rewriting the document model.
- Record the exact application, version, and settings used to produce an interchange file. Reproducibility is part of compatibility.

**Legal work that is not optional**

- Review the licenses for VST3, AU, AAX, codecs, and any vendor SDK before shipping a host or exporter.
- Review patent exposure for codecs and for any interaction that resembles a competitor’s patented workflow.
- Review trademark use for format names, compatibility badges, and comparative marketing.
- Review C2PA signing-key custody, revocation, and privacy implications before enabling provenance by default.

## 13. Prioritized opportunities

### Cross-cutting themes

| Theme | Documented market pattern | EQwec position |
|---|---|---|
| Local-first and offline | Cloud libraries, account activation, and store-managed updates are common [S7][S49][S79] | Local projects, offline install and repair, no mandatory account |
| Resource efficiency | Chrome exposes Memory Saver and Energy Saver; NLEs document proxy workflows [S10][S31] | Measured budgets and transparent cache/tab policy |
| Predictable licensing | Subscription, perpetual, upgrade-plan, and free models coexist [S57][S64][S66] | No forced subscription for local use; clear upgrade terms |
| Open formats | OTIO, USD, SVG, C2PA, and CLAP are documented open standards [S53][S54][S56][S71] | Open project archive and documented interchange tiers |
| Accessibility | Adobe publishes ACRs; Apple documents system accessibility [S73][S74][S75] | Accessibility as a release gate with published test evidence |
| Cross-product exchange | CC Libraries and Figma libraries are ecosystem-bound [S49][S50] | CraftAnything references canonical documents with provenance |
| Security | Store review, code signing, and sandboxing are partial signals | Signed manifests, verified updates, sandboxed untrusted content |
| User data ownership | Arc’s sunset requires export before deletion [S8] | Sunset guarantee, local export, no deletion of local work |

### Do better now

1. **Hub trust chain.** Signed manifests, content digests, atomic staging, verification before activation, rollback, repair, offline install, and an audit log. This is the product’s reason to exist, and the market’s documented weak spots include lockfile fidelity and account coupling [S5][S7].
2. **Data portability guarantee.** A documented, versioned project archive for every EQwec app, plus a sunset policy that guarantees local export. Arc’s deletion deadline is the cautionary case [S8].
3. **CraftAnything as provenance-aware exchange.** Content-addressed asset IDs, link health, review/accept, and a C2PA-compatible manifest; integrate with native documents instead of owning them [S53][S56].
4. **Accessibility as a release gate.** Keyboard-first workflows, screen-reader tests, scalable UI, and published conformance reports. Adobe’s own Illustrator ACR documents major screen-reader gaps, so this is a credible differentiator [S73][S74].
5. **Audio plugin openness.** CLAP-first hosting with sandboxing, crash isolation, and a real-time-safe graph. CLAP’s MIT license and ABI stability reduce legal and integration friction [S71][S72].

### Defer

- Native PSD/AI editing. Start with documented import, tested export, and an explicit fidelity report.
- Deep compositing and USD scene graphs. Build the core compositor and color pipeline first.
- A browser extension ecosystem. Start with compatibility, permission scoping, and a security model.
- Cloud sync and real-time collaboration. Local-first correctness, conflict handling, and export come first.
- Mobile and tablet parity. Desktop professional workflows are the initial scope.

### Do not attempt

- A universal editor inside CraftAnything. It would duplicate every product’s document model and fail at all of them.
- Reimplementing Chromium. Use a maintained Chromium foundation and own application logic, persistence, and native UI.
- “Fully compatible” claims for proprietary formats. Publish tiers and test evidence instead.
- Bundling competitor assets, templates, brushes, LUTs, or sample content.
- Performance marketing without a benchmark harness, workload, environment, and reproducible result.
- Account-only local use, subscription lock-in, or data deletion at sunset.
- Copying a distinctive interaction without legal review and an independent design.

## 14. Scope risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Feature-parity trap | High | High | Vertical slices, explicit non-goals, compete on document lifecycle rather than feature count |
| Browser engine maintenance | High | High | Pin CEF, define a patch SLA, exercise a rebuild, keep the platform-webview fallback |
| Codec and DRM licensing | High | High | Royalty-free default, legal gate before gated codecs, no DRM in the first release |
| Plugin compatibility | Medium | High | CLAP first, VST3 second, out-of-process hosting, conformance fixtures |
| Format lock-in | Medium | Medium | Open project archive, export guarantee, published fidelity tiers |
| Team capacity | High | High | Sequence one product at a time, reuse shared infrastructure, delete non-essential scope |
| Breadth before depth | High | High | Gate each phase; do not start a new product before the previous vertical slice is defensible |

Breadth is the compounding risk: eight product families, including two platform-scale products, are planned while the repository still has no Cargo workspace. Interchange and legal exposure compound it, because PSD, AI, FCPXML, AAF, VST3, AU, AAX, codecs, and C2PA each carry technical and legal obligations. Real-time and render quality cannot be validated without a benchmark harness and representative workloads; without that harness, performance claims and regressions are indistinguishable.

**Accessibility evidence used**

- Adobe publishes accessibility conformance reports for its applications; the Illustrator report documents major screen-reader gaps, which is a caution against treating a feature list as conformance [S73][S74].
- Apple documents system-level accessibility features; this review did not verify app-specific DAW or NLE conformance [S75].
- GIMP publishes a VPAT, but it dates from 2015 and its scope should be checked against current releases [S76].
- For products marked `UNVERIFIED`, no official conformance statement was located; independent testing is required before any comparison claim.

## 15. Sources

All sources accessed `2026-11-07`.

1. Steam Cloud — https://partner.steamgames.com/doc/features/cloud
2. Steam Workshop — https://partner.steamgames.com/doc/features/workshop
3. GOG Galaxy — https://www.gog.com/galaxy
4. WinGet documentation — https://learn.microsoft.com/en-us/windows/package-manager/winget/
5. Homebrew Bundle and Brewfile — https://docs.brew.sh/Brew-Bundle-and-Brewfile
6. Flatpak introduction — https://docs.flatpak.org/en/latest/introduction.html
7. Adobe Creative Cloud desktop app — https://www.adobe.com/creativecloud/desktop-app.html
8. Arc — https://arc.net/
9. Brave features — https://brave.com/features/
10. Chrome Memory Saver — https://support.google.com/chrome/answer/12929150
11. Chrome profiles — https://support.google.com/chrome/answer/2364824
12. Firefox 136 release notes — https://www.mozilla.org/en-US/firefox/136.0/releasenotes/
13. Firefox Containers — https://wiki.mozilla.org/Security/Contextual_Identity_Project/Containers
14. Safari profiles — https://support.apple.com/guide/safari/create-a-profile-ibrwf3a9e7d6/mac
15. Vivaldi features — https://vivaldi.com/features/
16. Photoshop file formats — https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
17. Lightroom Classic Develop module — https://helpx.adobe.com/lightroom-classic/desktop/help/applying-adjustments-develop-module-basic.html
18. Affinity announcement — https://www.canva.com/newsroom/news/all-new-affinity/
19. Affinity download and activation — https://affinity.studio/get-affinity
20. GIMP 3.0 release — https://www.gimp.org/news/2025/03/16/gimp-3-0-released/
21. GIMP 3.2.8 release — https://www.gimp.org/news/2026/11/02/gimp-3-2-8-released/
22. GIMP about — https://www.gimp.org/about/
23. Krita features — https://krita.org/en/features/
24. darktable about — https://www.darktable.org/about/
25. Capture One features — https://www.captureone.com/en/features
26. Illustrator supported file formats — https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/supported-file-formats-in-illustrator.html
27. CorelDRAW — https://www.coreldraw.com/en/product/coreldraw/
28. Inkscape 1.4 release notes — https://wiki.inkscape.org/wiki/Release_notes/1.4
29. DaVinci Resolve — https://www.blackmagicdesign.com/products/davinciresolve
30. Final Cut Pro — https://www.apple.com/final-cut-pro/
31. Shotcut features — https://shotcut.org/features/
32. Premiere Pro text-based editing — https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html
33. Premiere Pro ingest and proxy workflow — https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/ingest-and-proxy-workflow.html
34. Avid Media Composer — https://www.avid.com/media-composer
35. Avid third-party storage workflows — https://kb.avid.com/pkb/articles/en_US/Knowledge/shared-project-workflows-with-third-party-storage
36. Kdenlive project files — https://docs.kdenlive.org/en/project_and_asset_management/file_management/project_files.html
37. Kdenlive introduction — https://docs.kdenlive.org/en/getting_started/introduction.html
38. After Effects composition basics — https://helpx.adobe.com/after-effects/desktop/work-with-compositions/composition-settings/composition-basics.html
39. After Effects motion graphics templates — https://helpx.adobe.com/after-effects/desktop/motion-graphics/work-with-motion-graphics-templates/creating-motion-graphics-templates.html
40. Foundry Nuke — https://www.foundry.com/products/nuke-family/nuke
41. Nuke Non-commercial — https://learn.foundry.com/nuke/current/content/getting_started/meet_nuke/about_nc.html
42. Foundry support — https://support.foundry.com/hc/en-us/articles/360001227130
43. Blender compositor — https://docs.blender.org/manual/en/latest/compositing/index.html
44. Blender render layers — https://docs.blender.org/manual/en/latest/compositing/types/input/scene/render_layers.html
45. Cavalry documentation — https://cavalry.studio/docs/
46. Cavalry press — https://cavalry.scenegroup.co/press
47. Adobe Bridge metadata — https://helpx.adobe.com/bridge/desktop/organize-and-find-files/tag-and-find-files/about-metadata.html
48. Adobe Bridge and Creative Cloud Libraries — https://helpx.adobe.com/bridge/desktop/organize-and-find-files/organize-files-and-folders/organize-content-in-creative-cloud-libraries.html
49. Creative Cloud Libraries overview — https://helpx.adobe.com/creative-cloud/apps/create-and-manage-libraries/create-and-organize-libraries/libraries-overview.html
50. Figma publish a library — https://help.figma.com/hc/en-us/articles/360025508373-Publish-a-library
51. Figma libraries guide — https://help.figma.com/hc/en-us/articles/360041051154-Guide-to-libraries-in-Figma
52. Figma review and accept library updates — https://help.figma.com/hc/en-us/articles/360039234193-Review-and-accept-library-updates
53. OpenTimelineIO — https://opentimelineio.readthedocs.io/en/latest/
54. OpenUSD introduction — https://openusd.org/release/intro.html
55. OpenUSD FAQ — https://openusd.org/release/usdfaq.html
56. C2PA explainer — https://spec.c2pa.org/specifications/specifications/2.4/explainer/Explainer.html
57. FL Studio features — https://www.image-line.com/fl-studio/features
58. FL Studio pricing — https://www.image-line.com/fl-studio/pricing
59. Ableton Live — https://www.ableton.com/en/live/
60. Ableton Live concepts — https://www.ableton.com/en/live-manual/12/live-concepts/
61. Logic Pro — https://www.apple.com/logic-pro/
62. Bitwig modern foundations — https://www.bitwig.com/modern-foundations
63. Bitwig plugin handling — https://www.bitwig.com/userguide/latest/vst_plug-in-handling_and_options
64. Bitwig shop — https://www.bitwig.com/en/shop.html
65. REAPER — https://www.cockos.com/reaper/
66. REAPER purchase — https://www.reaper.fm/purchase.php
67. Ardour features — https://ardour.org/features.html
68. Ardour plugins — https://manual.ardour.org/working-with-plugins
69. Cubase 15 release notes — https://www.steinberg.net/cubase/release-notes/15/
70. Cubase Expression Maps — https://archive.steinberg.help/cubase_pro/v12/en/cubase_nuendo/topics/expression_maps/expression_maps_c.html
71. CLAP — https://github.com/free-audio/clap
72. Bitwig CLAP story — https://www.bitwig.com/stories/clap-the-new-audio-plug-in-standard-201/
73. Adobe accessibility compliance — https://www.adobe.com/accessibility/compliance.html
74. Illustrator accessibility conformance report — https://www.adobe.com/trust/accessibility/conformance/adobe-illustrator-2024-acr.html
75. Apple accessibility — https://www.apple.com/accessibility/
76. GIMP VPAT — https://www.gimp.org/about/ITI-VPAT.pdf
77. Homebrew FAQ — https://docs.brew.sh/FAQ
78. Flatpak sandbox permissions — https://docs.flatpak.org/en/latest/sandbox-permissions.html
79. Microsoft Store apps — https://apps.microsoft.com/
80. JetBrains Toolbox — https://www.jetbrains.com/toolbox-app/
81. Microsoft Edge — https://www.microsoft.com/edge
82. Adobe Lightroom — https://www.adobe.com/products/photoshop-lightroom.html
83. Blackmagic Fusion — https://www.blackmagicdesign.com/products/davinciresolve/fusion
84. Natron — https://github.com/NatronGitHub/Natron
85. Avid Pro Tools — https://www.avid.com/pro-tools
86. Eagle — https://eagle.cool/
87. PureRef — https://www.pureref.com/

**Source count:** 87. Vendor documentation and specifications dominate; no benchmark, review site, or marketing comparison is used as evidence.
