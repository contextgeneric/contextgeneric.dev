//! Code from `docs/reference/providers/handler/promote_ref.md` — `PromoteRef`.

/// ## Usage and Examples
///
/// `PromoteRef` bridges by-value and by-reference handlers. To fill a by-reference slot such as
/// `ComputerRefComponent`, the inner provider is a by-value provider whose input is a reference; the
/// bridge calls it on the borrowed input.
pub mod bridging_value_and_reference {
    use core::marker::PhantomData;

    use cgp::extra::handler::{Computer, ComputerRefComponent, PromoteRef};
    use cgp::prelude::*;

    /// A by-value computer whose input is a reference: doubles the value behind it.
    #[cgp_new_provider]
    impl<Context, Code> Computer<Context, Code, &u64> for DoubleRef {
        type Output = u64;

        fn compute(_context: &Context, _code: PhantomData<Code>, input: &u64) -> u64 {
            input * 2
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerRefComponent: PromoteRef<DoubleRef>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerRefComponent: ((), u64),
            }
        }
    }

    #[test]
    fn the_by_reference_slot_is_served_from_the_inner_provider() {
        use cgp::extra::handler::CanComputeRef;

        let app = App;
        assert_eq!(app.compute_ref(PhantomData::<()>, &5u64), 10);
    }
}
