//! Code from `docs/reference/types/chars.md` — *`Chars`*.
//!
//! A `Symbol!` wraps a `Chars` chain. The page reads a field by its `Symbol!` tag and rebuilds the
//! string from the type; both run here. The greet provider duplicates the one on the `PhantomData`
//! page, which the crate prefers over a shared helper so each file answers for its own page.

/// ## Examples
///
/// A field-name tag drives a getter, and a `Symbol!` rebuilds its string through `Display`.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter
    where
        Self: HasField<Symbol!("name"), Value = String>,
    {
        fn greet(&self) {
            println!("Hello, {}!", self.get_field(PhantomData::<Symbol!("name")>));
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    delegate_components! {
        App {
            GreeterComponent: GreetHello,
        }
    }

    check_components! {
        App {
            GreeterComponent
        }
    }

    #[test]
    fn test_a_symbol_rebuilds_its_string() {
        let s = <Symbol!("hello")>::default();
        assert_eq!(s.to_string(), "hello");
    }

    #[test]
    fn test_greet_reads_the_name_field() {
        let app = App {
            name: "World".to_owned(),
        };
        app.greet();
    }
}
