//! Code from `docs/reference/components/has_runtime.md` — `HasRuntime`.
//!
//! Pins a context that declares its runtime type with `UseType` and its runtime field with `UseField`,
//! and the generic accessor that reaches the runtime through `HasRuntime`.

/// ## Examples
pub mod examples {
    use cgp::extra::runtime::{
        HasRuntime, RuntimeGetterComponent, RuntimeOf, RuntimeTypeProviderComponent,
    };
    use cgp::prelude::*;

    pub struct TokioRuntime;

    #[derive(HasField)]
    pub struct App {
        pub runtime: TokioRuntime,
    }

    delegate_components! {
        App {
            RuntimeTypeProviderComponent: UseType<TokioRuntime>,
            RuntimeGetterComponent: UseField<Symbol!("runtime")>,
        }
    }

    mod check_app {
        use super::*;

        check_components! {
            App {
                RuntimeTypeProviderComponent,
                RuntimeGetterComponent,
            }
        }
    }

    pub fn runtime_of<Context>(context: &Context) -> &RuntimeOf<Context>
    where
        Context: HasRuntime,
    {
        context.runtime()
    }
}
