# Fidget Spinner Example

This crate packages the repository's fidget spinner demo as a standalone Cargo
crate.

## Run the example

From the workspace root:

```bash
cargo run -p fidget-spinner
```

Or directly via the manifest path:

```bash
cargo run --manifest-path examples/fidget_spinner/Cargo.toml
```

The binary builds the sample design with `matter-sdk`, compiles it with
`matter-compiler`, and prints the selected manufacturing target, material, pass
report, and graph size.

## Validate the example

Run the end-to-end test from the workspace root:

```bash
cargo test -p fidget-spinner
```
