//! Code from `docs/reference/components/handler/computer_ref.md` — `ComputerRef`.
//!
//! Pins the `StringLength` provider from the page's Examples, wired for the borrowed input and, through
//! `PromoteRef`, for an owned input that dereferences to it, and the reference-parameter function
//! Common Mistakes gives as the fix. The owned-input computer that cannot answer the borrowed slot is a
//! trybuild fixture.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::{CanCompute, CanComputeRef, ComputerRef, PromoteRef};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> ComputerRef<Context, Code, String> for StringLength {
        type Output = usize;

        fn compute_ref(_context: &Context, _code: PhantomData<Code>, input: &String) -> usize {
            input.len()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerRefComponent: StringLength,
            ComputerComponent: PromoteRef<StringLength>,
        }
    }

    check_components! {
        App {
            ComputerRefComponent: ((), String),
            ComputerComponent: ((), Box<String>),
        }
    }

    pub fn demo() {
        let name = "hello".to_owned();

        assert_eq!(App.compute_ref(PhantomData::<()>, &name), 5);
        assert_eq!(App.compute(PhantomData::<()>, Box::new(name)), 5);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Common Mistakes
///
/// The fix the page gives: a `#[cgp_computer]` function over a reference answers `compute_ref`. The
/// owned-input computer behind `PromoteRef` is
/// `tests/compile_fail/reference/components/computer_ref_common_mistakes_owned_input_computer.rs`.
pub mod common_mistakes {
    use core::marker::PhantomData;

    use cgp::extra::handler::CanComputeRef;
    use cgp::prelude::*;

    #[cgp_computer]
    fn double(value: &u64) -> u64 {
        value * 2
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerRefComponent: Double,
        }
    }

    check_components! {
        App {
            ComputerRefComponent: ((), u64),
        }
    }

    #[test]
    fn a_reference_parameter_answers_compute_ref() {
        assert_eq!(App.compute_ref(PhantomData::<()>, &21), 42);
    }
}
