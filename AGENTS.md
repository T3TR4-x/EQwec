# AGENTS.md — EQwec Agent Entry Point

This is the canonical instruction file for every AI agent working in the EQwec repository. Read it before acting. `CLAUDE.md` is a Claude-specific entry point that defers to this file; if the two disagree, this file wins and the disagreement is a defect.

## 1. Project facts

- **Project:** EQwec — *Engineer Quality with Empathy to Customer*.
- **Repository:** `~/Documents/EQwec` (resolve the actual home directory; do not hardcode a username).
- **Architecture:** Rust-first modular monorepo with a Cargo workspace. Native-first, cross-platform, security-first.
- **Current phase:** Bootstrap and planning. Version `0.0.0`. **No product is implemented.**
- **Actual repository contents today:** `README.md`, `.agents/skills/`, `docs/development/`, `scripts/verify-skills.py`, and this file. There is **no** `Cargo.toml`, `apps/`, `crates/`, CI pipeline, installer, or release artifact yet.
- **Environment used for current evidence:** macOS on Apple silicon. Windows and Linux are planned targets, not verified platforms.

Never describe a planned capability as implemented. Never invent a command, path, version, test result, benchmark, or release.

## 2. Branch model

| Branch | Purpose | Contents |
|---|---|---|
| `main` | Informational branch | `README.md` only |
| `integration` | Shared workspace, policies, skills, architecture | Current development baseline |
| `dev-<product>` | Long-lived product workstreams | Product-specific work |
| `feature/*`, `fix/*`, `debug/*`, `perf/*`, `security/*`, `docs/*`, `release/*`, `hotfix/*` | Scoped short-lived work | One coherent change |
| `production` | Reviewed release source and metadata | Verified releases only |

Rules:

- Do not add `AGENTS.md`, `CLAUDE.md`, skills, or technical documents to `main`.
- Keep `main` synchronized only through a deliberate README-only update.
- Do not merge whole development trees into `main` or `production`.
- Inspect the worktree and index before staging, switching, merging, rebasing, or cherry-picking.
- Do not run `reset --hard`, destructive `clean`, blanket checkout overwrite, or delete lock files.
- Never force-push or rewrite shared history without explicit authorization.
- Amend only the immediately preceding local, unpushed commit that belongs to the same logical task; never amend `production`, `integration`, or shared `dev-*` history.

## 3. Commit policy

Use Conventional Commits with a **Ukrainian imperative description**:

```text
feat(hub): додати перевірку підпису пакетів
fix(browser): виправити відновлення вкладок
docs(core): оновити правила версіонування
```

- Keep `type` and `scope` in standard Latin spelling.
- Write the subject and body in Ukrainian.
- Use `BREAKING CHANGE:` only when the Conventional Commits standard requires that exact token.
- Keep commits atomic; never stage unrelated files blindly.
- Never commit secrets, private keys, build outputs, or generated binaries.

## 4. Skills gate

The repository has exactly **10 project-owned skills** in `.agents/skills/`. They are canonical, version-controlled, and mandatory.

| Skill | Load when |
|---|---|
| `eqwec-engineering-core` | Every EQwec engineering task |
| `eqwec-quality-control` | Before and after any change; before merge or release |
| `eqwec-hub` | Hub launcher, install, update, rollback, repair, uninstall, manifests, packaging, signing |
| `eqwec-browser` | Browser app, Chromium integration, FFI/IPC, profiles, tabs, sessions, browser security/performance |
| `eqwec-photocraft` | Photo document model, layers, masks, RAW, color, tiled rendering, photo project/export |
| `eqwec-vectorcraft` | Vector documents, paths, Bézier geometry, booleans, SVG, hit-testing, vector export |
| `eqwec-filmcraft` | NLE timeline, media, A/V sync, proxy/cache, render queue, video export |
| `eqwec-effectcraft` | Compositing graph, nodes, keyframes, GPU effects, cache invalidation, FilmCraft interchange |
| `eqwec-craftanything` | Workspace manifests, asset exchange, canonical ownership, provenance, cross-product IPC |
| `eqwec-audiocraft` | DAW, real-time audio, MIDI, plugin hosting, latency/xruns, audio render |

Precedence: `eqwec-engineering-core` governs global policy. A product skill may add domain constraints but must never weaken core security, versioning, Git, or documentation rules. `eqwec-quality-control` governs gates and evidence. If a product skill conflicts with core, core wins and the conflict is a skill defect.

Loading all eight product skills for every task is unnecessary. Load the skill for each affected product. A missing mandatory skill is `BLOCKED`, not `NOT_RELEVANT`.

Before relying on the registry, run:

```bash
python3 scripts/verify-skills.py
```

To include the official schema validator, create a local validation environment once:

```bash
python3 -m venv .venv-skillcheck
.venv-skillcheck/bin/pip install -r scripts/skill-validation-requirements.txt
.venv-skillcheck/bin/python scripts/verify-skills.py \
  --validator ~/.agents/skills/skill-creator/scripts/quick_validate.py
```

The official validator requires PyYAML. Without it, the script fails with a clear remediation message instead of silently skipping validation. This script proves package and description facts. It does **not** prove model activation. Record agent-level trigger evidence in `docs/development/SKILLS-AUDIT.md`.

## 5. Mandatory workflow

1. **Understand** — restate the requirement and acceptance criteria; identify unknowns and blockers.
2. **Preflight** — read this file, the relevant ADRs, product spec, and skill; inspect branch, worktree, and recent commits.
3. **Plan the smallest correct change** — prefer deletion, reuse, and standard-library solutions before new abstractions.
4. **Implement** — keep the diff scoped; add the regression test with the fix.
5. **Verify** — run the repository's real commands and record exit codes. Mark unavailable checks `BLOCKED`, never `PASS`.
6. **Review** — apply `eqwec-quality-control`; resolve critical and high findings before merge.
7. **Document** — update only impacted documents and skills, or record why no change is needed.
8. **Commit** — stage the intended files; write a scoped Ukrainian Conventional Commit.
9. **Integrate** — merge into the correct `dev-*` branch, then `integration`, with contract and impact checks.
10. **Release** — only after the release gate, packaging, signing, and explicit authorization.

## 6. Engineering rules

- Correctness and user-data integrity come before feature volume.
- Security and privacy are design inputs. Untrusted input is parsed with bounded resources and validated before use.
- Secrets, tokens, passwords, private keys, and prompts are never logged, returned, committed, or placed in fixtures.
- A truncated stream, partial write, or unverified package is a failure, never a success.
- Authentication, signature, and credential-store failures fail closed.
- No replay or duplicate side effect after a response is committed.
- Every non-trivial behavior has a regression test before the fix.
- A performance claim requires a reproducible baseline, workload, environment, command, and result.
- Prefer `#![forbid(unsafe_code)]`; isolate and document every `unsafe` boundary.
- Use typed domain models and typed errors; avoid `unwrap()`/`expect()` on recoverable production paths.
- Bound allocations, queues, parser depth, and caches; define eviction and backpressure.
- Do not hold a lock across an `await`; do not put an async runtime into a latency-sensitive audio or render callback.
- Keep dependencies minimal and justified; record license and maintenance status.
- Do not disable a platform security control to reduce memory, improve startup, or simplify packaging.

## 7. Build, test, and quality commands

There is no Cargo workspace yet. Therefore:

- `cargo build`, `cargo test`, `cargo clippy`, and `cargo fmt --check` are **BLOCKED** until the workspace exists.
- There is no CI pipeline, installer, or benchmark harness yet.
- The only executable project check today is `python3 scripts/verify-skills.py`.

When the workspace is created, this section must be updated with the actual commands, and every command must be validated before being documented. Do not copy generic Cargo command lists and present them as verified project commands.

## 8. Documentation lifecycle

Priority order:

1. `README.md` — mission, products, architecture summary, real status, platforms, links.
2. `AGENTS.md` — this file; canonical agent rules.
3. `CLAUDE.md` — Claude-specific entry point; must not contradict this file.

Also maintain, as their subject matter becomes real:

- `docs/architecture/SYSTEM-ARCHITECTURE.md`, `TECHNOLOGY-SELECTION.md`, `DEPENDENCY-MAP.md`
- `docs/adr/`
- `docs/research/FEASIBILITY.md`, `COMPETITOR-ANALYSIS.md`, `LEGAL-LICENSE-REVIEW.md`
- `docs/security/THREAT-MODEL.md`, `SECURITY-BASELINE.md`
- `docs/performance/PERFORMANCE-STRATEGY.md`
- `docs/development/ROADMAP.md`, `BRANCHING-STRATEGY.md`, `VERSIONING.md`, `TEST-STRATEGY.md`, `RELEASE-STRATEGY.md`, `QUALITY-GATES.md`, `SKILLS-AUDIT.md`, `SKILLS-LOCK.json`, `SKILL-USAGE.md`
- `docs/product-specs/<product>.md`

Rules:

- Update impacted documents only; do not rewrite every Markdown file on every commit.
- Do not create fake links, commands, versions, or capabilities.
- Keep historical ADRs and changelogs; mark superseded instructions instead of deleting history.
- Relative links must resolve.
- `main` remains README-only.

## 9. Security baseline

- Maintain a threat model for local data, downloaded packages, updates, browser sessions, untrusted documents, plugins, IPC, native libraries, and release infrastructure.
- Validate paths against traversal, absolute-path injection, symlink escapes, and platform separators.
- Verify signatures and integrity before executing or installing a downloaded artifact.
- Use safe deserialization and bounded parsing; reject inconsistent or unknown critical fields.
- Apply least privilege and preserve platform sandboxing.
- Keep signing keys out of the repository; document custody, rotation, and revocation.
- Default to local storage and explicit user control; no telemetry without documented consent.

## 10. Definition of done

A task is complete only when:

1. The required behavior is implemented and its acceptance workflow passes.
2. The code builds in the supported environment.
3. Relevant automated tests pass.
4. The change is reviewed; no blocking regression is introduced.
5. Security and performance implications are assessed.
6. User-facing behavior is verified where applicable.
7. Impacted docs, skills, versions, and release metadata are reconciled or a no-change decision is recorded.
8. The change is committed to the correct branch with a scoped Ukrainian Conventional Commit.
9. `SKILL_USAGE` and the quality-gate report distinguish `APPLIED`, `AVAILABLE_NOT_RELEVANT`, `BLOCKED`, `PASS`, `FAIL`, and `NOT_APPLICABLE` truthfully.

## 11. Current gate status

- **SKILLS_GATE:** open. All 10 packages must exist, validate, be discoverable, and pass trigger/scenario tests before closure.
- **DOCUMENTATION_GATE:** open. Research, architecture, security, performance, and development plans are not yet written.
- **ARCHITECTURE_GATE:** open. No ADRs or approved system architecture yet.
- **SECURITY_GATE:** open. No threat model or security baseline yet.
- **PRODUCT_IMPLEMENTATION:** prohibited until the gates above pass.

Report blockers as `BLOCKED` with the exact missing precondition. Do not fabricate progress.
