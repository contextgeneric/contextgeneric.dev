//! Code from `docs/reference/components/runner.md` — `CanRun`.
//!
//! Pins the `SpawnAndRun` provider from the page's Examples, together with the cast the page names but
//! does not show in full: a base `RunWithFooBar` runner, a `UseDelegate` table routing two tasks, a
//! `SendRunner` proxy on the concrete context, and a `dummy_spawn` standing in for `tokio::spawn`. It
//! follows the library's own async-and-send spawn test. The elided `RunWithFooBar` body is a trivial
//! success here.

/// ## Examples
pub mod examples {
    use core::convert::Infallible;
    use core::future::Future;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::run::{
        CanRun, CanSendRun, Runner, RunnerComponent, SendRunner, SendRunnerComponent,
    };
    use cgp::prelude::*;

    // Stands in for `tokio::spawn`: it demands the future be `Send + 'static`.
    fn dummy_spawn<F>(_future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
    }

    #[cgp_impl(new RunWithFooBar)]
    #[use_type(HasErrorType.Error)]
    impl<Code> Runner<Code> {
        async fn run(&self, _code: PhantomData<Code>) -> Result<(), Error> {
            // The page's `RunWithFooBar` fetches and acts; its body is elided there, so a trivial
            // success stands in.
            Ok(())
        }
    }

    #[cgp_impl(new SpawnAndRun<InCode>: RunnerComponent)]
    #[use_type(HasErrorType.Error)]
    impl<Code, InCode> Runner<Code>
    where
        Self: 'static + Send + Clone + CanSendRun<InCode>,
    {
        async fn run(&self, _code: PhantomData<Code>) -> Result<(), Error> {
            let context = self.clone();

            dummy_spawn(async move {
                let _ = context.send_run(PhantomData).await;
            });

            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct App;

    pub struct ActionA;
    pub struct ActionB;

    delegate_components! {
        App {
            open RunnerComponent;

            ErrorTypeProviderComponent: UseType<Infallible>,
            @RunnerComponent.ActionA: RunWithFooBar,
            @RunnerComponent.ActionB: SpawnAndRun<ActionA>,
        }
    }

    // The `Send` variant is a proxy on the concrete context, forwarding to its own `run`.
    #[cgp_provider]
    impl SendRunner<App, ActionA> for App {
        async fn send_run(context: &App, code: PhantomData<ActionA>) -> Result<(), Infallible> {
            context.run(code).await
        }
    }

    #[test]
    fn spawns_and_runs_a_task() {
        use futures::executor::block_on;

        let app = App;
        block_on(app.run(PhantomData::<ActionB>)).unwrap();
    }
}
