# Agent Instructions Policy

**Status:** approved. This document explains how agent instructions are authored, loaded, reviewed, and kept consistent. It does not replace `AGENTS.md`.

## 1. Canonical source

`AGENTS.md` is the single canonical agent entry point for the repository. `CLAUDE.md` is a Claude-specific entry point that defers to `AGENTS.md`; it may add loading notes but must not contradict the canonical rules.

The project skill registry is `.agents/skills/`. It is the canonical, version-controlled source for the 10 EQwec skills. Do not maintain a second editable copy in another agent-specific directory. If another discovery path is required, generate or synchronize it from `.agents/skills/` and test for drift.

## 2. Precedence

When instructions conflict, apply this order:

1. Platform and agent security policy, law, and user authorization.
2. `AGENTS.md` and `eqwec-engineering-core`.
3. `eqwec-quality-control` gate rules.
4. The affected product skill.
5. Task-specific instructions and local preferences.

A product skill may add domain constraints but may not weaken security, Git, versioning, documentation, or quality rules. A conflict is a defect to report and fix, not a rule to silently ignore.

## 3. Required reading order

For every substantial task:

1. `AGENTS.md`.
2. `eqwec-engineering-core`.
3. `eqwec-quality-control`.
4. The skill for each affected product.
5. Relevant ADRs, product specs, security/performance documents, and the actual code.

Do not claim a skill was applied because its name was mentioned. Record evidence in `docs/development/SKILL-USAGE.md`.

## 4. Change control

Before editing an instruction file:

- Read the current file and the relevant skill. Preserve valid user-maintained content.
- Identify the exact rule that changes and why.
- Check for contradictions with `AGENTS.md`, `CLAUDE.md`, skills, ADRs, and current repository state.
- Update only impacted sections. Do not rewrite every document on every commit.
- Verify that commands, paths, versions, and links are real.
- Review the diff and record the change in the commit message.

After a significant architecture, API, Git, skill, or release change:

- Inspect the impacted Markdown and skills.
- Update factual sections or record why no change is needed.
- Re-run skill validation and the documentation checks that exist.
- If `AGENTS.md` and `CLAUDE.md` disagree, fix the disagreement in the same change.

## 5. What agents must not do

- Do not begin product implementation before the skills, documentation, architecture, and security gates pass.
- Do not add `AGENTS.md`, `CLAUDE.md`, skills, or technical documents to `main`.
- Do not invent commands, paths, versions, benchmarks, test results, or capabilities.
- Do not mark an unavailable check `PASS`; use `BLOCKED`.
- Do not log, commit, or expose secrets, private keys, credentials, or personal data.
- Do not force-push or rewrite shared history without explicit authorization.
- Do not disable a security control to make a test or build pass.
- Do not describe planned behavior as shipped.

## 6. Skill loading and discovery

- OpenCode discovers project skills under `.agents/skills/`.
- A skill is not available merely because a file exists; verify discovery in the active agent.
- A missing mandatory skill is `BLOCKED`, not `NOT_RELEVANT`.
- Load only the product skills that the task touches; loading all eight for an unrelated task is wasteful and can distort decisions.
- When a skill is updated, rerun `scripts/verify-skills.py`, the official validator, and the affected trigger/scenario cases.

## 7. Evidence and reporting

Every substantial task records:

- The skills applied and the evidence for each.
- The exact commands run and their exit codes.
- Checks that could not run, marked `BLOCKED` with the missing precondition.
- The quality-gate decision and any open findings.
- Documentation, version, and release impact.

A checker can prove files exist. It cannot prove instructions were followed. Human or independent-agent review remains necessary.

## 8. Synchronization and drift

- Keep one canonical copy of each rule; link to it instead of duplicating it.
- If a second agent-specific representation is needed, generate it from the canonical source and add a drift test.
- Do not use unchecked symlinks on cross-platform targets.
- Record the resolution order and loaded status in `docs/development/SKILLS-AUDIT.md`.
- When an external skill changes, review its instructions and update `SKILLS-LOCK.json` and the audit.

## 9. Current files

| File | Role | Branch |
|---|---|---|
| `README.md` | Public project information | `main` and development branches |
| `AGENTS.md` | Canonical agent entry point | `integration`, `dev-*`, eligible releases |
| `CLAUDE.md` | Claude-specific entry point | `integration`, `dev-*`, eligible releases |
| `.agents/skills/` | Canonical project skill registry | `integration`, `dev-*`, eligible releases |
| `docs/development/SKILLS-AUDIT.md` | Skill audit and gate evidence | `integration` |
| `docs/development/SKILLS-LOCK.json` | Machine-readable skill manifest | `integration` |
| `docs/development/SKILL-USAGE.md` | Per-task activation matrix | `integration` |

`main` remains README-only.
