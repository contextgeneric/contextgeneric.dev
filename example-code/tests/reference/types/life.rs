//! Code from `docs/reference/types/life.md` — *`Life`*.
//!
//! `Life` is inserted by the macros; the only user-written code the page shows is a component whose
//! consumer trait carries a lifetime. This pins that `#[cgp_component]` accepts that form. The
//! generated provider trait, which records the lifetime as `Life<'a>`, is shown as a comment on the
//! page and checked by `cargo cgp expand` rather than here.

/// ## Examples
///
/// A borrowing getter component. Defining it is the check: the macro accepts the lifetime and lifts it
/// into the dependency marker.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(ReferenceGetter)]
    pub trait HasReference<'a, T: 'a + ?Sized> {
        fn get_reference(&self) -> &'a T;
    }
}
