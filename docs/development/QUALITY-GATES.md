# Quality Gates

**Status:** approved gate model. It applies to every change, merge, and release. The authoritative procedural skill is `.agents/skills/eqwec-quality-control/SKILL.md`.

## 1. Gate outcomes

Every gate returns exactly one outcome:

| Outcome | Meaning |
|---|---|
| `PASS` | Required evidence exists and the criteria are met. |
| `FAIL` | Evidence exists and shows a defect, regression, or unmet criterion. |
| `BLOCKED` | The check cannot run, or a mandatory prerequisite is missing. |
| `NOT_APPLICABLE` | The check does not apply, with a written rationale. |

Never convert `BLOCKED` into `PASS`. Never treat a skipped test, a successful compile, or a code review comment as functional verification.

## 2. Gate sequence

### Gate 1 — Pre-change

Required:

- Scope, acceptance criteria, branch, and worktree state.
- `eqwec-engineering-core` and every affected product skill loaded.
- Relevant ADRs, product spec, known defects, and baseline metrics.
- Requirement-drift check against approved architecture and scope.
- Affected products, shared crates, contracts, docs, skills, versions, and release metadata identified.
- Verification plan and evidence defined.

Failure to identify the baseline for a performance claim makes the claim `BLOCKED`.

### Gate 2 — Implementation

Inspect the actual code, not the summary:

- Public API and contracts: compatible or explicitly versioned.
- Edge cases, empty inputs, limits, error paths.
- Failure behavior: fail closed, no partial writes, no replay.
- Rust safety: ownership, lifetimes, `unsafe`, FFI, concurrency, cancellation, lock-across-await, bounded allocation.
- Security: untrusted input, path safety, secrets, permissions, IPC validation, dependency changes.
- Resource lifetimes: files, processes, GPU resources, threads, subscriptions, temp files.
- Documentation and skills updated or explicitly assessed.
- Real functionality, not a mock, placeholder, or hard-coded demo.
- Scope limited to the task; no cross-product contamination.

### Gate 3 — Verification

Run the repository's real commands. Discover them from `AGENTS.md`, CI, or build files; do not invent them. Record the exact command and exit code.

Expected categories once the workspace exists:

- Formatting.
- Static analysis and lints.
- Compilation for supported targets.
- Unit tests.
- Integration and contract tests.
- End-to-end workflow on a real fixture.
- Property, fuzz, and adversarial input tests where applicable.
- Security checks: advisories, licenses, secret scanning, signature tests.
- Platform-specific checks.

Then exercise a real functional workflow, not only isolated units.

### Gate 4 — Performance

- Compare representative measurements against a relevant baseline.
- Record hardware, OS, build profile, workload, command, target, observed result, and regression threshold.
- Investigate regressions before accepting them.
- Check allocations, copies, UI responsiveness, startup, shutdown, memory, and binary/install size where relevant.
- Do not accept a synthetic microbenchmark as proof of a real-workload improvement.
- If the measurement cannot be reproduced, mark the claim `BLOCKED` and remove it from release notes.

### Gate 5 — Merge

- Commit scope and Ukrainian Conventional Commit messages.
- Branch provenance and current base.
- Intended diff; unrelated files not staged.
- Conflict resolution preserved both sides' contracts.
- Dependency and shared-crate impact assessed.
- Documentation, skills, versions, and release metadata updated as needed.
- No cross-product contamination.
- Critical unresolved defect or missing essential gate blocks the merge.

### Gate 6 — Release

- Independent product version and changelog correct.
- Compatibility and dependency impact assessed.
- Installers/packages build reproducibly.
- Integrity and signature verification pass.
- Install, launch, update, rollback, repair, and uninstall verified as applicable.
- Supported platform checks pass.
- No unresolved critical security finding.
- Release notes accurate; no prototype described as stable.
- Authorization to publish exists.
- Released commit set and rollback plan recorded.

## 3. Severity and waivers

| Severity | Merge effect | Release effect |
|---|---|---|
| `critical` | Blocks | Blocks |
| `high` | Blocks unless fixed or explicitly waived | Blocks unless authorized waiver |
| `medium` | Fix or record with owner and target | Assess; may block if user-facing or security-relevant |
| `low` | Fix or record | May ship with a note |

A waiver requires explicit authorization, reason, exact scope and versions, expiration or review trigger, residual risk, and compensating control. A waiver is not a `PASS`. Critical security controls must not be waived merely to accelerate delivery.

## 4. Evidence format

```text
Gate: verification
Command: <exact command>
Exit code: <code>
Outcome: PASS | FAIL | BLOCKED | NOT_APPLICABLE
Evidence: <output summary, artifact, log path, or reason>
Environment: <OS, architecture, toolchain, build profile>
```

For a performance result, also record hardware, workload, baseline, target, observed value, and regression threshold.

## 5. Current baseline

| Check | State | Reason |
|---|---|---|
| Skill registry preflight | PASS | `python3 scripts/verify-skills.py` — 10 skills, 0 failures |
| Official skill schema validation | PASS | `quick_validate.py` — 10/10 valid |
| `cargo fmt --check` | PASS | `cargo fmt --all -- --check`, exit 0, 2026-11-07 |
| `cargo clippy` | PASS | `cargo clippy --workspace --all-targets --all-features -- -D warnings`, exit 0 |
| `cargo build` | PASS | `cargo build --workspace --all-targets --locked`, exit 0 |
| `cargo test` | PASS | `cargo test --workspace --all-features --locked`, 211 tests passed, 0 failed |
| Hub manifest/trust/package verification | PASS | 19 integration tests in `crates/eqwec-updater/tests/verification.rs`, including a fixed OpenSSL-generated Ed25519 vector |
| Hub registry/journal/layout/active-pointer primitives | PASS | `registry.rs`, `journal.rs`, `layout.rs`, `active.rs`, `atomic.rs`; unit tests cover round-trip, bounds, traversal, partial lines, and atomic replacement |
| Hub activation transaction and recovery | PASS for the in-process engine | `activation.rs`; 25 tests cover install, update, rollback, five fault points, recovery before and after the pointer, rollback recovery, and single-in-flight enforcement |
| Hub package archive validation and staging | PASS for in-process gates 1–11 | `staging.rs`; 31 tests cover ZIP/ZIP64, traversal, links, modes, duplicate names, every bound, header confusion, encryption, multi-disk, cleanup, and staging-to-activation |
| Hub fixture install/update/rollback workflow | PASS for in-process and local macOS evidence | `tests/fixtures/eqwec-hub/`; 3 end-to-end tests plus 4 demo-bundle tests install and launch v1, update and launch v2, roll back, reject a tampered package, and recover an interrupted activation |
| Hub launcher workflow | PASS for local macOS developer preview | `apps/eqwec-hub`; 17 unit and 13 integration tests cover install, update, rollback, uninstall, launch, health check, pointer-first resolution, and disk preflight; `--self-check` exits 0; the window stayed alive for 5 seconds |
| Hub installer and launcher | BLOCKED for release; local developer preview PASS | Launcher, uninstaller, and packaging scripts exist and pass local/in-process tests; download transport, repair, offline bundles, diagnostics, release signing, and Windows/Linux desktop acceptance do not |
| `cargo audit` | PASS | `cargo audit`, exit 0; 0 advisories across 458 locked dependencies with 4 documented RUSTSEC ignores |
| `cargo deny check` | PASS | `cargo deny check`, exit 0; advisories, bans, licenses, sources ok |
| CI pipeline | PASS | GitHub Actions run `42398364288` on commit `67c7444`: all five jobs succeeded (`test` on macOS, Ubuntu, Windows; `advisories`; `licenses and bans`) |
| Hub install/update/rollback platform acceptance | BLOCKED for Windows/Linux desktop; headless and package smoke checks PASS | In-process and launcher tests run on all three CI platforms; package install/self-check/uninstall passed on macOS, Windows, and Linux. The launcher window has only local macOS evidence |
| Packaging/signing checks | BLOCKED for release signing; unsigned artifact checks PASS | Run `42398364283`: macOS DMG mounted and self-checked; Windows per-user MSI installed, self-checked, and uninstalled; Linux `.deb` installed, self-checked, and removed. Developer ID, notarization, and Authenticode do not exist |
| Benchmarks | BLOCKED | No benchmark harness or baseline |

A `BLOCKED` check is not a failure of the project at the planning stage, but it must remain visible and must be executed before the relevant release gate.

## 6. Independence

The author should not be the only reviewer of a high-risk change. When a separate agent or human reviewer is available, use it. If self-review is unavoidable, state that independence was limited and apply the same evidence standard.

## 7. CI execution

The workspace exists and `.github/workflows/ci.yml` runs on GitHub Actions. Run `42398364288` on commit `67c7444` passed all five jobs: format, clippy, build, and tests on macOS, Ubuntu, and Windows, plus advisory and license/ban checks. `.github/workflows/package.yml` builds and smoke-tests the macOS DMG, Windows MSI, and Linux packages; all three passed in run `42398364283`. The pipeline executes, in order:

```text
format -> lint -> license/advisory -> build -> unit -> integration
-> security -> platform tests -> packaging validation -> release approval
-> signing/publishing
```

The pipeline must use reproducible dependency resolution, pinned toolchains, and cached artifacts that cannot bypass verification. CI configuration is not a substitute for local evidence.

## 8. Gate closure rule

A product milestone closes only when its acceptance workflow passes. A release closes only when the actual artifact is built, verified, and available through an authorized channel. Documentation that says "done" is not evidence.
