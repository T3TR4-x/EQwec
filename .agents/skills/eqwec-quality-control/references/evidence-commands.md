# Evidence Commands

Discover the actual commands from the repository before using this list. These are expected baseline commands for a Cargo workspace; they are not valid until `Cargo.toml` exists and the commands have been run successfully.

## Repository state

```sh
git status --short --branch
git log --oneline --decorate -10
git diff --stat
git diff --check
```

## Rust workspace

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace --all-targets
cargo test --workspace --all-targets
```

Use `--locked` where the repository requires a locked dependency graph. Use a target-specific command when the change is platform-sensitive.

## Security and supply chain

```sh
cargo audit
cargo deny check
```

Run these only when the tools and advisory database are available locally or in CI. If the network is unavailable, mark the check `BLOCKED`, not `PASS`.

## Performance

Record the exact command and environment, for example:

```sh
cargo bench --bench <name> -- --save-baseline <label>
```

Do not publish a result without the hardware, OS, build profile, workload, and measurement method.

## Packaging and release

Use the repository's actual packaging commands once they exist. Verify:

- Package hash.
- Signature verification.
- Install into a clean staging directory.
- Launch from the installed location.
- Update from the previous version.
- Interrupt the update and recover.
- Roll back.
- Uninstall and verify no orphaned files or registry entries.

## Evidence discipline

- Record the exact command, exit code, and a bounded output excerpt.
- Redact secrets, tokens, personal paths, and raw payloads.
- Distinguish `PASS`, `FAIL`, `BLOCKED`, and `NOT_APPLICABLE`.
- Do not claim a check ran if it did not.
