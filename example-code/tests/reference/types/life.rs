//! Code from `docs/reference/types/life.md` — *`Life`*.
//!
//! `Life` is inserted by the macros into the dependency marker of a component with a lifetime, and a
//! reader names it in one place: the `check_components!` entry for such a component. The Examples
//! program wires and checks a borrowing getter component and calls it. The generated provider trait,
//! which records the lifetime as `Life<'a>`, is checked with `cargo cgp expand`. The unsized-target
//! defect the page records is a trybuild fixture under `tests/compile_fail/reference/types/`.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(ReferenceGetter)]
    pub trait HasReference<'a, T: 'a + ?Sized> {
        fn get_reference(&self) -> &'a T;
    }

    pub struct Config {
        pub name: String,
    }

    #[cgp_impl(new GetConfig)]
    #[uses(HasField<Symbol!("config"), Value = &'a Config>)]
    impl<'a> ReferenceGetter<'a, Config> {
        fn get_reference(&self) -> &'a Config {
            self.get_field(PhantomData::<Symbol!("config")>)
        }
    }

    #[derive(HasField)]
    pub struct App<'a> {
        pub config: &'a Config,
    }

    delegate_components! {
        <'a> App<'a> {
            ReferenceGetterComponent: GetConfig,
        }
    }

    check_components! {
        <'a> App<'a> {
            ReferenceGetterComponent: (Life<'a>, Config),
        }
    }

    pub fn demo() {
        let config = Config {
            name: "demo".to_owned(),
        };
        let app = App { config: &config };

        let config_ref: &Config = app.get_reference();
        assert_eq!(config_ref.name, "demo");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Definition
///
/// The covariant half of the invariance comparison: a function that shortens a
/// `PhantomData<&'l ()>` compiles, while the same function over `Life<'l>` is the fixture
/// `life_definition_invariant.rs`. A `Life` is zero-sized.
pub mod definition {
    use cgp::prelude::*;

    pub fn shorten<'s, 'l: 's>(marker: PhantomData<&'l ()>) -> PhantomData<&'s ()> {
        marker
    }

    #[test]
    fn test_life_is_zero_sized() {
        assert_eq!(core::mem::size_of::<Life<'static>>(), 0);
    }
}
