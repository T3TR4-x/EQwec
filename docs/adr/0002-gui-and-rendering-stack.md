# ADR-0002: GUI and rendering stack

- **Status:** ACCEPTED (with a mandatory prototype gate)
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0001, `docs/architecture/TECHNOLOGY-SELECTION.md`, `docs/performance/PERFORMANCE-STRATEGY.md`

## 1. Context

EQwec plans eight native desktop products: Hub, Browser, PhotoCraft, VectorCraft, FilmCraft, EffectCraft, CraftAnything, and AudioCraft. They share a need for a responsive, accessible, keyboard-first desktop UI with custom canvases, docked panels, inspectors, timelines, and node graphs. They must run on Windows, macOS, and Linux without bundling a browser runtime for application chrome.

The UI choice is expensive to reverse. It determines the design system, accessibility path, text input behavior, rendering integration, packaging size, and the skills required to build every product. The project has no Cargo workspace and no prototype, so the decision must separate the architectural direction from the prototype evidence that can confirm or reject it.

## 2. Requirements

- Rust-first and native-first; no Electron or web runtime for app chrome.
- Windows, macOS, and Linux on x86_64 and ARM64 where feasible.
- Dense professional layouts with docking, panels, inspectors, timelines, node graphs, and large canvases.
- Custom visual identity, dark/light/high-contrast themes, scalable UI, and reduced-motion support.
- Keyboard navigation, visible focus, screen-reader access, and logical focus order.
- Text input and IME for English, Ukrainian, and Slovak, including composed characters.
- High-DPI rendering, smooth pan/zoom, and predictable frame times.
- A permissive license compatible with an undecided project license.
- A single GPU abstraction for UI and custom canvases.
- A credible fallback if the primary fails accessibility or text input.
- Bounded memory and startup; no per-product UI stack duplication.

## 3. Alternatives considered

### A. egui / eframe

Pure Rust, immediate mode, MIT OR Apache-2.0. Uses `wgpu` or `glow`; AccessKit is a required dependency since 0.34. Strong custom-widget and canvas story; large ecosystem; used in production tools. Weaknesses: immediate-mode redraw cost, text editing/IME less mature than native toolkits, native menus/dialogs require separate integration, and frequent breaking releases.

### B. Slint

Declarative DSL, Rust/C++/JavaScript/Python, GPLv3, royalty-free desktop/mobile/web, or commercial. Good performance and native-looking widgets; accessibility support exists but has open issues. The license couples EQwec to GPLv3 or a commercial/royalty-free agreement, which is a strategic constraint while the project license is undecided. Custom canvas integration is less direct.

### C. Iced

Pure Rust, Elm architecture, MIT, `wgpu` backend. Good app-chrome model. Stable 0.14 has no AccessKit integration; the accessibility issue has been open since 2020. Text/IME and the widget ecosystem are less mature for professional editors. Rejected for v1.

### D. Qt 6 via cxx-qt

Mature accessibility, native integration, widgets, and deployment tooling. LGPLv3, GPLv3 (some modules only), or commercial. Adds a C++ toolchain and bindings, a large footprint, and LGPL/commercial obligations. Not Rust-first. Viable fallback if egui cannot meet accessibility.

### E. Platform-native UI per OS

WinUI 3/Win32, AppKit, and GTK 4 provide the best integration and accessibility, but require three separate UIs and skill sets. Rejected for v1; platform-native adapters are still used for menus, dialogs, and accessibility bridges.

### F. Tauri or Electron

Both use a web runtime for app chrome. Tauri is lighter than Electron but still carries a webview, web-security model, and accessibility/memory costs. Electron's own security documentation states that it is not a browser and is unsuitable for arbitrary untrusted content. Rejected under the native-first policy.

## 4. Evidence

- egui 0.35.0 was published on 2026-06-25; AccessKit 0.24.1 is a required dependency of core egui since 0.34.
- Iced 0.14.0 has no AccessKit dependency; issue #552 has been open since 2020-10-05.
- Slint is available under GPLv3, a royalty-free desktop/mobile/web license, and a commercial license; the royalty-free license has distribution conditions and embedded royalties. Slint 1.18 is current.
- Qt is available under LGPLv3, GPLv3 (some modules only), and commercial terms; mixing commercial and community editions in one project is not allowed.
- `wgpu` 30.0.1 is MIT OR Apache-2.0.
- Vello 0.11.0 is Apache-2.0 OR MIT; the project separates Vello CPU, Vello GPU, and an experimental compute renderer. Vello CPU is described as the most mature choice; Vello GPU is intended to become the primary production GPU renderer as it matures.
- Skia is BSD-3-Clause but requires a C++ build and adds significant size and a second rendering model.
- Electron's security documentation explicitly warns against using it for arbitrary untrusted content.

## 5. Decision

### 5.1 Primary stack

1. Use `egui`/`eframe` as the primary application UI framework.
2. Wrap it in a project-owned `eqwec-ui` crate that owns design tokens, typography, spacing, color, elevation, motion, focus, keyboard navigation, i18n, and reusable primitives. Product code must not depend on raw `egui` types outside this layer unless the dependency is reviewed.
3. Use `wgpu` as the single GPU abstraction. `egui`'s `wgpu` backend and custom canvas renderers share one device manager. Do not add a second GPU API in product code without a new ADR.
4. Use `kurbo` for Bézier and geometric math, `lyon` for tessellation, and `cosmic-text` for shaping and layout.
5. Use a CPU reference renderer for correctness tests. Evaluate `vello_cpu` and `vello_gpu` for high-quality 2D rendering; do not make Vello a release dependency until its 0.x API, quality, and performance are verified on all target platforms.
6. Do not use Skia initially. Reconsider only if the Rust rendering path cannot meet documented quality or performance budgets.
7. Use platform adapters for native menus, file dialogs, clipboard, drag-and-drop, notifications, and IME. Candidate crates (`muda`, `rfd`, `arboard`, platform notification crates) require normal dependency admission review.
8. Use AccessKit through egui and the platform bridges (Windows UIA, macOS NSAccessibility, Linux AT-SPI). Accessibility annotations are part of the UI layer, not an afterthought.

### 5.2 Fallback order

If the prototype fails accessibility, text input, or performance gates:

1. **Slint**, if the project license or a commercial/royalty-free agreement is resolved and the accessibility defects are fixed.
2. **Qt 6 via cxx-qt**, if an LGPLv3 distribution or commercial license is approved and the C++ toolchain cost is accepted.

A failed gate is a decision trigger. Do not lower the accessibility or text-input requirement to preserve the primary choice.

### 5.3 Rejected for v1

- Iced, because stable 0.14 lacks AccessKit.
- Tauri/Electron for app chrome, because of the web runtime and security/accessibility costs.
- Platform-native triple UI, because of the cost of three separate implementations.

## 6. Prototype gate

Before product UI work, a time-boxed prototype must prove on Windows, macOS, and Linux:

1. A representative editor shell with docked panels, inspector, timeline or node graph, canvas, menus, dialogs, and command palette.
2. Text input and IME for English, Ukrainian, and Slovak, including dead keys and composed characters.
3. Screen-reader access through AccessKit and the platform bridge: names, roles, values, states, focus order, and live updates.
4. Keyboard-only navigation, visible focus, high-contrast mode, and reduced motion.
5. High-DPI rendering and 60 fps interaction on the mid-range hardware profile.
6. Native file dialogs, menus, clipboard, drag-and-drop, and multi-window behavior.
7. A measured idle and active memory baseline for the shell.

The prototype is a gate, not a product. Its results are recorded in `docs/performance/` and the ADR is updated with the outcome.

## 7. Consequences

**Positive**

- One permissive, pure-Rust UI stack for all products.
- Custom visual identity and editor-specific widgets without fighting a native widget set.
- AccessKit path exists in the primary stack.
- `wgpu` gives one GPU abstraction for UI and canvases.
- The `eqwec-ui` wrapper limits churn from egui's breaking releases.

**Negative**

- Immediate-mode redraw can waste CPU if repaint is not managed carefully.
- Text editing and IME are less mature than native toolkits; the prototype may expose defects.
- Native menus, dialogs, and accessibility bridges require separate integration work.
- egui's release cadence requires deliberate upgrades and visual regression tests.
- A fallback would be expensive; the prototype must run early.

**Security**

- The UI framework is not a trust boundary. It must not execute web content or load untrusted code.
- Accessibility bridges, clipboard, drag-and-drop, and file dialogs are input boundaries and must validate data.
- CEF surfaces remain isolated from the UI process; engine IPC is validated independently.

**Performance**

- Immediate-mode UI requires explicit repaint control and profiling.
- Custom canvases must not block the UI thread; use worker pools and GPU resources with bounded caches.
- The prototype records frame times, input latency, startup, and memory on the defined hardware profiles.

**Licensing**

- egui, AccessKit, wgpu, kurbo, lyon, and cosmic-text are permissive (MIT/Apache/BSD-style; verify exact versions at admission).
- Vello is Apache-2.0 OR MIT but is 0.x; its API risk is an engineering risk, not a license risk.
- The primary stack does not force GPLv3 or a commercial UI license.
- Fonts and icons require their own licenses and notices.

**Maintenance**

- egui's breaking releases are absorbed by `eqwec-ui`.
- A rendering abstraction keeps Vello/Skia replaceable.
- The fallback options remain documented but are not maintained in parallel.

## 8. Reconsideration criteria

Revisit this ADR if:

- The prototype fails screen-reader access, IME, or keyboard navigation on any target platform.
- egui cannot sustain 60 fps or acceptable input latency on the mid-range profile after profiling.
- egui's breaking-release cadence becomes unmanageable despite the wrapper.
- Slint resolves its accessibility defects and the project license makes its terms acceptable.
- A commercial Qt license or an approved LGPLv3 distribution makes Qt the better accessibility path.
- Vello reaches a stable API and meets quality/performance budgets, or the Rust rendering path cannot meet them.
- A product requires a native control that egui cannot provide without disproportionate custom work.

## 9. Links

- `docs/architecture/TECHNOLOGY-SELECTION.md`
- `docs/architecture/SYSTEM-ARCHITECTURE.md`
- `docs/performance/PERFORMANCE-STRATEGY.md`
- `docs/research/LEGAL-LICENSE-REVIEW.md`
- ADR-0001
