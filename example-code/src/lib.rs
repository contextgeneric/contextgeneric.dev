//! Compiled counterparts of the code shown on <https://contextgeneric.dev>.
//!
//! This crate is **not part of the website**. Nothing here is rendered, linked, or served, and no
//! page refers to it. It exists so that an agent revising a documentation page can check the code on
//! that page against something the compiler has agreed to, instead of reading the snippet and hoping.
//!
//! The layout mirrors the `docs/` tree one file per page: the code on
//! `docs/concepts/consumer-and-provider-traits.md` lives in
//! `src/concepts/consumer_and_provider_traits.rs`, and so on for every page that shows code. Pages
//! that show none get no file. Duplication between files is expected and wanted — each file answers
//! for one page, so a shared example is written out again rather than factored into a helper that
//! neither page shows.
//!
//! See `README.md` for how to use it, what "matches" means when a page elides a body, and the rules
//! for adding a file.

// A page shows the shape of a program rather than a whole one, so its code is full of items nothing
// calls and parameters nothing reads once an elided body is filled in. Those warnings would be noise
// here; a warning that means something — an unused import, an unreachable pattern — still fires.
#![allow(dead_code, unused_variables)]

pub mod cargo_cgp;
pub mod concepts;
pub mod reference;
