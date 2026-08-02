//! Code from `docs/concepts/handlers.md` — *Handlers*.

/// ## One function, the whole family
///
/// A synchronous, infallible function becomes a provider callable as every member of the family,
/// because CGP promotes the weaker variants into the stronger ones.
pub mod one_function_the_whole_family {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    #[cgp_computer]
    fn add(a: u64, b: u64) -> u64 {
        a + b
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,
        }
    }

    #[test]
    fn the_same_definition_answers_four_components() {
        use futures::executor::block_on;

        let app = App;

        assert_eq!(Add::compute(&app, PhantomData::<()>, (1, 2)), 3);
        assert_eq!(Add::try_compute(&app, PhantomData::<()>, (1, 2)), Ok(3));
        assert_eq!(block_on(Add::compute_async(&app, PhantomData::<()>, (1, 2))), 3);
        assert_eq!(block_on(Add::handle(&app, PhantomData::<()>, (1, 2))), Ok(3));
    }
}

/// ## A fallible function, promoted the same way
pub mod a_fallible_function {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    #[cgp_computer]
    fn checked_add(a: u64, b: u64) -> Result<u64, String> {
        a.checked_add(b).ok_or_else(|| "overflow".to_owned())
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,
        }
    }

    #[test]
    fn the_error_path_surfaces_through_the_fallible_members() {
        let app = App;

        assert_eq!(CheckedAdd::try_compute(&app, PhantomData::<()>, (1, 2)), Ok(3));
        assert_eq!(
            CheckedAdd::try_compute(&app, PhantomData::<()>, (u64::MAX, 1)),
            Err("overflow".to_owned()),
        );
    }
}

/// ## A pipeline is a wiring entry
pub mod a_pipeline_is_a_wiring_entry {
    use core::marker::PhantomData;

    use cgp::extra::handler::{Computer, ComputerComponent, PipeHandlers};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code, Field> Computer<Context, Code, u64> for Multiply<Field>
    where
        Context: HasField<Field, Value = u64>,
    {
        type Output = u64;

        fn compute(context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * context.get_field(PhantomData)
        }
    }

    #[cgp_new_provider]
    impl<Context, Code, Field> Computer<Context, Code, u64> for Add<Field>
    where
        Context: HasField<Field, Value = u64>,
    {
        type Output = u64;

        fn compute(context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input + context.get_field(PhantomData)
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub foo: u64,
        pub bar: u64,
        pub baz: u64,
    }

    delegate_components! {
        App {
            ComputerComponent: PipeHandlers<
                Product![
                    Multiply<Symbol!("foo")>,
                    Add<Symbol!("bar")>,
                    Multiply<Symbol!("baz")>,
                ]
            >,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: ((), u64),
            }
        }
    }

    #[test]
    fn the_stages_run_left_to_right() {
        use cgp::extra::handler::CanCompute;

        let app = App {
            foo: 2,
            bar: 3,
            baz: 4,
        };

        // ((5 * 2) + 3) * 4
        assert_eq!(app.compute(PhantomData::<()>, 5u64), 52);
    }
}

/// ## The `Code` tag is what lets one context host many handlers
pub mod the_code_tag_hosts_many_handlers {
    use core::marker::PhantomData;

    use cgp::extra::handler::{Computer, ComputerComponent};
    use cgp::prelude::*;

    pub struct Doubled;
    pub struct Negated;

    #[cgp_new_provider]
    impl<Context> Computer<Context, Doubled, i64> for ComputeDoubled {
        type Output = i64;

        fn compute(_context: &Context, _code: PhantomData<Doubled>, input: i64) -> i64 {
            input * 2
        }
    }

    #[cgp_new_provider]
    impl<Context> Computer<Context, Negated, i64> for ComputeNegated {
        type Output = i64;

        fn compute(_context: &Context, _code: PhantomData<Negated>, input: i64) -> i64 {
            -input
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ComputerComponent;

            @ComputerComponent.Doubled.i64: ComputeDoubled,
            @ComputerComponent.Negated.i64: ComputeNegated,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: [(Doubled, i64), (Negated, i64)],
            }
        }
    }

    #[test]
    fn two_computations_on_one_context() {
        use cgp::extra::handler::CanCompute;

        assert_eq!(App.compute(PhantomData::<Doubled>, 21i64), 42);
        assert_eq!(App.compute(PhantomData::<Negated>, 21i64), -21);
    }
}
