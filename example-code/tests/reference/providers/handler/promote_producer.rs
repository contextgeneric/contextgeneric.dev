//! Code from `docs/reference/providers/handler/promote_producer.md` — `PromoteProducer`.

/// ## Usage and Examples
///
/// `PromoteProducer` fills in the input-taking family members from an input-free `Producer`. The base
/// answers `ProducerComponent`, and the bundle answers `ComputerComponent` (and the rest) by
/// discarding the input and calling the producer.
pub mod filling_the_family_from_a_producer {
    use cgp::extra::handler::{ComputerComponent, ProducerComponent, PromoteProducer};
    use cgp::prelude::*;

    #[cgp_producer]
    pub fn make_default() -> u64 {
        42
    }

    pub struct App;

    delegate_components! {
        App {
            // The base answers `ProducerComponent`; the bundle answers `ComputerComponent` by
            // discarding the input and calling the producer.
            ProducerComponent: MakeDefault,
            ComputerComponent: PromoteProducer<MakeDefault>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ProducerComponent: (),
                ComputerComponent: ((), u64),
            }
        }
    }

    #[test]
    fn the_produced_value_flows_out_of_every_shape() {
        use cgp::extra::handler::{CanCompute, CanProduce};

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(app.produce(code), 42);
        // The computer shape discards its input and produces the same value.
        assert_eq!(app.compute(code, 999u64), 42);
    }
}
