# Website example code on nightly

This crate compiles the site's snippets that need an unstable Rust feature, on the nightly that
[`rust-toolchain.toml`](rust-toolchain.toml) pins. **It is not part of the website**, and it follows
every convention of the stable [`example-code`](../example-code/README.md) crate: one test binary per
section, one file per page, one module per heading, and compile-fail fixtures under
`tests/compile_fail/` with `.stderr` snapshots.

Run it from this directory:

```sh
cargo test              # the full check, compile-fail snapshots included
cargo test --no-run     # the fast check: everything compiles, nothing runs
```

CI runs the same suite with the compile-fail snapshots skipped
(`cargo test -- --skip rejected_snippets_do_not_compile`), in a job of its own beside the stable one.

## What belongs here

A snippet belongs here only when the page shows an unstable feature that the snippet needs. Everything
else stays in `example-code`, so the site's code is checked on stable wherever it can be.

A feature attribute such as `#![feature(const_type_name)]` must sit at a crate root, so each test
binary's root file carries the features its modules need, where the page shows the attribute at the top
of its snippet. That difference is expected and not a defect.

The crate covers these pages:

| Page | Coverage |
|---|---|
| `blog/2026-11-25-incoherent-rust-today.md` (draft) | the four sections that use the `Name` component, whose `RealName` provider calls `core::any::type_name` in a constant and so needs `const_type_name`; `tests/blog/incoherent_rust_today.rs`, with the struct-binding type error as a compile-fail fixture |

## The toolchain

The pin is `nightly-2026-11-16`, the nightly `cargo-cgp` pins. The `.stderr` snapshots quote that
compiler's wording, so bump the pin and re-bless them in one change:

```sh
TRYBUILD=overwrite cargo test --test compile_fail_tests
```

The `cgp` pin matches `example-code`'s and moves with it at each release.
