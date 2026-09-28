//! Code from `docs/reference/components/handler/handler_ref.md` — `HandlerRef`.
//!
//! Pins the `NonEmptyLength` provider from the page's Examples, an async, fallible computation over a
//! borrowed input, wired onto a context that supplies its error type and raiser.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::extra::handler::{CanHandleRef, HandlerRef};
    use cgp::prelude::*;

    #[cgp_impl(new NonEmptyLength)]
    #[uses(CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl<Code> HandlerRef<Code, String> {
        type Output = usize;

        async fn handle_ref(
            &self,
            _code: PhantomData<Code>,
            input: &String,
        ) -> Result<Self::Output, Error> {
            if input.is_empty() {
                return Err(Self::raise_error("empty request".to_owned()));
            }

            Ok(input.len())
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            ErrorRaiserComponent: RaiseFrom,
            HandlerRefComponent: NonEmptyLength,
        }
    }

    check_components! {
        App {
            HandlerRefComponent: ((), String),
        }
    }

    pub async fn demo() {
        let request = "GET /".to_owned();

        assert_eq!(App.handle_ref(PhantomData::<()>, &request).await, Ok(5));
        assert_eq!(
            App.handle_ref(PhantomData::<()>, &String::new()).await,
            Err("empty request".to_owned())
        );
    }

    #[test]
    fn test_demo() {
        futures::executor::block_on(demo());
    }
}
