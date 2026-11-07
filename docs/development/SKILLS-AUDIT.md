# SKILLS-AUDIT.md — EQwec Skill Registry Audit

**Audit date:** 2026-11-07
**Branch:** `integration`
**Scope:** mandatory external skills, the 10 project-owned EQwec skills, discovery, validation, trigger behavior, and scenario coverage.
**Gate decision:** `SKILLS_GATE = PASS` for the bootstrap scope. The Phase 2 Cargo workspace exists, but runtime product scenarios are `BLOCKED` because no product code exists; they are not counted as passed. See §8.

## 1. Result summary

| Check | Result | Evidence |
|---|---|---|
| Exactly 10 project skill packages exist | PASS | `scripts/verify-skills.py`; 10/10 expected names |
| Frontmatter `name` matches directory | PASS | verifier; 10/10 |
| Official schema validation | PASS | `quick_validate.py`; 10/10 "Skill is valid!" |
| Description has trigger and non-trigger language | PASS | verifier; 0 failures, 0 warnings |
| Cross-references to core and quality skills | PASS | verifier; 10/10 |
| No placeholder text or absolute user paths | PASS | verifier; 0 failures |
| Active-agent discovery | PASS | OpenCode exposed all 10; `eqwec-vectorcraft` appeared dynamically after creation |
| Static positive/negative trigger cases | PASS | 20 positive, 18 negative; all description-coverage checks matched |
| Fresh-context trigger classification | PASS | 8/8 prompts classified correctly; 1 correct "no EQwec skill" |
| Realistic + edge scenario coverage | PASS (instruction coverage) | §7; runtime execution `BLOCKED` |
| Version-controlled registry | PASS after commit | Commit recorded in §9 |

**No product code exists.** The repository has a Phase 2 `Cargo.toml`, `crates/eqwec-core/`, `Cargo.lock`, and `.github/workflows/ci.yml`, but no `apps/`, product crate, installer, or release artifact. All product paths in the skills are explicitly marked planned.

## 2. Environment

| Fact | Value |
|---|---|
| OS / architecture | macOS 26.7.2, Apple silicon (arm64) |
| Agent | OpenCode 2.2.5 |
| Project skill discovery | `.agents/skills/<name>/SKILL.md` |
| Global skill discovery | `~/.agents/skills/` |
| Rust | 1.99.1; Cargo 1.99.1; Phase 2 workspace exists |
| Python | system `python3`; PyYAML is **not** installed there |
| Validation environment | temporary venv with PyYAML 6.x, used only for `quick_validate.py` |
| In-repo validation recipe | `python3 -m venv .venv-skillcheck` + `scripts/skill-validation-requirements.txt` |

The official `quick_validate.py` imports `yaml`. Running it with the system interpreter fails with `ModuleNotFoundError: No module named 'yaml'`. This is an environment prerequisite, not a skill defect. `scripts/verify-skills.py` now reports that condition explicitly and supports `--validator-python`.

## 3. External skills inventory

Sources below are recorded from the global installer lock `~/.agents/.skill-lock.json` and the bootstrap provenance comparison. Upstream commits marked "recorded" were verified in the earlier audit and were not re-fetched from the network in this session.

| # | Requirement | Installed IDs | Source | Upstream commit | License evidence | Status |
|---|---|---|---|---|---|---|
| 1 | Caveman | `caveman` (+ commit/explore/review variants) | `juliusbrussee/caveman` | not recorded | no LICENSE file | AVAILABLE |
| 2 | Superpowers | `using-superpowers`, `brainstorming`, `systematic-debugging`, `test-driven-development`, `writing-plans`, `executing-plans`, `verification-before-completion`, `requesting-code-review`, `receiving-code-review`, `subagent-driven-development`, `dispatching-parallel-agents`, `using-git-worktrees`, `finishing-a-development-branch`, `writing-skills`, `diagnosing-superpowers` | `obra/superpowers` | `22f0b46` (recorded) | no LICENSE in installed packages | AVAILABLE |
| 3 | Code-review | `code-review` | `mattpocock/skills` | `4ae2317` (recorded) | no LICENSE in installed package | AVAILABLE |
| 4 | Frontend-design | `frontend-design` | `anthropics/skills` | `2c5958e` (recorded) | `LICENSE.txt` present | AVAILABLE |
| 5 | UI/UX Pro Max | `ui-ux-pro-max` | `nextlevelbuilder/ui-ux-pro-max-skill` | not recorded | MIT | AVAILABLE |
| 6 | Playwright | `playwright-cli`, `playwright-best-practices` | `microsoft/playwright-cli`; `currents-dev/playwright-best-practices-skill` | not recorded | MIT declared by best-practices; CLI package has no LICENSE | AVAILABLE; CLI execution `BLOCKED` until the binary is verified |
| 7 | Ponytail | `ponytail`, `ponytail-audit`, `ponytail-review` | `dietrichgebert/ponytail` | not recorded | no LICENSE file | AVAILABLE |
| 8 | Matt Pocock Skills | `code-review`, `codebase-design`, `diagnosing-bugs`, `domain-modeling`, `implement-spec`, `improve-codebase-architecture`, `prototype`, `research`, `tdd`, `to-spec`, `grill-me` | `mattpocock/skills` | `4ae2317` (recorded) | no LICENSE in installed packages | AVAILABLE |
| 9 | Andrej Karpathy Skills | `karpathy-guidelines` | `szkocot/andrej-karpathy-skills` (community packaging) | not recorded | MIT declared in frontmatter | AVAILABLE |
| 10 | Unslop | `unslop` (nested at `unslop/skills/unslop/SKILL.md`) | `heydamir/unslop` | `a6f121e` (git HEAD) | CC-BY-4.0 | AVAILABLE |
| 11 | Security Audit | `security-audit` | `cloudflare/security-audit-skill` | `c1c8a8c` (recorded) | MIT | AVAILABLE |
| 12 | Skill Creator | `skill-creator` | `anthropics/skills` | not recorded | Apache-2.0 | AVAILABLE; `quick_validate.py` works, eval scripts `BLOCKED` |

### External-skill limitations

- **Caveman is not a deep-investigation skill.** Its installed instructions define a terse output voice. The original responsibility mapping in the master prompt was incorrect. Use Superpowers and Matt Pocock skills for investigation and planning; use Caveman only when a terse voice is explicitly requested.
- **Skill Creator eval loop is Claude-specific.** Its scripts invoke the Claude CLI (`claude -p`). Under OpenCode, the schema validator is usable; the full eval-viewer loop is `BLOCKED` and is not claimed.
- **Playwright CLI execution is unverified.** The skill instructions are present, but the `playwright-cli` executable has not been confirmed. Browser automation is `BLOCKED` until installed and tested.
- **License gaps.** Several installed packages contain no license file. This does not block internal use, but redistribution or bundling requires upstream license verification before release.
- **Commit pinning is incomplete.** The global installer lock records folder hashes, not commit SHAs, for most packages. `docs/development/SKILLS-LOCK.json` records local SHA-256 hashes so drift is detectable even where the upstream commit is unknown.

## 4. Project skills inventory

| Skill | Lines | Official validator | Discovery | Domain-specific evidence |
|---|---:|---|---|---|
| `eqwec-engineering-core` | 167 + 5 references | valid | exposed | Rust rules, branch governance, versions, documentation lifecycle, cross-product contracts |
| `eqwec-quality-control` | 145 + 3 references | valid | exposed | independent gates, PASS/FAIL/BLOCKED, waivers, findings, release gate |
| `eqwec-hub` | 327 | valid | exposed | signed manifests, transactional install, rollback, repair, offline, platform installers |
| `eqwec-browser` | 282 | valid | exposed | Chromium boundaries, profile isolation, IPC, tab lifecycle, memory, engine updates |
| `eqwec-photocraft` | 260 | valid | exposed | non-destructive graph, masks, RAW, color, tiles, project format, export |
| `eqwec-vectorcraft` | 305 | valid | exposed dynamically | Bézier geometry, tolerances, booleans, SVG security/fidelity, hit-testing, export |
| `eqwec-filmcraft` | 317 | valid | exposed | timeline, A/V sync, media, proxy/cache, render queue, export recovery |
| `eqwec-effectcraft` | 249 | valid | exposed | graph, keyframes, GPU/CPU parity, cache invalidation, FilmCraft interchange |
| `eqwec-craftanything` | 208 | valid | exposed | workspace manifests, canonical ownership, provenance, conflict handling, IPC |
| `eqwec-audiocraft` | 277 | valid | exposed | real-time callbacks, sample clock, MIDI, plugin containment, render, recovery |

All ten state that the product is planned and that paths are not implemented. No product skill claims a shipped capability.

## 5. Validation evidence

Commands actually executed:

```text
$ /private/var/folders/.../eqwec-skill-audit/venv/bin/python scripts/verify-skills.py \
    --validator ~/.agents/skills/skill-creator/scripts/quick_validate.py
PASS  eqwec-engineering-core        167 lines
PASS  eqwec-quality-control         145 lines
PASS  eqwec-hub                     327 lines
PASS  eqwec-browser                 282 lines
PASS  eqwec-photocraft              260 lines
PASS  eqwec-vectorcraft             305 lines
PASS  eqwec-filmcraft               317 lines
PASS  eqwec-effectcraft             249 lines
PASS  eqwec-craftanything           208 lines
PASS  eqwec-audiocraft              277 lines

RESULT: PASS (10 skills, 0 failures, 0 warnings)
exit=0
```

The official validator printed `Skill is valid!` for every package. The lock manifest was regenerated with `python3 scripts/update-skills-lock.py --validated`; it records SHA-256 per project skill and per installed external `SKILL.md`.

Reproduction:

```bash
python3 -m venv .venv-skillcheck
.venv-skillcheck/bin/pip install -r scripts/skill-validation-requirements.txt
.venv-skillcheck/bin/python scripts/verify-skills.py \
  --validator ~/.agents/skills/skill-creator/scripts/quick_validate.py
```

## 6. Trigger tests

### 6.1 Static description coverage

`docs/development/SKILL-TRIGGER-CASES.json` defines 20 positive and 18 negative cases. Each positive case requires at least two domain terms in the skill description; each negative case requires an explicit exclusion term. The verifier reported 0 failures and 0 warnings.

This is a **description-coverage test**, not proof of model activation. It catches missing trigger vocabulary and missing "do not use" clauses.

### 6.2 Fresh-context classification probe

A separate agent read only the ten frontmatter descriptions and classified eight task prompts. Result: 8/8 correct.

| Prompt | Expected | Probe result | Confidence |
|---|---|---|---|
| VectorCraft coincident-path boolean failure | `eqwec-vectorcraft` | correct, quoted "boolean operations, hit-testing" | high |
| Hub revoked-key update accepted | `eqwec-hub` | correct, quoted "installation, update, rollback... trust" | high |
| Typo in an unrelated repository | none | correct: no EQwec skill; core explicitly excludes unrelated repositories | high |
| AudioCraft real-time callback allocation | `eqwec-audiocraft` | correct, quoted "real-time callback, latency, xrun" | high |
| PhotoCraft lost non-destructive mask | `eqwec-photocraft` | correct, quoted "non-destructive", "masks", "native project format" | high |
| FilmCraft A/V drift + black export | `eqwec-filmcraft` | correct, quoted "A/V sync... proxy/cache... export" | high |
| EffectCraft stale GPU frame | `eqwec-effectcraft` | correct, quoted "keyframes... cache invalidation" | high |
| CraftAnything wrong editor + overwritten package | `eqwec-craftanything` | correct, quoted "canonical document ownership, provenance... conflict handling" | high |

The probe identified one expected overlap: FilmCraft and EffectCraft both mention caching. It resolved FilmCraft for proxy/render-cache failures and EffectCraft for a compositing-graph cache after a keyframe change. This is a documentation seam, not a defect; the audit records it for future trigger refinement.

### 6.3 Discovery

- OpenCode's active skill registry exposed all ten `eqwec-*` identifiers.
- `eqwec-vectorcraft` was created after the session started and appeared dynamically in the available-skills list, proving discovery is not limited to a stale session snapshot.
- File existence alone was not accepted as discovery evidence.

### 6.4 Not measured

Harness-level automatic activation (the exact routing decision the runtime makes before a skill is loaded) is not instrumented. The classification probe is a proxy, not a measurement of that internal decision. This is recorded as `BLOCKED`, not `PASS`.

## 7. Scenario tests (instruction coverage)

No product code exists; the Phase 2 workspace contains only `eqwec-core`. Runtime end-to-end scenarios therefore cannot execute. Each skill was tested at the instruction level: does it prescribe the controls needed for a realistic task and a failure/edge task? Verdict `PASS (instruction coverage)` means the required procedure and evidence are present; it does **not** mean the product behavior was executed.

| Skill | Realistic scenario | Edge / failure scenario | Evidence sections | Verdict |
|---|---|---|---|---|
| `eqwec-engineering-core` | Add a shared crate and assess every consumer | A product skill conflicts with core on security or Git | Repository map; Cross-product contracts; Git governance; Handoff | PASS (instruction coverage) |
| `eqwec-quality-control` | Review a diff before merge | Essential test cannot run in the environment | Gate model; Verification gate; Waivers; Findings | PASS (instruction coverage) |
| `eqwec-hub` | Interrupted update at every transaction phase | Revoked signer, tampered package, replay, downgrade | H-E2E-05, H-E2E-10, H-E2E-11; threat table | PASS (instruction coverage) |
| `eqwec-browser` | Tabs restore into the wrong profile after restart | Renderer crash or compromised renderer crosses IPC | Domain model; Threat boundaries; E2E workflows; failure modes | PASS (instruction coverage) |
| `eqwec-photocraft` | Non-destructive adjustment and mask survive save/reopen | Corrupt RAW or recovery file, memory exhaustion | Domain invariants; Threat boundaries; E2E workflows | PASS (instruction coverage) |
| `eqwec-vectorcraft` | Boolean subtract on coincident, tangent, holed paths | Hostile SVG with XXE, scripts, external refs, bombs | Geometry/numerics; SVG/interchange; V-E2E-03/05/11 | PASS (instruction coverage) |
| `eqwec-filmcraft` | Audio stays in sync after ripple trim | Black export frames after proxy-cache invalidation; interrupted render | Domain invariants; E2E workflows; render recovery | PASS (instruction coverage) |
| `eqwec-effectcraft` | Keyframe change invalidates the correct cache entries | GPU device loss and CPU/GPU divergence | Domain model; Required workflows; failure modes | PASS (instruction coverage) |
| `eqwec-craftanything` | Asset opens in its canonical owner and keeps provenance | Two products overwrite the same interchange package | Ownership split; invariants; conflict handling; E2E workflows | PASS (instruction coverage) |
| `eqwec-audiocraft` | No allocation or blocking in the real-time callback at 64 samples | Hosted plugin crashes during render; project recovery | Real-time invariants; plugin containment; E2E workflows | PASS (instruction coverage) |

Runtime product scenarios remain `BLOCKED` until product code exists. They must be executed before the affected product's release gate; they are not waived.

## 8. Conflicts, overlaps, and gate decision

### Resolved overlaps

- **Core vs product skills:** core wins on security, Git, versioning, and documentation. Product skills add domain constraints only. This is stated in every product skill.
- **`eqwec-quality-control` vs external `code-review`:** quality-control is the project gate and evidence standard; the external skill is a diff-review technique. A gate decision must use quality-control.
- **`skill-creator` vs Superpowers `writing-skills`:** skill-creator is authoritative for schema, validation, and packaging. `writing-skills` may inform prose quality but cannot override the schema.
- **Design skills:** `frontend-design` provides direction, `ui-ux-pro-max` provides searchable design data, `unslop` removes generic AI patterns. They are complementary; none is a default for every UI task.
- **Playwright:** `playwright-cli` is for browser automation; `playwright-best-practices` is for test design. Neither is used for native desktop GUI testing.

### Gate decision

```text
SKILLS_GATE = PASS (bootstrap scope)
```

Conditions met: all 10 packages exist, validate, are discoverable, have trigger/non-trigger contracts, pass 38 static trigger cases, pass an 8-prompt fresh-context classification probe, and have instruction-level realistic/edge scenario coverage.

Explicit limitations, not counted as passes:

- Harness-level automatic activation is not instrumented: `BLOCKED`.
- Runtime product end-to-end scenarios are `BLOCKED` until product code exists.
- Playwright CLI execution is `BLOCKED` until the binary is verified.
- Skill Creator's Claude-based eval loop is `BLOCKED` under OpenCode; schema validation is `PASS`.
- Some external licenses and upstream commits are not pinned; recorded as release-risk, not as a skills-gate failure.

This gate authorizes the research and architecture planning phases. It does **not** authorize product implementation; the Phase 2 workspace gate and the Hub acceptance workflow must close first.

## 9. Traceability

- Canonical registry: `.agents/skills/`
- Machine manifest: `docs/development/SKILLS-LOCK.json`
- Trigger fixtures: `docs/development/SKILL-TRIGGER-CASES.json`
- Preflight: `scripts/verify-skills.py`
- Lock generator: `scripts/update-skills-lock.py`
- Validation dependency: `scripts/skill-validation-requirements.txt`
- Agent entry points: `AGENTS.md`, `CLAUDE.md`
- Commit: `docs(skills): завершити реєстр і перевірку десяти навичок` on `integration`; resolve the current SHA with `git log --oneline -- .agents/skills docs/development`.
