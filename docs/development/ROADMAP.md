# Roadmap

**Status:** planning roadmap. No dates are promised. A phase closes only when its gate passes with evidence.

## Phase 0 — Environment, skills, and agent governance

**State:** complete for the bootstrap scope.

Deliverables:

- Environment report and external skill inventory.
- Ten project-owned skills, validated and discoverable.
- `AGENTS.md`, `CLAUDE.md`, skill audit, lock manifest, usage matrix.
- Trigger and scenario evidence.
- Branch and documentation policy.

Gate: `SKILLS_GATE = PASS` for bootstrap. See `docs/development/SKILLS-AUDIT.md`.

## Phase 1 — Technical research and feasibility

**State:** complete for the planning scope. Runtime product feasibility remains `BLOCKED` until prototypes exist.

Deliverables:

- `docs/research/FEASIBILITY.md`
- `docs/research/COMPETITOR-ANALYSIS.md`
- `docs/research/LEGAL-LICENSE-REVIEW.md`
- `docs/research/BROWSER-ENGINE-FEASIBILITY.md`
- `docs/architecture/TECHNOLOGY-SELECTION.md`
- ADRs 0002–0010 for GUI/rendering, browser engine, media/codecs, audio/plugins, project format, packaging/signing, licensing, diagnostics, and plugin trust.

Gate: high-risk choices have credible implementation paths, known blockers, and a fallback. Unresolved legal or security blockers stop the affected workstream.

Outcome: `PASS` for planning. ADR-0004 and ADR-0008 remain `PROPOSED`; codec/DRM licensing, GUI accessibility, audio real-time behavior, and browser integration are `BLOCKED` until their stated prototype or legal gates run. See `docs/research/FEASIBILITY.md`.

## Phase 2 — Architecture and repository baseline

**State:** in progress. Workspace, `eqwec-core`, CI configuration, local advisory/license checks, and `dev-hub` exist; the gate is not closed until CI executes.

Deliverables:

- Approved `SYSTEM-ARCHITECTURE.md` and `DEPENDENCY-MAP.md`.
- Accepted ADRs for the first implementation decisions.
- Cargo workspace with a pinned toolchain. **Done.**
- Minimal shared crates with real APIs and tests. **`eqwec-core` done; 17 tests pass.**
- CI baseline: format, lint, build, test, advisory/license checks. **Configured; never executed because no remote exists.**
- `dev-*` and `production` branch creation from the approved baseline. **`dev-hub` created; `production` deferred until a verified release.**
- Initial version metadata at `0.0.0`. **Done.**
- Documentation and security gates reviewed against the actual workspace. **Done.**

Gate: reproducible local build and passing baseline checks. No product feature yet.

## Phase 3 — EQwec Hub MVP

**State:** in progress on `dev-hub`. Verification core, durable-state primitives, the in-process activation engine, bounded package staging, and a non-production fixture product with in-process macOS end-to-end tests implemented; download transport, installer orchestration, uninstall, and launcher not started.

Completed:

- Normative manifest, trust-store, registry, journal, and layout schemas (ADR-0011 and `docs/product-specs/eqwec-hub-schemas.md`).
- Bounded manifest v1 parsing, canonical byte generation, and exact fixed-vector test.
- Pinned trust-store parsing with channel, epoch, revocation, and key-window policy.
- Strict Ed25519 and SHA-256 primitives with RFC 8032 and known-answer tests.
- Package size and hash verification for reader and in-memory paths.
- Registry v1 with parse, serialize, atomic write, and layout containment.
- Journal v1 with bounded append/read, partial-line recovery, and fail-closed corrupt lines.
- Installation layout and atomic `active.json` pointer primitives.
- Activation engine for fresh install, update, rollback, and startup recovery, with five fault-injection points and a single-in-flight-operation invariant.
- Bounded ZIP/ZIP64 package validation and extraction into a durable staging tree, including path, mode, header-confusion, ratio, size, and timeout rejection with partial-tree cleanup.
- A real non-production fixture product (`tests/fixtures/eqwec-hub`) with v1 and v2 executables and signed packages. Three in-process macOS tests install, launch, update, roll back, reject a tampered package, and recover an interrupted activation.
- 163 workspace tests pass; fmt, clippy, build, audit, and deny pass locally.

Remaining:

- Native launcher UI.
- Verified download transport, uninstall, and offline install.
- Repair orchestration and integrity re-verification of installed trees.
- Diagnostics with redaction.
- Installer packaging and a real fixture product installed, launched, updated, repaired, and removed on each claimed platform.

Acceptance: install, verify, launch, update, interrupt/recover, repair, roll back, and uninstall a real fixture application on each claimed platform. This remains `BLOCKED` until the remaining items exist and pass.

Why first: Hub establishes package trust, versioning, installation, and update infrastructure that the ecosystem needs. It is smaller than a browser or creative editor and its security model is well bounded.

## Phase 4 — Browser feasibility prototype

**State:** not started.

Scope:

- Narrow Chromium-based shell.
- Real website rendering and navigation.
- Multiple tabs and a profile.
- Rust-to-engine integration and validated IPC.
- Process isolation and sandbox preservation.
- Session persistence and basic restore.
- Initial memory, startup, and crash measurements.

Gate: the engine integration is maintainable, secure, and demonstrably functional. Do not expand into advanced browser features before the fundamental engine risks are resolved.

## Phase 5 — ArtCraft foundation

**State:** not started.

Scope:

- Shared document lifecycle: create, open, edit, save, reopen, recover.
- Command/undo model.
- Asset loading and bounded caches.
- GPU canvas abstraction where justified.
- Project serialization and migrations.
- Autosave, journals, and crash recovery.
- Shared design tokens and UI primitives.

Gate: a real document can be created, modified, saved, reopened, and recovered without data loss.

## Phase 6 — First complete creative application

**State:** not started.

Select VectorCraft or PhotoCraft after the feasibility and dependency analysis. Criteria: smallest credible vertical slice, testability, format clarity, and reuse of the Phase 5 foundation.

Acceptance: the application can produce, save, reload, and export meaningful user work, with a documented format and tested failure paths.

## Phase 7 — Remaining products

**State:** not started.

Order by dependency and risk, not by visual appeal. Each product has its own scope, milestones, tests, version, and release gate. Do not destabilize a released product to accelerate a new one.

Candidate order (to be confirmed after research):

1. PhotoCraft or VectorCraft (whichever is not first).
2. FilmCraft (depends on media and render infrastructure).
3. AudioCraft (independent real-time and plugin stack).
4. EffectCraft (depends on compositing and FilmCraft interchange).
5. CraftAnything (depends on stable product contracts and asset exchange).

## Phase 8 — Performance and security hardening

**State:** not started.

Scope:

- Profiling and memory optimization against approved budgets.
- UI responsiveness and startup audits.
- Threat-model validation and fuzzing.
- Compatibility and regression matrices.
- Installation, update, and recovery fault injection.
- Accessibility and localization audits.

Gate: release-specific performance and security criteria pass with recorded evidence.

## Phase 9 — Production releases

**State:** not started.

Per product:

- Release checklist and changelog.
- Reproducible packages.
- Integrity and signature verification.
- Install, launch, update, rollback, repair, uninstall verification.
- Merge into `production`, tag, publish through an authorized channel.
- Verify installation from the published artifact.
- Reconcile back into `integration` and `dev-*`.

Products release independently. The ecosystem is not "finished" until each product meets its own acceptance criteria.

## Cross-cutting workstreams

| Workstream | Runs across |
|---|---|
| Security | All phases; threat model review at each boundary change |
| Performance | Baselines in Phase 3+, budgets before optimization |
| Accessibility | Design system in Phase 2/5; audits in Phase 8 |
| Localization | Infrastructure early; English/Ukrainian/Slovak candidates |
| Documentation | Every phase; impacted docs only |
| Licensing | Research now; SBOM and notices before release |
| CI/CD | Phase 2 baseline; expand per platform and product |

## Current blockers

- ADR-0004 (media/codecs) and ADR-0008 (project license) are `PROPOSED`, not normative. Codec/patent and DRM decisions require counsel; the project license requires owner approval.
- Hub verification, durable-state, activation, and staging crates and an in-process fixture workflow exist, but no product application, installer, uninstaller, or launcher exists.
- GUI accessibility, text input, browser integration, and audio real-time behavior are `UNVERIFIED` until their prototypes run.
- No remote, signing infrastructure, or release channel exists.
- Windows and Linux are planned targets but have not been tested in this environment.
- The competitor, legal, and feasibility reviews are planning evidence, not implementation approval.
