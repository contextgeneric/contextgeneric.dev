//! Runnable checks for the code shown on the `docs/concepts/` pages.
//!
//! One module per page, under `tests/concepts/`. See the crate `README.md` for the layout and for
//! how the rejected snippets are handled (they live under `tests/compile_fail/`).

// A page shows the shape of a program rather than a whole one, so its code carries items nothing calls
// and parameters nothing reads once an elided body is filled in. A genuinely unused import still warns.
#![allow(dead_code, unused_variables)]

mod concepts;
