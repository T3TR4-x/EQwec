# ADR-0013: Hub uninstall transaction

- **Status:** ACCEPTED
- **Date:** 2026-11-07
- **Deciders:** EQwec project owner and engineering agent (bootstrap mandate)
- **Supersedes:** none
- **Related:** ADR-0011, ADR-0012, `docs/product-specs/eqwec-hub-schemas.md`, `docs/security/THREAT-MODEL.md`

## 1. Context

ADR-0011 defines the Hub journal, registry, and installation layout. Its
journal phases cover install, update, and rollback. The schema requires that a
registry record is removed only after uninstall completes and the journal
records it, but it does not define uninstall phases or recovery. Without a
durable intent, a crash between file removal and the registry write leaves
either a launchable record pointing at a missing tree or an untracked orphan
directory.

## 2. Decision

Uninstall is a journaled transaction with two complete outcomes: the product is
installed, or its tree and registry record are gone.

Two phases are added to journal v1:

| Phase | Meaning | Next mutation |
|---|---|---|
| `uninstall-intent` | The record and containment were validated; removal is about to start | Remove the product root |
| `uninstalled` | The tree and registry record are removed | Terminal; prune |

`uninstalled` is terminal alongside `committed`, `failed`, and `rolled-back`.

Order:

1. Validate that no other operation is unfinished.
2. Load the registry record. If it is absent, return success only when the same
   idempotency key already has an `uninstalled` record; otherwise return
   `ProductNotInstalled`.
3. Require `record.install_root == layout.product_root(product_id)` and strict
   containment under the profile base.
4. Append `uninstall-intent` with the current version, channel, and package
   hash.
5. Remove the product root recursively without following symbolic links.
6. Remove the registry record and write the registry atomically.
7. Append `uninstalled`.

## 3. Recovery

`uninstall-intent` maps to `ResumeUninstall`:

1. Reject a registry record whose install root does not match the layout.
2. Remove the product root; a missing root is success.
3. Remove the record when present and write the registry atomically.
4. Append `uninstalled`.

Recovery is idempotent. Running it twice produces the same registry and no
duplicate records. A crash after step 5 and before step 6 is resolved by
repeating steps 2–4.

A non-injected failure during removal or the registry write appends `failed`
with `UNINSTALL_REMOVE_FAILED` or `UNINSTALL_REGISTRY_FAILED`. The operation
then needs a new uninstall attempt; the journal never guesses that a failed
removal completed.

## 4. Security

- The root must be an absolute, normalized, strict descendant of the profile
  base.
- A product root that is itself a symbolic link is rejected.
- `std::fs::remove_dir_all` removes inner symbolic links without following
  them; a test proves an outside target survives.
- The registry is the source of truth and is updated only after file removal.
- The launcher holds the exclusive profile lock, so no reader observes the
  intermediate state; recovery runs before the window becomes interactive.

## 5. Consequences

- Journal v1 gains two phase strings. Existing journals parse unchanged; a
  journal containing the new phases is readable by this implementation only.
  No released version exists, so no migration is required.
- The schema document, `recovery_action`, and `recover` must stay in sync with
  this ADR.
- Uninstall removes Hub-owned files and the registry record. User documents are
  never inside the product root and are not touched.
