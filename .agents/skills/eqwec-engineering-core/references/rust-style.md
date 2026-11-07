# Rust Style and Safety Rules

## Toolchain and lints

- Pin the toolchain in `rust-toolchain.toml`. Use stable Rust unless a documented platform constraint requires otherwise.
- Run `cargo fmt --all --check` and the repository's Clippy command. A warning that the CI treats as an error is a failure.
- Prefer `#![forbid(unsafe_code)]` at crate level. If a crate needs FFI, use `#![deny(unsafe_op_in_unsafe_fn)]` and isolate the unsafe modules.
- Do not add `#[allow(...)]` at crate or module level without a comment naming the reason, the scope, and the condition under which it can be removed.

## Module and API design

- A module has one reason to change. Keep parsing, domain rules, persistence, and presentation separate.
- Export the smallest useful API. Prefer private fields and constructors that enforce invariants.
- Use newtypes for identifiers, paths, versions, and units. Do not pass a bare `String` where `ProductId`, `TenantId`, `Version`, or `ByteSize` is meant.
- Model invalid states out of existence where practical. A validated value should not need re-validation at every call site.
- Keep public traits small. A trait with one implementation and no test seam is usually an unnecessary abstraction.
- Do not expose a dependency's type in a public API unless the dependency is part of the contract.

## Errors

- Use typed errors per boundary. A library returns a structured error; a binary decides how to present it.
- Preserve the source chain with `#[source]` or `Error::source`. Do not stringify an error and discard the cause.
- Do not use `anyhow` in a library's public API. It is acceptable in an application's top-level orchestration when the error is not part of a contract.
- Avoid `unwrap()` and `expect()` in recoverable paths. If an invariant makes a panic correct, state the invariant in the message and cover it with a test.
- Separate user-correctable errors from internal defects. Do not expose secrets, full paths, prompts, or raw payloads in an error shown to a user or written to a log.

## Ownership and concurrency

- Shared ownership is explicit. Use `Arc<T>` for genuinely shared immutable data; use `Arc<Mutex<T>>` only when shared mutation is required.
- Prefer channels or actor-style ownership over a large shared mutable graph.
- Never hold a `std::sync::Mutex` or `tokio::sync::Mutex` guard across an `await`.
- Use `tokio::sync` primitives in async code when a lock must be held across an await, and keep the critical section short.
- Define cancellation. A cancelled task must release resources and must not commit a partial result.
- Use `JoinHandle` or a structured concurrency equivalent and await shutdown. Do not detach a task that owns user data.
- Bound every queue and channel. Define what happens when it is full: backpressure, drop with a recorded metric, or fail closed.
- Do not spawn an unbounded number of blocking tasks. Use a bounded worker pool for CPU-bound work.

## Unsafe and FFI

- Put `unsafe` in a dedicated module with a safe wrapper. The wrapper documents the contract and validates inputs.
- Every `unsafe` block has a `// SAFETY:` comment explaining why the pointer, lifetime, alignment, aliasing, and thread assumptions hold.
- Validate all pointers, lengths, UTF-8 assumptions, and ownership transfers at the FFI boundary. Never trust a length or enum value from another process.
- Keep FFI structs `#[repr(C)]` and versioned. Do not pass Rust `String`, `Vec`, or trait objects across an FFI boundary.
- Provide an explicit destroy/free function for every object handed to another language. Test double-free, use-after-free, and null handling where the boundary allows it.
- Do not unwind across an FFI boundary. Catch panics at the boundary and convert them to a defined error code.

## Parsing and untrusted input

- Use a bounded reader. Reject inputs that exceed a documented size, depth, or item count.
- Do not recurse without a depth limit. Prefer an explicit stack for untrusted trees.
- Validate paths component by component. Reject `..`, absolute paths, Windows drive prefixes, UNC paths, and symlink escapes where the contract forbids them.
- Deserialize with a schema that rejects unknown fields when the format requires strictness. Treat duplicate keys and inconsistent lengths as errors.
- Do not log the raw payload of an untrusted document. Log a bounded identifier, size, and error class.

## Performance

- Measure before optimizing. Record the hardware, OS, build profile, workload, command, and result.
- Prefer streaming and borrowing over copying. Use `&str`, `&[u8]`, and slices at boundaries; allocate when ownership is required.
- Use `Vec::with_capacity` only when the size is known or bounded. Do not reserve attacker-controlled sizes.
- Use memory mapping only when the platform and file lifetime make it safe. Document truncation and concurrent-write behavior.
- Avoid unnecessary serialization between modules. Keep a typed in-process representation and serialize only at a real boundary.
- For image, video, and audio data, define the buffer layout, color space, sample format, stride, and ownership in the type. Do not rely on comments.
- Release-profile optimization, LTO, and stripping are decisions backed by measurements and crash-diagnostic requirements, not defaults applied blindly.

## Testing

- Unit tests cover pure logic and edge cases.
- Property tests cover parsers, geometry, serialization, and numeric invariants where a generator is practical.
- Integration tests cover the real workflow across module and process boundaries.
- Regression tests reproduce the defect before the fix.
- Fuzz targets cover untrusted parsers and file formats.
- A test that cannot run in the current environment is reported as `BLOCKED`, not skipped silently.

## Review checklist

- Is the ownership model clear at every boundary?
- Is every `unsafe` block justified and wrapped?
- Are all inputs bounded and validated?
- Are errors typed and causes preserved?
- Are locks and tasks safe under cancellation?
- Is the change the smallest one that fully solves the problem?
- Is there a regression test for the behavior change?
- Is a performance claim backed by a reproducible measurement?
