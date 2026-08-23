//! Code from `docs/reference/components/send_runner.md` — `CanSendRun`.
//!
//! Pins the concrete `SendRunner` proxy from the page's Examples, together with the `SpawnAndRun`
//! spawner that consumes it and the cast needed to compile and run them: a base `RunWithFooBar`
//! runner, a `UseDelegate` table, and a `dummy_spawn` standing in for `tokio::spawn`. It follows the
//! library's own async-and-send spawn test. The proxy is what discharges the `Send` bound at the
//! concrete context.

/// ## Examples
pub mod examples {
    use core::convert::Infallible;
    use core::future::Future;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::run::{
        CanRun, CanSendRun, Runner, RunnerComponent, SendRunner, SendRunnerComponent,
    };
    use cgp::prelude::*;
    use futures::executor::block_on;

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

    // Because this impl names the concrete `App` and `ActionA`, the future `context.run(code)` produces
    // has a fully known type, so the compiler can prove it is `Send` and satisfy the `+ Send` bound.
    #[cgp_provider]
    impl SendRunner<App, ActionA> for App {
        async fn send_run(context: &App, code: PhantomData<ActionA>) -> Result<(), Infallible> {
            context.run(code).await
        }
    }

    #[test]
    fn the_proxy_lets_a_task_be_spawned() {
        let app = App;
        block_on(app.run(PhantomData::<ActionB>)).unwrap();
    }
}
