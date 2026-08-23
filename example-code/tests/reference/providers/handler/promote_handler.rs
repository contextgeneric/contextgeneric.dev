//! Code from `docs/reference/providers/handler/promote_handler.md` — `PromoteHandler`.

/// ## Usage and Examples
///
/// `PromoteHandler` fills the family from the most general base, a `Handler`. It answers
/// `HandlerComponent` and `HandlerRefComponent`, so those are routed to it; a base computer returning
/// a `Result` supplies the fallible async behavior.
pub mod filling_the_family_from_a_handler {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{HandlerComponent, PromoteHandler};
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

            HandlerComponent: PromoteHandler<CheckedAdd>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                HandlerComponent: ((), (u64, u64)),
            }
        }
    }

    #[test]
    fn the_handler_base_answers_the_family() {
        use cgp::extra::handler::CanHandle;
        use futures::executor::block_on;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(block_on(app.handle(code, (1u64, 2u64))), Ok(3));
        assert_eq!(block_on(app.handle(code, (u64::MAX, 1u64))), Err("overflow".to_owned()));
    }
}
