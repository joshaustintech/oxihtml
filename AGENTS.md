# Agent instructions (oxihtml)

## Scope
These rules apply to the entire repository.

## Project goal
Implement a **std-only** (no external crates) HTML5 parser in Rust that can run and pass the full `html5lib-tests` suite from `~/html5lib-tests`.

## Non-negotiable constraints
- **No external crates**: `Cargo.toml` must not include any dependencies or dev-dependencies.
- Prefer **enums for data modeling** and **composition over inheritance**.
- Minimize `Arc`, `Box`, and `dyn` usage; prefer arena indices (`NodeId`) and `Vec` storage.
- Add multi-threading only where race-free and beneficial (e.g., parallelizing independent test files).

## Repository layout (intended)
- `src/lib.rs`: crate entry, exports, `Options`, `Parser`, DOM types.
- `src/input.rs`: character stream + location tracking.
- `src/tokenizer.rs`: HTML5 tokenizer state machine.
- `src/treebuilder.rs`: HTML5 tree construction.
- `src/dom.rs`: arena-based DOM model.
- `src/serialize.rs`: HTML serialization + html5lib “test format” serializer.
- `src/html5lib.rs`: fixture parsing utilities (`.dat` and JSON `.test`).
- `src/bin/html5lib-runner.rs`: runs `~/html5lib-tests` and reports failures.

## Commands
- Build: `cargo build`
- Unit tests: `cargo test`
- Run html5lib runner (once implemented): `cargo run --bin html5lib-runner -- --all --tests ~/html5lib-tests`
- Tree-construction only: `cargo run --bin html5lib-runner -- --tree --tests ~/html5lib-tests`
- Tokenizer only: `cargo run --bin html5lib-runner -- --tokenizer --tests ~/html5lib-tests`

## Coding conventions
- Keep modules small; avoid giant files where possible.
- Favor total functions with explicit error returns over panics.
- Keep allocations off hot paths where practical (tokenizer).
- When matching html5lib output, match exactly; add targeted regression tests for any fixed mismatch.

