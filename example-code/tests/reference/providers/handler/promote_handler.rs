//! Code from `docs/reference/providers/handler/promote_handler.md` — `PromoteHandler`.
//!
//! Pins the Examples program: an async `#[cgp_computer]` function returning `Result` is the base, an
//! `AsyncComputer` whose output is a `Result`, and the bundle answers `HandlerComponent` from it
//! through `TryPromote`.

/// ## Examples
pub mod examples {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::CanHandle;
    use cgp::prelude::*;

    #[cgp_computer]
    pub async fn checked_double(value: u64) -> Result<u64, String> {
        value.checked_mul(2).ok_or_else(|| "overflow".to_owned())
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            HandlerComponent: PromoteHandler<CheckedDouble>,
        }
    }

    check_components! {
        App {
            HandlerComponent: ((), u64),
        }
    }

    pub async fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(App.handle(code, 21).await, Ok(42));
        assert_eq!(App.handle(code, u64::MAX).await, Err("overflow".to_owned()));
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}
