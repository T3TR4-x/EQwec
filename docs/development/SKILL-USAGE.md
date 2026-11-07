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
| Merge | PENDING | Commit not yet created at the time of writing |
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
