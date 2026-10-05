//! Code from the pages under `docs/cargo-cgp/`.
//!
//! Four of the eight pages show Rust: `index.md`, `check.md`, `reading-output.md`, and `expand.md`.
//! The rest are shell commands, quoted messages, and tables, so they have no file here.
//!
//! Almost every program those pages show is broken on purpose, because the point of each is the error
//! `cargo cgp check` prints for it. Those programs are `trybuild` fixtures under
//! `tests/compile_fail/cargo_cgp/`, one per heading, named `<page>_<heading>.rs`. The modules here hold
//! the programs the pages say compile: the repaired `Rectangle` that `expand.md` expands, and the
//! repaired `greeting` that `reading-output.md` gives as the fix for `[CGP-E012]`.
//!
//! Neither the fixtures nor these modules check the `cargo cgp check` or `cargo cgp expand` output the
//! pages quote. That output was produced by running the tool on exactly these programs, and re-running
//! it is the only way to re-verify it.

pub mod check;
pub mod expand;
pub mod reading_output;
