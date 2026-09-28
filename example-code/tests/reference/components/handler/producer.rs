//! Code from `docs/reference/components/handler/producer.md` — `Producer`.
//!
//! Pins the `MagicNumber` provider from the page's Examples, wired onto a context and invoked through
//! `CanProduce`, and the `#[cgp_producer]` provider the page's Usage section wires into the
//! input-taking components. The hand-written producer behind a bundle, from Common Mistakes, is a
//! trybuild fixture.

/// ## Usage
///
/// A `#[cgp_producer]` provider is wired to its own promotion bundle, so a context can wire it to any
/// member of the family.
pub mod usage {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanCompute, CanHandle};
    use cgp::prelude::*;

    #[cgp_producer]
    fn default_port() -> u16 {
        8080
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            [ComputerComponent, HandlerComponent]: DefaultPort,
        }
    }

    check_components! {
        App {
            [ComputerComponent, HandlerComponent]: ((), ()),
        }
    }

    #[test]
    fn the_producer_answers_the_input_taking_members() {
        assert_eq!(App.compute(PhantomData::<()>, ()), 8080);
        assert_eq!(
            futures::executor::block_on(App.handle(PhantomData::<()>, ())),
            Ok(8080)
        );
    }
}

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanProduce;
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Producer<Context, Code> for MagicNumber {
        type Output = u64;

        fn produce(_context: &Context, _code: PhantomData<Code>) -> u64 {
            42
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ProducerComponent: MagicNumber,
        }
    }

    check_components! {
        App {
            ProducerComponent: (),
        }
    }

    pub fn demo() {
        assert_eq!(App.produce(PhantomData::<()>), 42);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Common Mistakes
///
/// The fix the page gives: a hand-written producer wired to `PromoteProducer<Self>`, as
/// `#[cgp_producer]` does, answers the fallible members. The unwired producer behind the bundle is
/// `tests/compile_fail/reference/components/producer_common_mistakes_bundle_on_a_plain_producer.rs`.
pub mod common_mistakes {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::CanTryCompute;
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Producer<Context, Code> for MagicNumber {
        type Output = u64;

        fn produce(_context: &Context, _code: PhantomData<Code>) -> u64 {
            42
        }
    }

    delegate_components! {
        MagicNumber {
            [ComputerComponent, TryComputerComponent]: PromoteProducer<Self>,
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            TryComputerComponent: MagicNumber,
        }
    }

    check_components! {
        App {
            TryComputerComponent: ((), ()),
        }
    }

    #[test]
    fn the_self_wired_producer_answers_try_compute() {
        assert_eq!(App.try_compute(PhantomData::<()>, ()), Ok(42));
    }
}
