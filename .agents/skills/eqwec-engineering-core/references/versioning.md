# Versioning and Release Metadata

## Independent product versions

Each product has its own semantic version:

```text
MAJOR.MINOR.PATCH[-prerelease][+build]
```

Initial planning state: `0.0.0`. Early milestones may use `0.0.1-alpha.1`, `0.1.0-beta.1`, `0.1.0-rc.1`, then `0.1.0`.

- `MAJOR` — incompatible public contract or project-format change.
- `MINOR` — backward-compatible capability.
- `PATCH` — backward-compatible fix.
- A prerelease is not stable. Do not describe it as production-ready.

## Canonical source

Maintain one canonical version source per product. Synchronize:

- Cargo package version.
- Release manifest and update metadata.
- Changelog.
- Installer/package metadata.
- Git tag.
- Compatibility notes.

Use verified tooling or a deterministic script. Do not hand-edit five files and hope they agree.

## When to bump

Bump deliberately as part of a scoped milestone or release. Do not bump because a commit happened, because a shared crate changed, or because a build was produced. Assess dependency and compatibility impact first.

A shared crate change does not automatically require every product to bump. It requires an impact assessment: which products consume the changed API, whether their behavior or compatibility changed, and which products need a release.

## Tags

Use namespaced tags:

```text
eqwec-hub/v0.1.0
eqwec-browser/v0.1.0
photocraft/v0.1.0
vectorcraft/v0.1.0
filmcraft/v0.1.0
effectcraft/v0.1.0
craftanything/v0.1.0
audiocraft/v0.1.0
```

Prefer signed tags when the release infrastructure supports them. Never move or reuse a published tag.

## Release record

Each release records:

- Product and version.
- Release date.
- Git commit SHA.
- Supported platform and architecture.
- Release channel (`stable`, `beta`, `dev`).
- Build identifier.
- Dependency and engine versions where applicable.
- Changelog.
- Compatibility notes.
- Package hashes and signature verification result.
- Upgrade and rollback test result.

## Release gate

A release is allowed only when:

- The product's tests and essential workflow pass.
- No unresolved critical security finding exists.
- The package builds reproducibly in the release environment.
- Integrity and signature verification pass.
- Install, launch, update, and rollback are verified on each supported platform.
- The changelog is accurate.
- The version and tag are consistent.
- Authorization to publish exists.

Do not release all products together merely because they share a workspace.
