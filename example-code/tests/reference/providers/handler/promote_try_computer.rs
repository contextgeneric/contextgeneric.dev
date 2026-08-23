//! Code from `docs/reference/providers/handler/promote_try_computer.md` — `PromoteTryComputer`.

/// ## Usage and Examples
///
/// `PromoteTryComputer` fills the family from a synchronous fallible base. It answers
/// `TryComputerComponent` and the async members, so those are routed to it; a base computer returning
/// a `Result` supplies the fallible behavior.
pub mod filling_the_family_from_a_try_computer {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{HandlerComponent, PromoteTryComputer, TryComputerComponent};
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn checked_add(a: u64, b: u64) -> Result<u64, String> {
        a.checked_add(b).ok_or_else(|| "overflow".to_owned())
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,

            [TryComputerComponent, HandlerComponent]: PromoteTryComputer<CheckedAdd>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                TryComputerComponent: ((), (u64, u64)),
                HandlerComponent: ((), (u64, u64)),
            }
        }
    }

    #[test]
    fn the_fallible_base_answers_the_family() {
        use cgp::extra::handler::{CanHandle, CanTryCompute};
        use futures::executor::block_on;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(app.try_compute(code, (1u64, 2u64)), Ok(3));
        assert_eq!(app.try_compute(code, (u64::MAX, 1u64)), Err("overflow".to_owned()));
        assert_eq!(block_on(app.handle(code, (1u64, 2u64))), Ok(3));
    }
}
