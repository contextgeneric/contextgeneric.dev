//! Code from `docs/reference/providers/monad/pipe_monadic.md` — `PipeMonadic`.
//!
//! Pins both Examples programs: a pipeline of fallible computers wired under `ErrMonadic`, and a
//! pipeline over nested `Result`s under a stacked monad, driven through `compute` and, through the
//! fallible bridge, `try_compute`.

/// ## Examples
pub mod examples {
    use cgp::extra::handler::CanCompute;
    use cgp::extra::monad::monadic::err::ErrMonadic;
    use cgp::extra::monad::providers::PipeMonadic;
    use cgp::prelude::*;

    #[cgp_computer]
    pub fn increment(value: u8) -> Result<u8, &'static str> {
        value.checked_add(1).ok_or("overflow")
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: PipeMonadic<ErrMonadic, Product![Increment, Increment, Increment]>,
        }
    }

    check_components! {
        App {
            ComputerComponent: ((), u8),
        }
    }

    pub fn demo() {
        assert_eq!(App.compute(PhantomData::<()>, 1), Ok(4));
        // 253 -> Ok(254) -> Ok(255) -> Err("overflow")
        assert_eq!(App.compute(PhantomData::<()>, 253), Err("overflow"));
    }

    #[test]
    fn test_demo() {
        demo();
    }

    /// The stacked-monad example: nested `Result`s under `OkMonadicTrans<ErrMonadic>`, and the same
    /// handlers under plain `OkMonadic` through `try_compute`.
    pub mod stacked {
        use cgp::core::error::ErrorTypeProviderComponent;
        use cgp::extra::monad::monadic::err::ErrMonadic;
        use cgp::extra::monad::monadic::ok::{OkMonadic, OkMonadicTrans};
        use cgp::extra::monad::providers::PipeMonadic;
        use cgp::prelude::*;

        #[cgp_computer]
        pub fn return_ok_ok(_value: u8) -> Result<Result<(), u8>, &'static str> {
            Ok(Ok(()))
        }

        #[cgp_computer]
        pub fn return_ok_err(value: u8) -> Result<Result<(), u8>, &'static str> {
            Ok(Err(value))
        }

        #[cgp_computer]
        pub fn return_err(_value: u8) -> Result<Result<(), u8>, &'static str> {
            Err("error")
        }

        pub struct App;

        delegate_components! {
            App {
                ErrorTypeProviderComponent: UseType<&'static str>,
            }
        }

        pub fn demo() {
            let code = PhantomData::<()>;

            // An inner `Ok` stops the pipeline; the last step never runs.
            assert_eq!(
                PipeMonadic::<
                    OkMonadicTrans<ErrMonadic>,
                    Product![ReturnOkErr, ReturnOkOk, ReturnOkErr],
                >::compute(&App, code, 1),
                Ok(Ok(())),
            );

            // An outer `Err` stops it too.
            assert_eq!(
                PipeMonadic::<
                    OkMonadicTrans<ErrMonadic>,
                    Product![ReturnErr, ReturnOkOk, ReturnOkErr],
                >::compute(&App, code, 1),
                Err("error"),
            );

            // Through the fallible bridge, plain `OkMonadic` behaves the same way.
            assert_eq!(
                PipeMonadic::<
                    OkMonadic,
                    Product![ReturnOkErr, ReturnOkOk, ReturnOkErr],
                >::try_compute(&App, code, 1),
                Ok(Ok(())),
            );
        }

        #[test]
        fn test_demo() {
            demo();
        }
    }
}
