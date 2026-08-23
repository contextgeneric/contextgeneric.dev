//! Code from `docs/reference/providers/handler/compose_handlers.md` — `ComposeHandlers`.

/// ## Usage and Examples
///
/// `ComposeHandlers<A, B>` runs `A`, then `B` on `A`'s output. Here `Double` then `AddOne` turns an
/// input of 5 into `(5 * 2) + 1`.
pub mod composing_two_handlers {
    use cgp::extra::handler::{ComposeHandlers, ComputerComponent};
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
            ComputerComponent: ComposeHandlers<Double, AddOne>,
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
    fn the_output_of_the_first_feeds_the_second() {
        use cgp::extra::handler::CanCompute;

        assert_eq!(App.compute(PhantomData::<()>, 5u64), 11);
    }
}
