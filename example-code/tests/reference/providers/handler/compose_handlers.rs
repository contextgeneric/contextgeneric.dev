//! Code from `docs/reference/providers/handler/compose_handlers.md` — `ComposeHandlers`.
//!
//! Pins the Examples program: `Double` then `AddOne`, composed and wired as one computer.

/// ## Examples
pub mod examples {
    use cgp::extra::handler::{CanCompute, ComposeHandlers};
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

    check_components! {
        App {
            ComputerComponent: ((), u64),
        }
    }

    pub fn demo() {
        assert_eq!(App.compute(PhantomData::<()>, 5), 11); // (5 * 2) + 1
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
