//! Code from `docs/reference/providers/handler/return_input.md` — `ReturnInput`.

/// ## Usage and Examples
///
/// `ReturnInput` is the identity handler: it returns its input unchanged. Here it is the middle stage
/// of a pipeline, so the pipeline behaves as though the stage were not there.
pub mod the_identity_stage {
    use cgp::extra::handler::{ComputerComponent, PipeHandlers, ReturnInput};
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn add_one(value: u64) -> u64 {
        value + 1
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: PipeHandlers<Product![AddOne, ReturnInput, AddOne]>,
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
    fn the_input_passes_straight_through() {
        use cgp::extra::handler::CanCompute;

        // AddOne, then ReturnInput (a no-op), then AddOne: 5 -> 6 -> 6 -> 7.
        assert_eq!(App.compute(PhantomData::<()>, 5u64), 7);
    }
}
