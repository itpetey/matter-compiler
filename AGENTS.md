# Repository Guidelines

## Project Structure & Module Organization

This repository is a small Rust workspace centred on domain crates under `crates/`.

- `crates/manufacturing-context/src/lib.rs`: manufacturing environment schema and constraints.
- `crates/ir/src/lib.rs`: intermediate representation graph types.
- `README.md`: project direction, terminology, and contributor-facing context.
- `LICENCE`: MPL 2.0 licence text.

Keep new crates under `crates/<crate-name>/` with a `Cargo.toml` and `src/lib.rs`. Prefer focused library crates over large mixed-purpose modules.

## Build, Test, and Development Commands

Run commands from the repository root unless you are working inside a single crate.

```bash
cargo check -p manufacturing-context
cargo check -p ir
cargo test -p manufacturing-context
cargo test -p ir
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

Use `cargo check` for fast validation, `cargo test` for unit tests and doctests, `cargo fmt` for formatting, and `cargo clippy` to catch lint regressions before opening a PR.

## Coding Style & Naming Conventions

Use the 2024 edition of Rust.

Follow standard Rust style: 4-space indentation, `snake_case` for functions and modules, `PascalCase` for types and enums, and `SCREAMING_SNAKE_CASE` for constants such as `IR_SCHEMA_VERSION`. Keep public APIs small and explicit. Prefer deriving common traits (`Debug`, `Clone`, `PartialEq`) where appropriate.

All public interfaces must be documented with Rust doc comments. Write repository prose in International English. Keep comments and doc examples short, factual, and directly tied to the crate API.

Code files have a soft limit of 500 lines. If a file grows beyond that, split it into focused submodules instead of extending a single large source file.

## Testing Guidelines

Place unit tests beside the code they exercise using `#[cfg(test)] mod tests`. Use Rust doctests for public API examples in doc comments. Name tests for the behaviour they verify, for example `rejects_zero_build_volume` or `creates_initial_prototype_context`.

## Commit & Pull Request Guidelines

Recent history uses short, imperative subjects such as `Initial pass of IR graph` and `More README patches`. Keep commit titles concise, capitalised, and focused on one change.

PRs should include:

- a brief summary of the change and affected crates
- the validation commands you ran
- linked issues or design notes when relevant
- note whether the change is code, docs, or both
