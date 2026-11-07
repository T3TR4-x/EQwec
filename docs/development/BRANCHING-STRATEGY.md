# Branching Strategy

**Status:** approved policy for the bootstrap and planning phases. Product branches are created in Phase 2 from the approved `integration` baseline.
**Authority:** `AGENTS.md` and `.agents/skills/eqwec-engineering-core/SKILL.md`. If this document conflicts with them, they win.

## 1. Branch roles

| Branch | Lifetime | Purpose | Allowed contents |
|---|---|---|---|
| `main` | Permanent | Public project information | `README.md` only |
| `integration` | Permanent | Shared workspace, shared crates, policies, skills, cross-product contracts | Current reviewed development baseline |
| `dev-eqwec-hub` | Permanent | Hub workstream | Hub app, Hub tests, Hub docs |
| `dev-eqwec-browser` | Permanent | Browser workstream | Browser app, engine integration, browser tests |
| `dev-photocraft` | Permanent | PhotoCraft workstream | PhotoCraft app and crates |
| `dev-vectorcraft` | Permanent | VectorCraft workstream | VectorCraft app and crates |
| `dev-filmcraft` | Permanent | FilmCraft workstream | FilmCraft app and crates |
| `dev-effectcraft` | Permanent | EffectCraft workstream | EffectCraft app and crates |
| `dev-craftanything` | Permanent | CraftAnything workstream | CraftAnything app and crates |
| `dev-audiocraft` | Permanent | AudioCraft workstream | AudioCraft app and crates |
| `production` | Permanent | Released source and release metadata | Verified releases only |
| `feature/<product>/<name>` | Short-lived | One feature | Scoped change plus tests and docs |
| `fix/<product>/<issue>` | Short-lived | One defect fix | Regression test plus fix |
| `debug/<product>/<topic>` | Temporary | Investigation only | Diagnostics; must not become a dumping ground |
| `perf/<product>/<topic>` | Short-lived | Measured optimization | Baseline, change, result |
| `security/<product>/<issue>` | Short-lived | Security fix | Fix, regression test, advisory notes |
| `docs/<topic>` | Short-lived | Documentation change | Docs only |
| `test/<product>/<scope>` | Short-lived | Test infrastructure | Tests and fixtures |
| `refactor/<product>/<component>` | Short-lived | Behavior-preserving change | Refactor plus proof of equivalence |
| `release/<product>/vX.Y.Z` | Per release | Scoped release assembly | Only reviewed product changes and required dependencies |
| `hotfix/<product>/<issue>` | Per hotfix | Critical production fix | Minimal verified fix plus release metadata |

## 2. Creation and protection

- `main` is created first and contains only the README. It is never a development base.
- `integration` is created from `main` and holds the workspace baseline.
- `dev-<product>` branches are created from the current approved `integration` baseline in Phase 2, once the workspace exists. Creating empty long-lived branches earlier adds no value and risks divergence.
- `production` is created from the first approved release commit, not from an arbitrary development tip.
- Protect `main`, `integration`, and `production` with review and CI checks once a remote and CI exist. Until then, local discipline and documented evidence apply.
- Never commit directly to `main` or `production`. A README-only `main` update is a deliberate, reviewed commit.

## 3. Workflow

### Normal change

1. Start from the correct `dev-<product>` branch, not from `main`.
2. Create a scoped branch: `feature/<product>/<name>`, `fix/<product>/<issue>`, etc.
3. Implement the smallest correct change, with tests and impacted docs.
4. Run the verification commands that actually exist; record exit codes.
5. Review the diff and apply `eqwec-quality-control`.
6. Commit with a Ukrainian Conventional Commit.
7. Merge into `dev-<product>` after review.
8. Integrate shared changes into `integration` with a contract/impact assessment.
9. Periodically integrate reviewed `integration` changes into `dev-*` branches.

### Shared-crate change

1. Identify every consumer before editing.
2. Change the API with compatibility in mind; version breaking changes explicitly.
3. Add contract tests that exercise every consumer.
4. Merge into `integration` only after the cross-product impact is documented and tested.
5. Reconcile the change into affected `dev-*` branches; do not silently overwrite uncommitted work.

### Release

1. Create `release/<product>/vX.Y.Z` from current `production`.
2. Promote only reviewed product changes and their required, tested dependencies from development. Use a reviewed cherry-pick sequence or scoped commits, not a full-branch merge.
3. Verify that unrelated unfinished work did not enter.
4. Run the release gate, packaging, integrity, signing, install/update/rollback checks.
5. Merge into `production` with protected review.
6. Tag the release with a namespaced tag.
7. Reconcile `production` back into `integration` and the affected `dev-*` branch.

### Hotfix

1. Branch `hotfix/<product>/<issue>` from the released `production` commit.
2. Apply the minimal fix and a regression test.
3. Verify the affected release workflow and compatibility.
4. Merge into `production`, tag a patch release, then reconcile back into `integration` and `dev-*`.

## 4. Merge rules

- Verify the base and upstream before merging. Do not merge a stale branch blindly.
- Inspect the intended diff and the staged file list. Reject unrelated files and cross-product contamination.
- Resolve conflicts by understanding both sides. Never use blanket `-X ours` or `-X theirs`.
- Record nontrivial conflict decisions, especially shared-contract changes.
- After merging, verify the resulting tree, impacted tests, branch state, and absence of accidental files.
- Do not rebase published shared branches or force-push protected refs.
- Do not delete a branch with unmerged work without explicit authorization.

## 5. Commit rules

- Conventional Commits with a Ukrainian imperative description.
- One coherent logical change per commit.
- No secrets, private keys, build outputs, or generated binaries.
- Amend only the immediately preceding local, unpushed commit that belongs to the same logical task.
- Never amend `production`, `integration`, or shared `dev-*` history.

## 6. Divergence control

- `integration` is the single shared baseline. A product branch that drifts for too long must be reconciled deliberately.
- Shared-crate changes require an impact assessment naming every consumer.
- Product branches must not depend on another product's internal code. Use a shared crate with an owner or a versioned interchange contract.
- A release branch is assembled from `production`, not from `integration`, to prevent unrelated work from leaking into a release.

## 7. Current repository state

| Branch | Exists | State |
|---|---|---|
| `main` | yes | README-only commit `8a2f462` |
| `integration` | yes | Skills gate commit `608677d` |
| `production` | no | Created at first approved release |
| `dev-*` | no | Created in Phase 2 from the approved `integration` baseline |

No remote is configured. Nothing has been pushed. Branch protection and CI checks are `BLOCKED` until a remote and CI exist.
