//! Code from `docs/reference/macros/cgp_producer.md` — *`#[cgp_producer]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`.

/// ## Usage
///
/// The default and explicit provider names, each in its own module since both define
/// `magic_number`, and the default for a raw function name.
pub mod usage {
    pub mod default_name {
        use core::marker::PhantomData;

        use cgp::extra::handler::Producer;
        use cgp::prelude::*;

        #[cgp_producer]
        fn magic_number() -> u64 {
            42
        }

        #[test]
        fn the_provider_is_named_after_the_function() {
            assert_eq!(MagicNumber::produce(&(), PhantomData::<()>), 42);
        }
    }

    /// An omitted return type, which the page states in prose.
    pub mod unit_output {
        use core::marker::PhantomData;

        use cgp::extra::handler::Producer;
        use cgp::prelude::*;

        #[cgp_producer]
        fn nothing() {}

        #[test]
        fn the_output_is_unit() {
            let () = Nothing::produce(&(), PhantomData::<()>);
        }
    }

    pub mod explicit_name {
        use core::marker::PhantomData;

        use cgp::extra::handler::Producer;
        use cgp::prelude::*;

        #[cgp_producer(TheAnswer)]
        fn magic_number() -> u64 {
            42
        }

        #[test]
        fn the_argument_names_the_provider() {
            assert_eq!(TheAnswer::produce(&(), PhantomData::<()>), 42);
        }
    }

    /// A raw function name, which the page says loses its `r#`: `r#loop` names `Loop`.
    pub mod raw_name {
        use core::marker::PhantomData;

        use cgp::extra::handler::Producer;
        use cgp::prelude::*;

        #[cgp_producer]
        fn r#loop() -> u64 {
            42
        }

        #[test]
        fn the_raw_prefix_is_dropped() {
            assert_eq!(Loop::produce(&(), PhantomData::<()>), 42);
        }
    }
}

/// ## Examples
///
/// The page's producer read through every shape, and the pipeline it seeds. The page names
/// `Double` without declaring it; it is a `#[cgp_computer]` here.
pub mod examples {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{Computer, Handler, Producer, TryComputer};
    use cgp::prelude::*;

    #[cgp_producer]
    pub fn magic_number() -> u64 {
        42
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
        }
    }

    pub fn demo() {
        // The single `magic_number` definition answers every shape, each yielding 42.
        assert_eq!(MagicNumber::produce(&App, PhantomData::<()>), 42);
        assert_eq!(MagicNumber::compute(&App, PhantomData::<()>, ()), 42);
        assert_eq!(MagicNumber::try_compute(&App, PhantomData::<()>, "ignored"), Ok(42));

        // The future resolves to Ok(42).
        let _future = MagicNumber::handle(&App, PhantomData::<()>, ());
    }

    #[test]
    fn every_shape_yields_the_value() {
        demo();
        assert_eq!(
            futures::executor::block_on(MagicNumber::handle(&App, PhantomData::<()>, ())),
            Ok(42)
        );
    }

    pub mod pipeline {
        use core::marker::PhantomData;

        use cgp::extra::handler::{CanCompute, ComputerComponent, PipeHandlers};
        use cgp::prelude::*;

        use super::MagicNumber;

        #[cgp_computer]
        fn double(value: u64) -> u64 {
            value * 2
        }

        pub struct App;

        delegate_components! {
            App {
                ComputerComponent:
                    PipeHandlers<Product![
                        MagicNumber,
                        Double,
                    ]>,
            }
        }

        check_components! {
            App {
                ComputerComponent: ((), ()),
            }
        }

        #[test]
        fn the_producer_seeds_the_pipeline() {
            assert_eq!(App.compute(PhantomData::<()>, ()), 84);
        }
    }
}

/// ## Common Mistakes
///
/// A `Result` output is not interpreted: `produce` returns it as a value, and `try_compute` wraps
/// it in `Ok`, which is the output the page quotes.
pub mod common_mistakes {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{Producer, TryComputer};
    use cgp::prelude::*;

    #[cgp_producer]
    fn failing() -> Result<u64, String> {
        Err("nope".to_owned())
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
        }
    }

    #[test]
    fn the_result_is_wrapped_again() {
        assert_eq!(
            Failing::produce(&App, PhantomData::<()>),
            Err("nope".to_owned())
        );
        assert_eq!(
            Failing::try_compute(&App, PhantomData::<()>, ()),
            Ok(Err("nope".to_owned()))
        );
    }
}
