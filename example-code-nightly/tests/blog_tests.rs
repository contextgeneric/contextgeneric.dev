//! Compiled counterparts of blog-post examples that need a nightly feature.
// The post's `Name` snippets open with this attribute; a module cannot carry it, so the
// test binary's root does.
#![feature(const_type_name)]

#[path = "blog/incoherent_rust_today.rs"]
mod incoherent_rust_today;
