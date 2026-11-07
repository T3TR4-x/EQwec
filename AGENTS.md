# AGENTS.md — EQwec Agent Entry Point

This is the canonical instruction file for every AI agent working in the EQwec repository. Read it before acting. `CLAUDE.md` is a Claude-specific entry point that defers to this file; if the two disagree, this file wins and the disagreement is a defect.

## 1. Project facts

- **Project:** EQwec — *Engineer Quality with Empathy to Customer*.
- **Repository:** `~/Documents/EQwec` (resolve the actual home directory; do not hardcode a username).
- **Architecture:** Rust-first modular monorepo with a Cargo workspace. Native-first, cross-platform, security-first.
- **Current phase:** Phase 3 — Hub MVP in progress on `dev-hub`. Workspace version `0.0.0`; the Hub launcher is a developer preview at `0.1.0-alpha.1`. A runnable `egui`/`eframe` launcher exists in `apps/eqwec-hub` and supports local signed-bundle install, update, rollback, uninstall, launch, and `--self-check`. Packaging scripts build a macOS DMG, a per-user Windows MSI, and Linux `.deb`/`tar.zst` artifacts. There is no production release, download transport, repair, dependency resolution, or release signing.
- **Actual repository contents today:** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `deny.toml`, `.github/workflows/ci.yml`, `.github/workflows/package.yml`, `apps/eqwec-hub/`, `crates/eqwec-core/`, `crates/eqwec-security/`, `crates/eqwec-updater/`, `packaging/eqwec-hub/`, `tests/fixtures/eqwec-hub/`, `docs/`, `.agents/skills/`, `scripts/`, `README.md`, and this file. The fixture under `tests/` is non-production test infrastructure, not a shipped product.
- **Environment used for current evidence:** macOS on Apple silicon for local commands, including a live launcher window and a locally built DMG. GitHub Actions runs the workspace tests and packaging jobs on macOS, Ubuntu, and Windows; no desktop launcher run has been accepted on Windows or Linux.

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

The Cargo workspace exists. Verified on macOS 26.7.2 arm64 with Rust 1.99.1 on 2026-11-07:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace --all-targets --locked
cargo test --workspace --all-features --locked
cargo audit
cargo deny check
python3 scripts/verify-skills.py
python3 scripts/check-docs.py --gate
```

Hub developer-preview commands:

```bash
cargo run -p eqwec-hub
cargo run -p eqwec-hub -- --self-check
cargo run -p eqwec-hub-fixture --bin make-demo-bundle -- /tmp/eqwec-demo/bundle
```

Packaging commands (artifacts land in `dist/`):

```bash
./packaging/eqwec-hub/macos/build-dmg.sh
./packaging/eqwec-hub/windows/build-msi.ps1   # PowerShell on Windows
./packaging/eqwec-hub/linux/build-deb.sh
```

- `cargo audit` and `cargo deny check` require the tools installed locally; CI installs them.
- CI is configured in `.github/workflows/ci.yml` and runs on `integration`, `dev-*`, and scoped work branches. The workspace has 211 tests. GitHub Actions run `42398364288` on commit `67c7444` passed all five jobs: `test (macos-latest)`, `test (ubuntu-latest)`, `test (windows-latest)`, `advisories`, and `licenses and bans`.
- `.github/workflows/package.yml` builds and smoke-tests platform artifacts. macOS DMG, Windows MSI, and Linux packages all passed in run `42398364283`, including install, `--self-check`, and uninstall where the runner permits.
- The launcher, installer, and packaging scripts exist, but there is no benchmark harness and no accepted Windows/Linux desktop run.
- Windows and Linux are exercised by CI tests and package smoke checks; no local or product-runtime desktop acceptance exists for them.

Do not copy generic command lists and present them as verified project commands.

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

- **SKILLS_GATE:** PASS for the bootstrap scope. All 10 packages exist, validate, and are discoverable; trigger and scenario evidence is recorded in `docs/development/SKILLS-AUDIT.md`. Runtime product end-to-end scenarios on Windows and Linux are `BLOCKED` until desktop runs exist.
- **DOCUMENTATION_GATE:** PASS. `python3 scripts/check-docs.py --gate` returned 0 failures across 63 Markdown files on 2026-11-07. Research, architecture, security, performance, and development documents exist.
- **ARCHITECTURE_GATE:** PASS for the planning baseline. ADR-0001 through ADR-0013 are recorded; ADR-0004 and ADR-0008 remain `PROPOSED`. Runtime verification beyond the Hub developer preview is `BLOCKED` until other prototypes exist.
- **SECURITY_GATE:** PASS for the planning baseline and local Hub bundle controls. `docs/security/THREAT-MODEL.md` and `SECURITY-BASELINE.md` exist; H-12–H-15 controls are implemented and tested. Other product runtime controls, platform trust, and fuzzing remain `BLOCKED`.
- **WORKSPACE_GATE:** PASS locally on macOS for format, clippy, build, test, audit, and deny, and PASS in GitHub Actions run `42398364288` on commit `67c7444` for all five jobs, including `test (windows-latest)` and `test (ubuntu-latest)`.
- **PRODUCT_IMPLEMENTATION:** Hub verification core, durable-state primitives, activation engine, bounded package staging, journaled uninstall, profile locking, disk preflight, a native `egui`/`eframe` launcher, and a non-production fixture product are implemented on `dev-hub` (`eqwec-security`, `eqwec-updater`, `apps/eqwec-hub`, `tests/fixtures/eqwec-hub`): bounded manifest v1, trust-store policy, strict Ed25519, package size/hash verification, registry, journal, installation layout, atomic active pointer, install/update/rollback transactions, fault-injected startup recovery, ZIP/ZIP64 package validation and extraction into a durable staging tree, uninstall recovery, local signed-bundle install/update/rollback/uninstall/launch, health checks, `--self-check`, and packaging scripts for macOS, Windows, and Linux. The workspace has 211 tests passing locally and in CI on macOS, Ubuntu, and Windows. The macOS launcher window and DMG were exercised locally. Hub download transport, repair orchestration, offline bundles, dependency resolution, release signing, notarization, Authenticode, and Windows/Linux desktop acceptance remain `BLOCKED`. Planning documents and verification crates are not a released product.
- **PLATFORM VERIFICATION:** Workspace tests `PASS` on GitHub-hosted macOS, Ubuntu, and Windows runners. macOS DMG, Windows MSI, and Linux package smoke checks `PASS` in run `42398364283`. Desktop launcher acceptance is `BLOCKED` on Windows and Linux because no desktop session has been exercised there.
- **RELEASE_GATE:** `BLOCKED`. No signing keys, certificates, notarization account, release channel, or signed release artifact exists.

Report blockers as `BLOCKED` with the exact missing precondition. Do not fabricate progress.
