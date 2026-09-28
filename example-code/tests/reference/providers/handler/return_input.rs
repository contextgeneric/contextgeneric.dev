//! Code from `docs/reference/providers/handler/return_input.md` — `ReturnInput`.
//!
//! Pins the Usage wiring and the Examples program, in which `ReturnInput` is the middle stage of a
//! pipeline and changes nothing.

/// ## Usage
pub mod usage {
    use cgp::extra::handler::{CanCompute, ReturnInput};
    use cgp::prelude::*;

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: ReturnInput,
        }
    }

    #[test]
    fn any_input_comes_back() {
        assert_eq!(App.compute(PhantomData::<()>, "hello"), "hello");
    }
}

/// ## Examples
pub mod examples {
    use cgp::extra::handler::{CanCompute, PipeHandlers, ReturnInput};
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

    check_components! {
        App {
            ComputerComponent: ((), u64),
        }
    }

    pub fn demo() {
        assert_eq!(App.compute(PhantomData::<()>, 5), 7); // 5 -> 6 -> 6 -> 7
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
