//! Code from `docs/reference/types/mref.md` — *`MRef`*.
//!
//! `MRef` is an ordinary runtime value, not a type-level marker. The Examples program declares a
//! getter returning `MRef`, wires one context to lend a stored field and another to build the value,
//! and reads both through one generic function.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasGreeting {
        fn greeting(&self) -> MRef<'_, String>;
    }

    // A provider that builds the value instead of lending a field.
    #[cgp_impl(new BuildGreeting)]
    impl GreetingGetter {
        fn greeting(&self, #[implicit] name: &str) -> MRef<'_, String> {
            MRef::Owned(format!("Hello, {name}!"))
        }
    }

    #[derive(HasField)]
    pub struct Stored {
        pub greeting: String,
    }

    #[derive(HasField)]
    pub struct Computed {
        pub name: String,
    }

    delegate_components! {
        Stored {
            GreetingGetterComponent: UseField<Symbol!("greeting")>,
        }
    }

    delegate_components! {
        Computed {
            GreetingGetterComponent: BuildGreeting,
        }
    }

    check_components! {
        Stored {
            GreetingGetterComponent,
        }
    }

    check_components! {
        Computed {
            GreetingGetterComponent,
        }
    }

    // Generic code reads either case through `Deref`.
    pub fn shout<Context: HasGreeting>(context: &Context) -> String {
        context.greeting().to_uppercase()
    }

    pub fn demo() {
        let stored = Stored {
            greeting: "Hi there".to_owned(),
        };
        let computed = Computed {
            name: "Alice".to_owned(),
        };

        assert_eq!(shout(&stored), "HI THERE");
        assert_eq!(shout(&computed), "HELLO, ALICE!");

        // `UseField` lends the stored field, and `get_or_clone` moves an owned value out.
        assert!(matches!(stored.greeting(), MRef::Ref(_)));
        let owned: String = computed.greeting().get_or_clone();
        assert_eq!(owned, "Hello, Alice!");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
