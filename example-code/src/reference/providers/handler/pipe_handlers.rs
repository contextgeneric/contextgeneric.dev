//! Code from `docs/reference/providers/handler/pipe_handlers.md` — `PipeHandlers`.

/// ## Usage and Examples
///
/// `PipeHandlers` folds a list of handlers into one pipeline. `Multiply<Field>` and `Add<Field>` each
/// read a factor or addend from a context field, so the three-stage list computes
/// `((5 * foo) + bar) * baz`.
pub mod a_pipeline_of_field_readers {
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

        let app = App { foo: 2, bar: 3, baz: 4 };
        // ((5 * 2) + 3) * 4
        assert_eq!(app.compute(PhantomData::<()>, 5u64), 52);
    }
}
