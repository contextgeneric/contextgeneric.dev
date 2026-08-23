//! Code from `docs/reference/providers/redirect_lookup.md` — *`RedirectLookup`*.
//!
//! `RedirectLookup` is generated machinery, so the only user-writable code the page shows is the
//! namespace registration that produces the redirect entries. This pins that that registration
//! compiles.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    pub struct App;
    pub struct TestProvider;

    delegate_components! {
        App {
            namespace DefaultNamespace;

            @bar.baz: TestProvider,
        }
    }
}
