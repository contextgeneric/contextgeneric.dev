//! Code from `docs/reference/components/handler/async_computer_ref.md` — `AsyncComputerRef`.
//!
//! Pins the `CountWords` provider from the page's Examples, wired onto a context and awaited through
//! `CanComputeAsyncRef` over a borrowed input.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanComputeAsyncRef;
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> AsyncComputerRef<Context, Code, String> for CountWords {
        type Output = usize;

        async fn compute_async_ref(
            _context: &Context,
            _code: PhantomData<Code>,
            input: &String,
        ) -> usize {
            input.split_whitespace().count()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            AsyncComputerRefComponent: CountWords,
        }
    }

    check_components! {
        App {
            AsyncComputerRefComponent: ((), String),
        }
    }

    pub async fn run(app: &App, text: &String) -> usize {
        app.compute_async_ref(PhantomData::<()>, text).await
    }

    #[test]
    fn run_counts_the_words_and_keeps_the_input() {
        let text = "hello context generic world".to_owned();

        assert_eq!(futures::executor::block_on(run(&App, &text)), 4);
        assert_eq!(text.len(), 27);
    }
}
