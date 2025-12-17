# OxiHTML spec + roadmap

## Objective
Port the Python JustHTML project (`~/justhtml`) to a **pure Rust** implementation (std-only; no external crates) that passes the full `html5lib-tests` suite (`~/html5lib-tests`).

## Hard constraints
- No external crates (no deps, no dev-deps).
- Enums for modeling; composition over inheritance.
- Minimize `Arc`, `Box`, and `dyn`; prefer arenas + indices.
- Multi-thread where safe/beneficial (test running is embarrassingly parallel).

## Public API (draft)
See `API.md`.

## Definition of done (success criteria)
- `cargo run --bin html5lib-runner -- --all --tests ~/html5lib-tests` exits `0`.
- All tokenizer, tree-construction, and serializer fixtures in `~/html5lib-tests` pass.
- `Cargo.toml` has **zero** dependencies/dev-dependencies.
- Library API supports parsing documents/fragments and serializing to html5lib “test format”.

## Milestones

### Milestone 0 — Rust skeleton + runner scaffolding
Status: COMPLETE
Scope:
- Create a std-only Rust crate.
- Add `html5lib-runner` binary with CLI flags and stable reporting.
- Discover fixture files under `~/html5lib-tests` (or `--tests <path>`).
- Add parallel execution across files (thread-per-file with bounded parallelism).

Acceptance:
- `cargo run --bin html5lib-runner -- --list --tests ~/html5lib-tests` lists discovered files and counts.
- `cargo run --bin html5lib-runner -- --tree --tests ~/html5lib-tests` runs, even if all cases fail, but never panics.

Stop condition:
- Runner exits non-zero on failures and prints top N failing cases with file + case index.

### Milestone 1 — Fixture parsers (std-only)
Status: INCOMPLETE
Scope:
- Parse tree-construction `.dat` files (sections `#data`, `#errors`, `#document`, `#document-fragment`, `#script-on/off`, `#document-fragment <ctx>`).
- Parse tokenizer/serializer `.test` JSON files using a minimal JSON parser (objects, arrays, strings with escapes, numbers, booleans, null).
- Normalize/compare outputs with html5lib rules used by JustHTML (`to_test_format` style).

Acceptance:
- All fixtures load without panic.
- Runner can report case counts per file and can print expected outputs for a selected case.

Stop condition:
- A `--smoke` run completes across all fixtures without panics.

### Milestone 2 — DOM + html5lib serialization
Status: INCOMPLETE
Scope:
- Implement arena-based DOM (`dom.rs`) and `serialize::to_test_format` matching html5lib tree format.
- Implement minimal constructors used by the tree builder (append child, insert before, detach, etc.).

Acceptance:
- For a set of hand-constructed DOMs (unit tests), serializer matches expected strings exactly.

Stop condition:
- Tree serializer is deterministic (stable attribute ordering, namespace formatting).

### Milestone 3 — Tokenizer (HTML5)
Status: INCOMPLETE
Scope:
- Implement HTML5 tokenizer state machine (as in JustHTML `tokenizer.py`), including character references/entities.
- Emit token stream types suitable for tree builder.
- Track locations for error reporting.

Acceptance:
- `cargo run --bin html5lib-runner -- --tokenizer --tests ~/html5lib-tests` reaches a steadily improving pass rate; iterate until 100%.

Stop condition:
- Tokenizer fixtures pass 100% with exact errors where asserted.

### Milestone 4 — Tree construction (HTML5)
Status: INCOMPLETE
Scope:
- Implement insertion modes, stack of open elements, active formatting elements, foster parenting, foreign content (SVG/MathML), template insertion mode stack.
- Support both document and fragment parsing contexts.

Acceptance:
- `cargo run --bin html5lib-runner -- --tree --tests ~/html5lib-tests` reaches 100% pass.

Stop condition:
- All tree-construction `.dat` cases pass.

### Milestone 5 — Serializer fixtures
Status: INCOMPLETE
Scope:
- Implement HTML serializer options required by `html5lib-tests/serializer/*.test`.
- Ensure entity escaping/attribute quoting behaviors match fixtures.

Acceptance:
- `cargo run --bin html5lib-runner -- --serializer --tests ~/html5lib-tests` passes 100%.

Stop condition:
- All serializer fixtures pass.

### Milestone 6 — Polish + performance
Status: INCOMPLETE
Scope:
- Tighten hot paths (tokenizer), reduce allocations, add regression unit tests for tricky fixed cases.
- Ensure public API ergonomics align with `API.md`.

Acceptance:
- Full suite still passes; runner output stable.

## Risks / notes
- Implementing the full HTML5 tree builder is the largest scope item; use failure categorization (by error code / insertion mode) to drive progress.
- JSON parsing is required without `serde_json`; keep it minimal and tailored to fixture shapes.
- Multi-threading is safest in the runner: parsing individual cases must be isolated per thread.
