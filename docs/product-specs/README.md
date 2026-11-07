# Product Specs

Detailed product specifications are written before each product's implementation, from approved research and architecture. This directory does not contain fabricated feature promises.

## Scope matrix (planning)

| Product | Minimum viable workflow | Explicit non-goals for the first milestone |
|---|---|---|
| EQwec Hub | Install a signed fixture package, verify it, launch it, update it, interrupt/recover, repair, roll back, uninstall | Store, social features, mandatory accounts, cloud sync |
| EQwec Browser | Render real sites, tabs, one profile, session restore, basic downloads and permissions | Full extension ecosystem, sync, advanced privacy suite, mobile |
| PhotoCraft | Open an image, non-destructive adjustments/masks, save/reopen native project, export | Full RAW camera matrix, plugin ecosystem, cloud library |
| VectorCraft | Create/edit Bézier paths, boolean operations, save/reopen, SVG import/export | Full SVG 2/filters, animation, print production |
| FilmCraft | Import media, timeline edit, save/reopen, render playable output with synced audio | Multi-user collaboration, full codec matrix, advanced color grading |
| EffectCraft | Build a small compositing graph, keyframe, render, exchange with FilmCraft | Full Nuke-style node library, 3D renderer, simulation |
| CraftAnything | Define workspace scope, open assets in their canonical owner, preserve provenance, handle a conflict | Universal editor, duplicate product internals, cloud workspace |
| AudioCraft | Record/import, arrange, automate, save/reopen, render without unacceptable audio errors | Full plugin marketplace, hardware control surfaces, cloud collaboration |

These are planning boundaries, not shipped capabilities.

## Required sections for a product spec

1. Problem and target user.
2. Goals and non-goals.
3. Core workflows and acceptance criteria.
4. Document/project format and interoperability.
5. Domain model and invariants.
6. Architecture and dependencies.
7. Security and privacy boundaries.
8. Performance budgets and measurement method.
9. Accessibility and localization.
10. Test fixtures and end-to-end workflows.
11. Release channels and versioning.
12. Known limitations and deferred features.
13. Open questions and blockers.

## Rules

- Do not describe a product as complete before its acceptance workflow passes.
- Do not claim compatibility with a proprietary format without legal and technical verification.
- Do not copy proprietary assets, branding, or protected interactions.
- Every numeric performance target needs a measurement method and a baseline.
- Every product spec must link to its skill, architecture ADRs, and test fixtures.
- A product spec is `DRAFT` until reviewed, `APPROVED` when the architecture and security gates accept it, and `SUPERSEDED` when replaced.

## Current state

No product spec is approved. The first spec should be EQwec Hub, because it is the first implementation milestone and its security model is well bounded.
