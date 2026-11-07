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
