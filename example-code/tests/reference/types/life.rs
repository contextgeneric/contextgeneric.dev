//! Code from `docs/reference/types/life.md` — *`Life`*.
//!
//! `Life` is inserted by the macros into the dependency marker of a component with a lifetime, and a
//! reader names it in one place: the `check_components!` entry for such a component. The Examples
//! program wires and checks a borrowing getter component and calls it. The generated provider trait,
//! which records the lifetime as `Life<'a>`, is checked with `cargo cgp expand`. The second
//! Examples program wires the same component at the unsized target `str` and calls it.

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

    // The unsized target: the parameter tuple `(Life<'a>, str)` is unsized, and the table's
    // forwarding impl accepts it.
    #[cgp_impl(new GetName)]
    #[uses(HasField<Symbol!("name"), Value = &'a str>)]
    impl<'a> ReferenceGetter<'a, str> {
        fn get_reference(&self) -> &'a str {
            self.get_field(PhantomData::<Symbol!("name")>)
        }
    }

    #[derive(HasField)]
    pub struct Borrowed<'a> {
        pub name: &'a str,
    }

    delegate_components! {
        <'a> Borrowed<'a> {
            ReferenceGetterComponent: GetName,
        }
    }

    check_components! {
        <'a> Borrowed<'a> {
            ReferenceGetterComponent: (Life<'a>, str),
        }
    }

    pub fn demo_unsized() {
        let text = String::from("demo");
        let borrowed = Borrowed { name: &text };

        let name: &str = borrowed.get_reference();
        assert_eq!(name, "demo");
    }

    #[test]
    fn test_demo_unsized() {
        demo_unsized();
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
