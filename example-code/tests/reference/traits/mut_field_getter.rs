//! Code from `docs/reference/traits/field-access/mut_field_getter.md` — `MutFieldGetter`.
//!
//! Pins the Examples program: a getter with a `&mut self` method, answered through `WithField`,
//! whose `WithProvider` impl bounds on `MutFieldGetter`.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::WithField;
    use cgp::prelude::*;

    #[cgp_getter(CounterGetter)]
    pub trait HasCounter {
        fn counter_mut(&mut self) -> &mut u64;
    }

    #[derive(HasField)]
    pub struct App {
        pub request_count: u64,
    }

    delegate_components! {
        App {
            CounterGetterComponent: WithField<Symbol!("request_count")>,
        }
    }

    check_components! {
        App {
            CounterGetterComponent,
        }
    }

    pub fn demo() {
        let mut app = App { request_count: 0 };
        *app.counter_mut() += 1;
        assert_eq!(app.request_count, 1);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
