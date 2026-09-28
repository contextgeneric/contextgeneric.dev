//! Code from `docs/reference/components/handler/async_computer.md` — `AsyncComputer`.
//!
//! Pins the `DoubleAsync` provider from the page's Examples, wired onto a context and awaited through
//! `CanComputeAsync`, and the providers the page's Usage section lists: a promoted synchronous
//! computer, a `#[cgp_computer]` async function, and `UseField`.

/// ## Usage
///
/// A synchronous `Computer` answers the async slot through `PromoteAsync`.
pub mod usage {
    use core::marker::PhantomData;

    use cgp::extra::handler::{CanComputeAsync, PromoteAsync};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, u64> for Double {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            AsyncComputerComponent: PromoteAsync<Double>,
        }
    }

    check_components! {
        App {
            AsyncComputerComponent: ((), u64),
        }
    }

    #[test]
    fn the_promoted_computer_answers_compute_async() {
        let output = futures::executor::block_on(App.compute_async(PhantomData::<()>, 21));
        assert_eq!(output, 42);
    }

    /// `#[cgp_computer]` on an `async fn` implements `AsyncComputer` directly, and `UseField<Tag>`
    /// forwards the async computation to a field whose value computes it.
    pub mod sources {
        use core::marker::PhantomData;

        use cgp::extra::handler::CanComputeAsync;
        use cgp::prelude::*;

        #[cgp_computer]
        async fn double_later(input: u64) -> u64 {
            input * 2
        }

        pub struct Doubler;

        delegate_components! {
            Doubler {
                AsyncComputerComponent: DoubleLater,
            }
        }

        #[derive(HasField)]
        pub struct App {
            pub doubler: Doubler,
        }

        delegate_components! {
            App {
                AsyncComputerComponent: UseField<Symbol!("doubler")>,
            }
        }

        check_components! {
            App {
                AsyncComputerComponent: ((), u64),
            }
        }

        #[test]
        fn both_sources_answer_compute_async() {
            let app = App { doubler: Doubler };

            assert_eq!(
                futures::executor::block_on(Doubler.compute_async(PhantomData::<()>, 21)),
                42
            );
            assert_eq!(
                futures::executor::block_on(app.compute_async(PhantomData::<()>, 21)),
                42
            );
        }
    }
}

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanComputeAsync;
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> AsyncComputer<Context, Code, u64> for DoubleAsync {
        type Output = u64;

        async fn compute_async(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            AsyncComputerComponent: DoubleAsync,
        }
    }

    check_components! {
        App {
            AsyncComputerComponent: ((), u64),
        }
    }

    pub async fn run(app: &App) -> u64 {
        app.compute_async(PhantomData::<()>, 21).await
    }

    #[test]
    fn run_awaits_the_doubled_input() {
        assert_eq!(futures::executor::block_on(run(&App)), 42);
    }
}
