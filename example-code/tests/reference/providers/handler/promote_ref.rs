//! Code from `docs/reference/providers/handler/promote_ref.md` — `PromoteRef`.
//!
//! Pins both directions the page shows: a by-value computer over `&u64` serves `ComputerRefComponent`,
//! and a `ComputerRef` over `String` serves `ComputerComponent` for a `Box<String>` input.

/// ## Examples
pub mod examples {
    use cgp::extra::handler::{CanCompute, CanComputeRef, ComputerRef, PromoteRef};
    use cgp::prelude::*;

    /// A by-value computer whose input is a reference.
    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, &u64> for DoubleRef {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: &u64) -> u64 {
            input * 2
        }
    }

    /// A by-reference computer over a `String`.
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
            ComputerRefComponent: PromoteRef<DoubleRef>,
            ComputerComponent: PromoteRef<StringLength>,
        }
    }

    check_components! {
        App {
            ComputerRefComponent: ((), u64),
            ComputerComponent: ((), Box<String>),
        }
    }

    pub fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.compute_ref(code, &21), 42);
        assert_eq!(App.compute(code, Box::new("hello".to_owned())), 5);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
