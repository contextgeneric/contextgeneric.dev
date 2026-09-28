//! Code from `docs/reference/providers/handler/promote_producer.md` — `PromoteProducer`.
//!
//! Pins the Examples program: the `ComputerComponent` entry serves any producer, and a
//! `#[cgp_producer]` provider answers the rest of the family too. The hand-written producer behind a
//! fallible entry is the C2 fixture
//! `tests/compile_fail/reference/components/producer_common_mistakes_bundle_on_a_plain_producer.rs`.

/// ## Examples
pub mod examples {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{CanCompute, CanProduce, CanTryCompute};
    use cgp::prelude::*;

    #[cgp_producer]
    pub fn default_port() -> u16 {
        8080
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ProducerComponent: DefaultPort,
            [ComputerComponent, TryComputerComponent]: PromoteProducer<DefaultPort>,
        }
    }

    check_components! {
        App {
            ProducerComponent: (),
            [ComputerComponent, TryComputerComponent]: ((), u64),
        }
    }

    pub fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.produce(code), 8080);
        assert_eq!(App.compute(code, 999_u64), 8080);
        assert_eq!(App.try_compute(code, 999_u64), Ok(8080));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
