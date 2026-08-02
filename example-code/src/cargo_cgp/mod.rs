//! Code from the pages under `docs/cargo-cgp/`.
//!
//! Two of the five pages show Rust. `installation.md` and `troubleshooting.md` are shell commands and
//! quoted diagnostics, so they have no file here.
//!
//! Both pages that do show Rust show the *same* program, once broken and once fixed, because that is
//! the point being made: `check.md` runs the tool on the broken one, and `expand.md` expands the fixed
//! one. The broken version cannot be a module, so it is a `compile_fail` doctest.

pub mod check;
pub mod expand;
