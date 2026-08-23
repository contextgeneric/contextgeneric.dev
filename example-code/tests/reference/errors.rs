//! Code from `docs/reference/errors.md` — *Compile errors*.
//!
//! Almost every block on that page is compiler *output* rather than Rust, so only one section shows
//! a program. That section carries the page's whole point in two halves that must both stay true:
//! the wiring compiles on its own, and the same wiring fails the moment a check asks the question.
//! Each half is pinned separately below, since a page claiming "this compiles, that does not" is
//! wrong if either half moves.

/// ## A dependency is not met
///
/// The page's one program: `GreetHello` needs a `name` field through
/// [`HasName`](https://contextgeneric.dev/docs/reference/macros/cgp_auto_getter), and `Person` has
/// only `age`.
///
/// **This module is the "that block compiles" claim.** The page says so in bold, because it is the
/// whole reason the error arrives later and somewhere else: `delegate_components!` records the
/// choice without proving the provider can honour it. If this module ever stops compiling, wiring
/// has stopped being lazy and the page's framing is wrong.
///
/// Deliberately **no `check_components!` here**, departing from this crate's usual rule of adding
/// one per wired context — the check is exactly what the page withholds until the next section, and
/// adding it would make this module fail.
pub mod a_dependency_is_not_met {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_impl(new GreetHello)]
    #[uses(HasName)]
    impl Greeter {
        fn greet(&self) {
            let _ = self.name();
        }
    }

    #[derive(HasField)]
    pub struct Person {
        pub age: u8, // no `name` field, so `Person` cannot satisfy `HasName`
    }

    delegate_components! {
        Person {
            GreeterComponent: GreetHello,
        }
    }
}

/// ### The fix
///
/// The other half: the same wiring, with the check the page adds to force the failure to the wiring
/// site. The page quotes the resulting `E0277` — `[CGP-E001]` through the toolchain — so the
/// snippet is carried as a `compile_fail` doctest.
///
/// The program is repeated in full rather than importing the module above, because a doctest is
/// compiled as its own crate and because the page shows it whole.
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/reference/errors_the_fix_1.rs`.
///
/// And the hidden half of the same mistake, which the page contrasts against the checked one: no
/// check anywhere, the failure reached by calling the method instead. The compiler reports `E0599`
/// and names nothing about the missing field.
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/reference/errors_the_fix_2.rs`.
pub mod the_fix {}
