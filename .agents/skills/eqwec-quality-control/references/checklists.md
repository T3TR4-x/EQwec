# Gate Checklists

## Pre-change

- [ ] Scope and acceptance criteria are written and understood.
- [ ] Current branch and worktree state are known.
- [ ] `eqwec-engineering-core` is loaded.
- [ ] Every affected product skill is loaded.
- [ ] Relevant ADRs, specs, and prior behavior are read.
- [ ] Known defects and baseline metrics are recorded.
- [ ] Requirement drift is assessed.
- [ ] Affected shared crates, contracts, docs, skills, versions, and release metadata are identified.
- [ ] Verification plan and evidence are defined.
- [ ] A missing mandatory precondition is reported as `BLOCKED`.

## Implementation

- [ ] API and contracts are compatible or explicitly versioned.
- [ ] Edge cases and error paths are handled.
- [ ] Failure behavior is correct: fail closed, no partial commit, no replay.
- [ ] Ownership, lifetimes, `unsafe`, FFI, concurrency, and cancellation are sound.
- [ ] Locks are not held across `await`.
- [ ] Allocations, queues, parsers, and caches are bounded.
- [ ] Untrusted input is validated.
- [ ] Paths are safe against traversal and symlink escape.
- [ ] Secrets are not logged, returned, committed, or placed in fixtures.
- [ ] Permissions and sandbox boundaries are preserved.
- [ ] Dependencies are justified and license-reviewed.
- [ ] Files, processes, GPU resources, threads, and subscriptions have clear lifetimes.
- [ ] Documentation and skills are updated or a no-change decision is recorded.
- [ ] The feature is real, not a mock or placeholder.
- [ ] The diff is scoped; no unrelated refactor or cross-product contamination.

## Verification

- [ ] Formatting command run; exit code recorded.
- [ ] Lint/static analysis run; exit code recorded.
- [ ] Compilation succeeds for the supported target(s).
- [ ] Unit tests pass.
- [ ] Integration/contract tests pass.
- [ ] End-to-end workflow passes on a realistic fixture.
- [ ] Property/fuzz/adversarial tests run where applicable.
- [ ] Security checks run where applicable.
- [ ] Platform-specific checks run where applicable.
- [ ] Every check is marked `PASS`, `FAIL`, `BLOCKED`, or `NOT_APPLICABLE` with evidence.
- [ ] The original defect is reproduced and the regression test fails before the fix, then passes after it.

## Performance

- [ ] Baseline and workload are defined.
- [ ] Hardware, OS, build profile, command, and environment are recorded.
- [ ] Representative measurement is repeated enough to be stable.
- [ ] Regression threshold is defined.
- [ ] Allocations, copies, memory, startup, shutdown, UI responsiveness, and binary size are considered where relevant.
- [ ] A regression is investigated before acceptance.
- [ ] No unmeasured performance claim is published.

## Merge

- [ ] Commit scope is coherent and the message is a Ukrainian Conventional Commit.
- [ ] Branch provenance and base are verified.
- [ ] Intended diff matches the task.
- [ ] No unrelated files are staged.
- [ ] Conflict resolution preserves both sides' contracts.
- [ ] Dependency and shared-crate impact is assessed.
- [ ] Docs, skills, versions, and release metadata are updated as needed.
- [ ] Cross-product contamination is absent.
- [ ] Critical/high findings are resolved or waived.
- [ ] The resulting tree and impacted tests are verified after merge.

## Release

- [ ] Independent product version is correct.
- [ ] Changelog is accurate.
- [ ] Compatibility and dependency impact is assessed.
- [ ] Package builds reproducibly.
- [ ] Integrity and signature verification pass.
- [ ] Install, launch, update, rollback, repair, and uninstall are verified as applicable.
- [ ] Supported platform checks pass.
- [ ] No unresolved critical security finding.
- [ ] Release notes do not describe a prototype as stable.
- [ ] Authorization to publish exists.
- [ ] Released commit set, ancestry, and rollback plan are recorded.
