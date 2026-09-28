//! Code from `docs/reference/providers/handler/pipe_handlers.md` — `PipeHandlers`.
//!
//! Pins the Examples program: three field-reading computers folded into one pipeline that computes
//! `((5 * foo) + bar) * baz`.

/// ## Usage
pub mod usage {
    use cgp::extra::handler::{CanCompute, PipeHandlers};
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn double(value: u64) -> u64 {
        value * 2
    }

    #[cgp_computer]
    pub fn add_one(value: u64) -> u64 {
        value + 1
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: PipeHandlers<Product![Double, AddOne, Double]>,
        }
    }

    #[test]
    fn the_list_runs_left_to_right() {
        assert_eq!(App.compute(PhantomData::<()>, 5), 22); // ((5 * 2) + 1) * 2
    }
}

/// ## Examples
pub mod examples {
    use cgp::extra::handler::{CanCompute, PipeHandlers};
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
    pub struct MyContext {
        pub foo: u64,
        pub bar: u64,
        pub baz: u64,
    }

    delegate_components! {
        MyContext {
            ComputerComponent:
                PipeHandlers<Product![
                    Multiply<Symbol!("foo")>,
                    Add<Symbol!("bar")>,
                    Multiply<Symbol!("baz")>,
                ]>,
        }
    }

    check_components! {
        MyContext {
            ComputerComponent: ((), u64),
        }
    }

    pub fn demo() {
        let context = MyContext { foo: 2, bar: 3, baz: 4 };
        assert_eq!(context.compute(PhantomData::<()>, 5), 52); // ((5 * 2) + 3) * 4
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
