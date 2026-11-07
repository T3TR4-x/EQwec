# SKILL-USAGE.md — Activation Matrix

This file records which skills were actually applied to a substantial task, which were available but not relevant, and which were blocked. It is evidence, not a checklist to copy. Update it per task; do not claim `APPLIED` without an artifact.

## Task: bootstrap skill registry and agent instructions

**Date:** 2026-11-07
**Branch:** `integration`
**Scope:** create/validate the 10 project skills, external skill audit, agent entry points, lock manifest, trigger and scenario tests. **No product code.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Branch/`main` constraints, Rust and security rules, documentation lifecycle, and Ukrainian commit policy used in `AGENTS.md`, `CLAUDE.md`, and this audit |
| `eqwec-quality-control` | APPLIED | Gate model and evidence standard used; PASS/BLOCKED distinction enforced; see report below |
| `eqwec-vectorcraft` | APPLIED | Authored and reviewed; 305 lines; validator PASS; V-E2E-01..12 and security table verified |
| `eqwec-hub` | AVAILABLE_NOT_RELEVANT | No Hub path, manifest, installer, or update code touched |
| `eqwec-browser` | AVAILABLE_NOT_RELEVANT | No browser path, engine, IPC, profile, or session code touched |
| `eqwec-photocraft` | AVAILABLE_NOT_RELEVANT | No photo document, mask, RAW, color, or export code touched |
| `eqwec-filmcraft` | AVAILABLE_NOT_RELEVANT | No timeline, media, render, or export code touched |
| `eqwec-effectcraft` | AVAILABLE_NOT_RELEVANT | No compositing, keyframe, GPU, or cache code touched |
| `eqwec-craftanything` | AVAILABLE_NOT_RELEVANT | No workspace, asset-exchange, provenance, or IPC code touched |
| `eqwec-audiocraft` | AVAILABLE_NOT_RELEVANT | No audio, MIDI, plugin, or render code touched |
| `skill-creator` | APPLIED | Schema conventions and `quick_validate.py` used for all 10 packages; eval loop `BLOCKED` (Claude CLI) |
| `superpowers` | AVAILABLE_NOT_RELEVANT | No implementation plan or code change requiring its workflow; `writing-skills` not used as the schema authority |
| `code-review` (Matt Pocock) | AVAILABLE_NOT_RELEVANT | No product diff; skill content was reviewed against the project rubric instead |
| `frontend-design` | AVAILABLE_NOT_RELEVANT | No UI implemented |
| `ui-ux-pro-max` | AVAILABLE_NOT_RELEVANT | No UI or design system implemented |
| `unslop` | AVAILABLE_NOT_RELEVANT | No UI implemented |
| `playwright` | AVAILABLE_NOT_RELEVANT | No web UI or browser test; CLI execution also unverified (`BLOCKED` for use) |
| `ponytail` | AVAILABLE_NOT_RELEVANT | No code refactor; minimal-change discipline already enforced by core |
| `karpathy-guidelines` | AVAILABLE_NOT_RELEVANT | No code implementation; assumptions and scope were recorded directly |
| `caveman` | AVAILABLE_NOT_RELEVANT | Terse voice not requested; it is not an investigation skill |
| `security-audit` | AVAILABLE_NOT_RELEVANT | No product trust boundary reviewed; skill provenance was audited, not application security |

## Quality-control report for this task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | Read `README.md`, branch state, `.agents/skills/`, docs skeleton; confirmed no workspace and no product implementation |
| Implementation | PASS | Changes limited to skills, docs, scripts, `.gitignore`, `.gitattributes`, `AGENTS.md`, `CLAUDE.md`; no product code or empty crates |
| Verification | PASS | `scripts/verify-skills.py` PASS 10/10; `quick_validate.py` valid 10/10; 38 static trigger cases; 8/8 classification probe |
| Performance | NOT_APPLICABLE | No runtime code or benchmark; no performance claim made |
| Merge | PASS after commit review | Diff scope, Ukrainian commit message, and branch checked before commit |
| Release | NOT_APPLICABLE | No product release; versions remain `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| Medium | Several external packages lack a license file; redistribution requires upstream verification | Open; release-risk, not a bootstrap blocker |
| Medium | `playwright-cli` executable is not verified | Open; browser automation is `BLOCKED` until tested |
| Medium | Skill Creator eval loop requires the Claude CLI and is unavailable under OpenCode | Open; schema validation still passes |
| Low | FilmCraft and EffectCraft descriptions both mention caching | Open; probe resolved the boundary; monitor trigger accuracy |
| Low | Harness-level automatic activation is not instrumented | Open; classification probe is a proxy, not a measurement |

Gate decision: `PASS` for the bootstrap scope. Runtime product scenarios remain `BLOCKED` and are release prerequisites.

## Task: research, technology selection, and ADRs 0002–0010

**Date:** 2026-11-07
**Branch:** `integration`
**Scope:** browser/GUI/media/audio research, competitor and legal review, feasibility synthesis, technology selection, and architecture decision records. **No product code, no Cargo workspace.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Evidence labels, gate honesty, branch rules, security and documentation policy applied across all research and ADR files |
| `eqwec-quality-control` | APPLIED | PASS/FAIL/BLOCKED separation enforced; unverified prototype and legal claims marked `BLOCKED`; see report below |
| `eqwec-browser` | APPLIED | `BROWSER-ENGINE-FEASIBILITY.md`, ADR-0003, ADR-0010; CEF boundary, sandbox/site isolation, profile isolation, extension limits |
| `eqwec-hub` | APPLIED | ADR-0007 packaging/signing; Hub trust chain, rollback, offline install, key custody |
| `eqwec-photocraft` | APPLIED | ADR-0004 image/RAW/color path; PhotoCraft feasibility and color/tile constraints |
| `eqwec-vectorcraft` | APPLIED | ADR-0002 geometry/text stack; VectorCraft feasibility and first-proof criteria |
| `eqwec-filmcraft` | APPLIED | ADR-0004 media path; FilmCraft feasibility, A/V sync, export and legal gates |
| `eqwec-effectcraft` | APPLIED | ADR-0004 and ADR-0010; EffectCraft feasibility, cache, compositing and plugin trust |
| `eqwec-craftanything` | APPLIED | ADR-0006 interchange and provenance; CraftAnything feasibility and ownership boundary |
| `eqwec-audiocraft` | APPLIED | ADR-0005 and ADR-0010; real-time callback rules, CLAP/VST3 hosting, crash isolation |
| `skill-creator` | AVAILABLE_NOT_RELEVANT | No skill created or edited in this task; validation was not re-run because skill files did not change |
| `research` | APPLIED | Primary-source research method used for browser, competitor, and legal reviews; sources and access dates recorded |
| `code-review` (Matt Pocock) | AVAILABLE_NOT_RELEVANT | No code diff; documents were reviewed against ADR and evidence requirements instead |
| `security-audit` | APPLIED | Threat boundaries, trust tiers, plugin isolation, crash-data sensitivity, and update-chain controls reviewed in ADRs 0003/0007/0009/0010 |
| `ui-ux-pro-max` | AVAILABLE_NOT_RELEVANT | Technology evaluation only; no interface designed |
| `frontend-design` | AVAILABLE_NOT_RELEVANT | No UI implemented |
| `unslop` | AVAILABLE_NOT_RELEVANT | No UI implemented |
| `playwright` | AVAILABLE_NOT_RELEVANT | No browser test executed; CLI remains unverified |
| `superpowers`, `ponytail`, `karpathy-guidelines`, `caveman` | AVAILABLE_NOT_RELEVANT | No implementation plan, refactor, code change, or terse-voice request |

## Quality-control report for the research and ADR task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | Branch and worktree inspected; existing research read before writing; no product code touched |
| Implementation | PASS | 4 research docs, 1 technology-selection doc, 9 ADRs; each ADR has context, alternatives, evidence, decision, consequences, and reconsideration criteria |
| Verification | PASS with blockers | `python3 scripts/check-docs.py --gate` run; structural checks pass. Prototype, legal, and runtime claims remain `BLOCKED` |
| Performance | NOT_APPLICABLE | No runtime code or benchmark; numeric targets remain hypotheses |
| Security | PASS for planning | Threat model and security baseline reconciled; plugin trust, update signing, and crash-data sensitivity covered |
| Merge | PASS | Diff reviewed; scoped Ukrainian Conventional Commits created on `integration`; no unrelated files staged |
| Release | NOT_APPLICABLE | No product release; versions remain `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| High | Codec and DRM licensing requires counsel; H.264/AAC/HEVC/ProRes/DNxHR and DRM are not approved | Open; `BLOCKED` |
| High | Project license is `PROPOSED`; redistribution remains unauthorized until ADR-0008 is accepted | Open; `BLOCKED` on owner |
| High | Browser engine maintenance, GUI accessibility/IME, and audio real-time behavior are unproven | Open; `BLOCKED` on prototypes |
| Medium | Windows and Linux were not exercised; platform claims are unverified | Open; `BLOCKED` |
| Medium | Competitor analysis is documentation-based; no product was benchmarked or used | Accepted limitation; stated in the document |
| Low | Some third-party license versions require admission-time verification | Open; dependency admission policy covers it |

Gate decision: `PASS` for the planning scope. Product implementation remains `BLOCKED` until the workspace and prototype gates exist.

## Task: Phase 2 workspace baseline

**Date:** 2026-11-07
**Branch:** `integration`
**Scope:** Cargo workspace, pinned toolchain, `eqwec-core`, CI workflow, advisory/license policy, and stale-document reconciliation. **No product code.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Resolver 3, edition 2024, pinned 1.99.1, `forbid(unsafe_code)`, typed errors, scoped Ukrainian commits |
| `eqwec-quality-control` | APPLIED | Local gate evidence recorded in `QUALITY-GATES.md`; CI execution kept `BLOCKED`; no invented pass |
| `eqwec-hub` | AVAILABLE_NOT_RELEVANT | No Hub crate, manifest, installer, or update path exists yet |
| `eqwec-browser` | AVAILABLE_NOT_RELEVANT | No engine, IPC, profile, or session code touched |
| `eqwec-photocraft` | AVAILABLE_NOT_RELEVANT | No photo document, mask, RAW, color, or export code touched |
| `eqwec-vectorcraft` | AVAILABLE_NOT_RELEVANT | No geometry, SVG, renderer, or vector document code touched |
| `eqwec-filmcraft` | AVAILABLE_NOT_RELEVANT | No timeline, media, render, or export code touched |
| `eqwec-effectcraft` | AVAILABLE_NOT_RELEVANT | No compositing, keyframe, GPU, or cache code touched |
| `eqwec-craftanything` | AVAILABLE_NOT_RELEVANT | No workspace, asset-exchange, provenance, or IPC code touched |
| `eqwec-audiocraft` | AVAILABLE_NOT_RELEVANT | No audio, MIDI, plugin, or render code touched |
| `skill-creator` | APPLIED | Skill status lines updated; `verify-skills.py` plus official `quick_validate.py` rerun, 10/10 valid |
| `security-audit` | APPLIED | `deny.toml` scoped private-crate exemption, no third-party dependencies, unsafe code forbidden |
| `code-review`, `ponytail`, `karpathy-guidelines`, `research`, UI skills, `playwright` | AVAILABLE_NOT_RELEVANT | No product diff, UI, browser test, or research task in this scope |

## Quality-control report for the Phase 2 task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | Branch and worktree inspected; no product code touched |
| Implementation | PASS | Workspace, `eqwec-core`, 17 tests, CI workflow, `deny.toml`; diff scoped |
| Verification | PASS locally; CI `BLOCKED` | `check-docs.py --gate` 0 failures; `verify-skills.py` 10/10; fmt, clippy, build, test, audit, and deny exit 0 |
| Performance | NOT_APPLICABLE | No runtime product code or benchmark |
| Security | PASS for baseline | No third-party dependencies; `unsafe_code = "forbid"`; advisory and license checks pass |
| Merge | PENDING | Commit and diff review not yet completed at the time of writing |
| Release | NOT_APPLICABLE | No product release; versions remain `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| High | CI has never executed; platform and pipeline behavior are unverified | Open; `BLOCKED` on a remote |
| High | ADR-0008 is `PROPOSED`; `eqwec-core` has no license field and `deny.toml` exempts private crates only | Open; `BLOCKED` on owner |
| Medium | Windows and Linux were not exercised locally | Open; `BLOCKED` |
| Low | `cargo deny` reports unused license allowances because no third-party crate exists yet | Expected; recheck when dependencies are added |

Gate decision: workspace baseline `PASS` locally. The Phase 2 gate is not closed until CI runs and `dev-hub` exists.

## Task: Hub verification core (manifest, trust, package)

**Date:** 2026-11-07
**Branch:** `dev-hub`
**Scope:** ADR-0011, normative Hub schemas, `eqwec-security`, `eqwec-updater`, fixed signature vector, and integration tests. **No installer, registry, journal, UI, or product code.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Fail-closed parsing, bounded input, typed errors, `forbid(unsafe_code)`, dependency admission, scoped Ukrainian commit policy |
| `eqwec-quality-control` | APPLIED | Local gate evidence recorded; install acceptance kept `BLOCKED`; fixed vector independently generated with OpenSSL |
| `eqwec-hub` | APPLIED | ADR-0011, `eqwec-hub-schemas.md`, manifest/trust/package implementation, rollback and activation rules |
| `security-audit` | APPLIED | Duplicate/unknown field rejection, strict Ed25519, weak-key rejection, path traversal rules, revocation and epoch checks, key-custody limits |
| `skill-creator` | AVAILABLE_NOT_RELEVANT | No skill file changed in this task |
| `code-review` (Matt Pocock) | APPLIED | Diff reviewed for private-field leakage, canonicalization order, and test coverage gaps |
| `ponytail`, `karpathy-guidelines` | AVAILABLE_NOT_RELEVANT | Implementation followed the schema directly; no refactor or design exploration |
| `eqwec-browser`, `eqwec-photocraft`, `eqwec-vectorcraft`, `eqwec-filmcraft`, `eqwec-effectcraft`, `eqwec-craftanything`, `eqwec-audiocraft` | AVAILABLE_NOT_RELEVANT | No path in those products was touched |
| `research`, UI skills, `playwright` | AVAILABLE_NOT_RELEVANT | No research question, UI, or browser test in this scope |

## Quality-control report for the Hub verification task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | ADR-0011 and schema doc read first; branch `dev-hub` at `0cc8d59`; no unrelated worktree changes |
| Implementation | PASS for verification core | Three crates; six admitted dependencies; no installer or filesystem mutation claimed |
| Verification | PASS locally; install acceptance `BLOCKED` | `cargo test --workspace --all-features --locked`: 76 passed, 0 failed; fmt, clippy, build, audit, deny exit 0; docs gate 0 failures; skills 10/10 |
| Performance | NOT_APPLICABLE | No runtime product workload or benchmark |
| Security | PASS for implemented boundary | Fixed OpenSSL vector, tampered payload/signature, wrong channel/target/key/epoch, revoked key, expired key, sequence regression, truncation, size and hash mismatch all rejected |
| Merge | PASS | Scoped Ukrainian Conventional Commit `2a47df8` created on `dev-hub`; no unrelated files staged |
| Release | NOT_APPLICABLE | No artifact; versions remain `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| High | Registry, journal, atomic activation, installer, and fixture product do not exist | Open; install/update/rollback acceptance `BLOCKED` |
| High | CI has never executed; local-only evidence | Open; `BLOCKED` on a remote |
| High | Trust-store update signing and root-key custody do not exist | Open; on-disk trust updates `BLOCKED` |
| Medium | Windows and Linux were not exercised | Open; `BLOCKED` |
| Low | `cargo deny` reports duplicate `syn` 2.x/3.x | Accepted; isolated to proc-macro expansion |

Gate decision: verification core `PASS` locally. Hub MVP acceptance remains `BLOCKED`.

## Task: Hub durable-state primitives (registry, journal, layout, active pointer)

**Date:** 2026-11-07
**Branch:** `dev-hub`
**Scope:** `atomic.rs`, `layout.rs`, `journal.rs`, `registry.rs`, `active.rs`, error variants, and schema reconciliation. **No activation orchestration, installer, UI, or fixture product.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Bounded files, typed errors, atomic replacement, fail-closed corrupt journal, no `unsafe`, scoped Ukrainian commit policy |
| `eqwec-quality-control` | APPLIED | 104 local tests pass; activation and install acceptance kept `BLOCKED`; no CI claim |
| `eqwec-hub` | APPLIED | Schema §4–§7 implemented; journal recovery mapping; layout containment; active-pointer rules |
| `security-audit` | APPLIED | Traversal and unnormalized-path rejection, size bounds, partial-line handling, fail-closed malformed records |
| `skill-creator` | APPLIED | `eqwec-hub` status text and crate table corrected to the real workspace state; validators rerun |
| `code-review` (Matt Pocock) | APPLIED | Diff reviewed for path normalization, temp-file cleanup, and recovery correctness |
| Other product skills | AVAILABLE_NOT_RELEVANT | No path in those products was touched |

## Quality-control report for the durable-state task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | ADR-0011 and schema §4–§7 read; branch `dev-hub`; worktree inspected |
| Implementation | PASS for primitives | Registry, journal, layout, active pointer, and atomic write implemented with bounds |
| Verification | PASS locally; activation `BLOCKED` | `cargo test --workspace --all-features --locked`: 104 passed, 0 failed; fmt, clippy, build, audit, deny exit 0; docs gate 0 failures; skills 10/10 |
| Performance | NOT_APPLICABLE | No runtime product workload |
| Security | PASS for implemented boundary | Unnormalized/relative roots, traversal trees, duplicate/unsorted records, oversized files, partial and corrupt journal lines, unknown fields, and invalid phases are rejected |
| Merge | PENDING | Commit not yet created |
| Release | NOT_APPLICABLE | No artifact; versions remain `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| High | Activation transaction, recovery orchestration, and fault injection do not exist | Open; install/update/rollback acceptance `BLOCKED` |
| High | No fixture product or signed v1/v2 packages | Open; H-E2E scenarios `BLOCKED` |
| Medium | Registry and journal retention/pruning are specified but not enforced by a runtime | Open; needs the activation engine |
| Low | `Path::components()` normalizes `.`; lexical checks now cover it | Fixed; regression test added |

Gate decision: durable-state primitives `PASS` locally. Hub MVP acceptance remains `BLOCKED`.

## Task: Hub activation, rollback, and recovery engine

**Date:** 2026-11-07
**Branch:** `dev-hub`
**Scope:** `activation.rs`, activation tests, journal/registry identity fields, typed errors, and schema §4.2/§5.2/§5.4/§10. **No download transport, extraction, uninstall, launcher UI, or fixture product.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Journal record before every mutation, atomic pointer and registry replace, fail-closed recovery, typed errors, bounded inputs, no `unsafe`, scoped Ukrainian commit policy |
| `eqwec-quality-control` | APPLIED | 129 local tests pass; installer and end-to-end acceptance kept `BLOCKED`; no CI claim |
| `eqwec-hub` | APPLIED | Fresh install, update, rollback, and startup recovery follow schema §5.4; retained trees are preserved; single-in-flight invariant |
| `security-audit` | APPLIED | Symlinked staging rejected, unexpected pointer conflicts fail closed, rollback target cannot be deleted, fault injection covers every durable phase |
| `skill-creator` | APPLIED | `eqwec-hub` status and crate table updated to the real workspace state; validators rerun |
| `code-review` (Matt Pocock) | APPLIED | Review and adversarial recovery testing fixed: rollback recovery deleting a retained tree, recovery losing the pointer activation time, a second in-flight operation being accepted, a rollback after `registry-committed` being marked `committed`, a missing source record being silently recreated, and an unbound `staged` record identity. Each has a regression test |
| Other product skills | AVAILABLE_NOT_RELEVANT | No path in those products was touched |

## Quality-control report for the activation task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | ADR-0011 and schema §4–§6 read; branch `dev-hub`; worktree inspected |
| Implementation | PASS for the in-process engine | `activate`, `rollback`, `recover`, `record_phase`, five fault points, retention, pruning, and recovery |
| Verification | PASS locally; installer `BLOCKED` | `cargo test --workspace --all-features --locked`: 129 passed, 0 failed; fmt, clippy, build, audit, deny exit 0; docs gate 0 failures; skills 10/10 |
| Performance | NOT_APPLICABLE | No runtime product workload |
| Security | PASS for implemented boundary | Missing, symlinked, and pre-existing trees rejected; downgrade rejected; corrupt or partial journal fails closed; incomplete manifest identity fails closed; unexpected active pointer fails closed; retained rollback target preserved |
| Merge | PENDING | Commit not yet created |
| Release | NOT_APPLICABLE | No artifact; versions remain `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| High | No download transport, extraction, uninstall, launcher, or fixture product | Open; install/update/rollback acceptance `BLOCKED` |
| High | No signed v1/v2 fixture packages or H-E2E scenarios | Open |
| Medium | Installed tree contents are not re-hashed after activation; repair and integrity re-verification do not exist | Open |
| Medium | The engine assumes exclusive access to one profile; no profile-lock implementation exists | Open |
| Low | First rollback-recovery draft deleted the retained target tree | Fixed; regression test added |
| Low | Recovery overwrote the pointer activation time with recovery time | Fixed; regression test added |
| Low | A second operation could start while another was unfinished | Fixed; single-in-flight check and test added |
| Low | A rollback after `registry-committed` was marked `committed` instead of `rolled-back` | Fixed; recovery validates pointer, registry, and tree, with a regression test |
| Low | Recovery could recreate an update record when the source registry record was missing | Fixed; `commit_registry` fails closed with `NoInstallation` |
| Low | `activate` did not bind the recorded `staged` phase to the release identity | Fixed; context equality check with a regression test |

Gate decision: activation engine `PASS` locally. Hub MVP acceptance remains `BLOCKED`.

## Package staging task

**Date:** 2026-11-07
**Branch:** `dev-hub`
**Scope:** `staging.rs`, staging tests, `layout.rs` download-package path, ADR-0012, schema §11, typed archive errors, and the `zip` dependency. **No download transport, uninstall, launcher UI, or fixture product.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Same-handle verification and parsing, bounded parser, no `unsafe`, typed errors, fail-closed cleanup, scoped Ukrainian commit policy |
| `eqwec-quality-control` | APPLIED | 160 local tests pass; installer and end-to-end acceptance kept `BLOCKED`; no CI claim; review found and fixed three defects |
| `eqwec-hub` | APPLIED | ADR-0012 and schema §11 enforced: path, mode, header, compression, size, ratio, timeout, staging, and journal rules |
| `security-audit` | AVAILABLE_NOT_RELEVANT | Archive threat review performed directly against ADR-0012 and the Hub threat table; the external workflow was not invoked |
| `code-review` (Matt Pocock) | APPLIED | Review fixed ZIP64 contiguity, reader-rewind, and directory-mode verification defects; each has a regression test |
| `skill-creator` | AVAILABLE_NOT_RELEVANT | No skill wording changed |
| Other product skills | AVAILABLE_NOT_RELEVANT | No path in those products was touched |

## Quality-control report for the package staging task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | ADR-0012, schema §11, and `eqwec-hub` loaded; branch and worktree inspected before editing |
| Implementation | PASS for the in-process staging engine | Central-directory preflight, two-pass validation/extraction, durable sync, tree verification, journal `verified`/`staged` integration |
| Verification | PASS locally; installer `BLOCKED` | `cargo test --workspace --all-features --locked`: 160 passed, 0 failed; fmt, clippy, build, audit, deny, docs gate, and skills gate exit 0 |
| Performance | NOT_APPLICABLE | No throughput claim; deflate is single-threaded and no benchmark was run |
| Security | PASS for implemented boundary | Same handle hashed and parsed, traversal/link/mode/header/ratio bounds enforced, partial trees removed, no unverified binary executed |
| Merge | PASS | Scoped commit on `dev-hub`; staging, tests, ADR, dependency inventory, and status documents only |
| Release | NOT_APPLICABLE | No artifact; versions remain `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| Medium | Staging does not preflight free disk space; the install orchestrator must do it before download or staging | Open; `BLOCKED` on the transport/install layer |
| Medium | No real fixture product or platform package; ADR-0012 gate 12 remains unmet | Open; H-E2E scenarios `BLOCKED` |
| Low | ZIP64 contiguity compared the central-directory end with the locator instead of the ZIP64 end record start | Fixed; ZIP64 regression test added |
| Low | The package reader was hashed from its current position but parsed from byte zero | Fixed; rewind plus regression test |
| Low | Staged directory modes were created but not re-verified | Fixed; tree verification now checks them |
| Low | `stage_package_from_path` checks for a symlink and then opens by path; a same-user swap race remains | Open; profile directory is mode `0700`, and `O_NOFOLLOW` is not yet used |

Gate decision: package staging `PASS` locally. Hub MVP acceptance remains `BLOCKED`.

## Task: Hub fixture product and end-to-end install/update/rollback tests

**Date:** 2026-11-07
**Branch:** `dev-hub`
**Scope:** `tests/fixtures/eqwec-hub/` workspace member, two real fixture executables, signed v1/v2 packages with a public non-production seed, three end-to-end tests, root workspace membership, and status reconciliation in ADR-0012, the Hub schema document, the roadmap, and the Hub skill. **No download transport, uninstall, launcher UI, installer packaging, or production key.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Fixture isolated as test infrastructure, no production key, bounded package use, scoped Ukrainian commit policy, no product claim beyond evidence |
| `eqwec-quality-control` | APPLIED | 163 local tests pass; gate 12 kept `BLOCKED` overall; no CI or platform claim; test-key isolation and tampered-package behavior verified |
| `eqwec-hub` | APPLIED | H-E2E-01/03/04/05/07/10 partially exercised in process; fixture requirements for real v1/v2 executables and non-production keys followed |
| `security-audit` | APPLIED | Confirmed the seed is public test material, manifests are signed and verified before staging, tampered package fails with `PackageHashMismatch`, and no unverified binary executes |
| `code-review` (Matt Pocock) | APPLIED | Review checked exact stdout assertions, retained-version state, recovery cleanup, and `FaultInjected` typing; no production path was changed |
| `skill-creator` | APPLIED | `eqwec-hub` status, ownership table, and fixture row corrected; `verify-skills.py` rerun |
| Other product skills | AVAILABLE_NOT_RELEVANT | No path in those products was touched |
| UI, browser, research, and Playwright skills | AVAILABLE_NOT_RELEVANT | No UI, browser test, or research question in this scope |

## Quality-control report for the fixture task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | Branch `dev-hub` at `c281d9a`; worktree inspected; ADR-0012 gate 12 and Hub fixture requirements read before writing |
| Implementation | PASS for the in-process fixture workflow | Real v1/v2 executables read `share/fixture.txt`; signed manifests; install, launch, update, rollback, tampered-package rejection, and interrupted-activation recovery |
| Verification | PASS locally; platform acceptance `BLOCKED` | `cargo test --workspace --all-features --locked`: 163 passed, 0 failed; fmt, clippy, build, audit, deny, docs gate, and skills gate exit 0 |
| Performance | NOT_APPLICABLE | No startup, throughput, or memory measurement; fixture tests assert behavior only |
| Security | PASS for the tested boundary | Public non-production seed; signature and hash verified before staging; tampered package rejected; active v1 unchanged; interrupted activation recovers to v1 |
| Merge | PASS | Scoped commit created on `dev-hub`; fixture, workspace membership, docs, and Hub skill only |
| Release | NOT_APPLICABLE | No artifact; fixture is non-published test infrastructure; version remains `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| High | ADR-0012 gate 12 is only partially met: uninstall, installer orchestration, download transport, platform packaging, and Windows/Linux runs do not exist | Open; overall gate remains `BLOCKED` |
| High | The fixture runs in process, not from a signed platform installer; shortcuts, launcher listing, and per-platform quarantine/Gatekeeper behavior are unverified | Open; `BLOCKED` on installer and launcher work |
| Medium | Only the `AfterTreeMove` activation interruption is exercised end to end; the full H-E2E-05 phase matrix remains open | Open |
| Medium | Staging still lacks a disk-space preflight and retains the same-user symlink-swap TOCTOU window noted in the staging task | Open |
| Low | The fixture seed is public and must never be reused for release signing | Accepted; documented in the test module and skill |

Gate decision: fixture workflow `PASS` locally. Hub MVP acceptance remains `BLOCKED`.

## Task: First GitHub Actions run and Windows portability fixes

**Date:** 2026-11-07
**Branch:** `dev-hub`, fast-forwarded to `integration`
**Scope:** Windows clippy fixes in `atomic.rs` and `staging.rs`, platform-neutral path fixtures in `layout.rs` and `registry.rs`, CI run `42389844319`, and status reconciliation in the root documents and gate reports. **No production behavior change.**

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | Inspected branch and worktree, kept the fix scoped, used typed test paths, and recorded the real CI result without inventing platform acceptance |
| `eqwec-quality-control` | APPLIED | Failure logs from run `42388967215` reproduced locally by Windows-target clippy; final run `42389844319` passed all five jobs; product acceptance kept `BLOCKED` |
| `eqwec-hub` | APPLIED | Only Hub workspace crates and fixture-adjacent tests were touched; no Hub capability was claimed beyond the tested workflow |
| Other product skills | AVAILABLE_NOT_RELEVANT | No path in another product was touched |
| UI, browser, research, and Playwright skills | AVAILABLE_NOT_RELEVANT | No UI, browser test, or research question in this scope |

## Quality-control report for the CI task

| Gate | Decision | Evidence |
|---|---|---|
| Pre-change | PASS | Branch `dev-hub` at `9df033d`; worktree inspected; Windows failure logs from run `42388967215` read before editing |
| Implementation | PASS | `PathBuf` import scoped to tests; `collect_directories` gated to Unix; mode assertions run on all platforms; layout and registry tests derive absolute paths from `temp_dir()` |
| Verification | PASS | Local fmt, clippy, build, and 163 tests pass; Windows-target clippy passes; GitHub Actions run `42389844319` on commit `c066d67` passed macOS, Ubuntu, Windows, advisories, and licenses/bans |
| Performance | NOT_APPLICABLE | No runtime or benchmark change |
| Security | PASS | `validate_absolute` was not weakened; Windows tests now exercise real absolute-path validation instead of failing before the intended assertion |
| Merge | PASS | Commits `288b27a` and `c066d67` on `dev-hub`; `integration` fast-forwarded to `c066d67` |
| Release | NOT_APPLICABLE | No artifact; version remains `0.0.0` |

Findings:

| Severity | Finding | State |
|---|---|---|
| Medium | GitHub did not discover `.github/workflows/ci.yml` until the default branch changed and a workflow file was created through the API | Worked around; the temporary probe was removed in `2fa0203` |
| Low | Windows verification is compile-only locally; runtime test execution depends on GitHub-hosted runners | Accepted; run `42389844319` is the runtime evidence |
| High | Windows and Linux product-runtime acceptance remains impossible because no Hub application or installer exists | Open; `BLOCKED` |

Gate decision: CI `PASS` for commit `c066d67`. Hub MVP acceptance remains `BLOCKED`.

## Template for future tasks

| Skill | State | Evidence |
|---|---|---|
| `eqwec-engineering-core` | APPLIED | |
| `eqwec-quality-control` | APPLIED | |
| `<affected product skill>` | APPLIED / AVAILABLE_NOT_RELEVANT / BLOCKED | |
| `<external skill>` | APPLIED / AVAILABLE_NOT_RELEVANT / BLOCKED | |

Rules:

- A missing mandatory skill is `BLOCKED`, never `NOT_RELEVANT`.
- `AVAILABLE_NOT_RELEVANT` needs a one-line reason tied to the actual scope.
- `APPLIED` needs a concrete artifact: a referenced rule, a test, a review finding, or a changed file.
- A checker can prove files exist; it cannot prove instructions were followed.
