# Style Guide

## Rust

- Use `anyhow::Context` on `Option` types. Plain `?` on `Result` — no `.context()` needed.
- No `unwrap()`. Handle errors with `?`, `match`, or `if let`.
- Prefer `if` over early returns (`if x.is_some() { return; }` → use `if` body instead).
- Prefer turbofish syntax: `::<Vec<_>>`, `collect::<Vec<_>>()`.
- Comments go inside function bodies only, as section labels for grouping related steps. No doc comments or section dividers outside functions.
- No `default-features` unless necessary. Enable only the features we actually use.
- Linux + Wayland only for now.

## Build

- Nix flake for dependencies (`nix develop`).
- Build with `cargo build` from `core/`.
- Run with `nix develop --command bash -c "cd core && cargo run"`.
