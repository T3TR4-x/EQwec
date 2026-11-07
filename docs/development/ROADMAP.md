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

**State:** in progress. Workspace, `eqwec-core`, CI configuration, and local advisory/license checks exist; the gate is not closed until `dev-hub` is created and CI executes.

Deliverables:

- Approved `SYSTEM-ARCHITECTURE.md` and `DEPENDENCY-MAP.md`.
- Accepted ADRs for the first implementation decisions.
- Cargo workspace with a pinned toolchain. **Done.**
- Minimal shared crates with real APIs and tests. **`eqwec-core` done; 17 tests pass.**
- CI baseline: format, lint, build, test, advisory/license checks. **Configured; never executed because no remote exists.**
- `dev-*` and `production` branch creation from the approved baseline. **`dev-hub` next; `production` deferred until a verified release.**
- Initial version metadata at `0.0.0`. **Done.**
- Documentation and security gates reviewed against the actual workspace. **Done.**

Gate: reproducible local build and passing baseline checks. No product feature yet.

## Phase 3 — EQwec Hub MVP

**State:** not started.

Scope:

- Native launcher UI.
- Versioned, signed product manifest.
- Local installation registry.
- Verified download and package integrity.
- Transactional install, launch, update, rollback, repair, uninstall.
- Offline install and interrupted-operation recovery.
- Diagnostics with redaction.

Acceptance: install, verify, launch, update, interrupt/recover, repair, roll back, and uninstall a real fixture application on each claimed platform.

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
- No product code exists; the Phase 2 workspace and CI configuration exist.
- GUI accessibility, text input, browser integration, and audio real-time behavior are `UNVERIFIED` until their prototypes run.
- No remote, signing infrastructure, or release channel exists.
- Windows and Linux are planned targets but have not been tested in this environment.
- The competitor, legal, and feasibility reviews are planning evidence, not implementation approval.
