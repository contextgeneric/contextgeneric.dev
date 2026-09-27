//! Code from `docs/reference/macros/cgp_computer.md` — *`#[cgp_computer]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`.

/// ## Usage
///
/// The default and the explicit provider name, in separate modules since both define `add`.
pub mod usage {
    pub mod default_name {
        use core::marker::PhantomData;

        use cgp::extra::handler::Computer;
        use cgp::prelude::*;

        #[cgp_computer]
        fn add(a: u64, b: u64) -> u64 {
            a + b
        }

        #[test]
        fn the_provider_is_named_after_the_function() {
            assert_eq!(Add::compute(&(), PhantomData::<()>, (1, 2)), 3);
        }
    }

    /// A destructuring parameter and an omitted return type, which the page states in prose.
    pub mod patterns_and_unit_output {
        use core::marker::PhantomData;

        use cgp::extra::handler::Computer;
        use cgp::prelude::*;

        #[cgp_computer]
        fn sum_pair((a, b): (u64, u64)) -> u64 {
            a + b
        }

        #[cgp_computer]
        fn log_value(_value: u64) {}

        #[test]
        fn only_the_types_are_read() {
            assert_eq!(SumPair::compute(&(), PhantomData::<()>, (1, 2)), 3);
            let () = LogValue::compute(&(), PhantomData::<()>, 5);
        }
    }

    pub mod explicit_name {
        use core::marker::PhantomData;

        use cgp::extra::handler::Computer;
        use cgp::prelude::*;

        #[cgp_computer(MyAdder)]
        fn add(a: u64, b: u64) -> u64 {
            a + b
        }

        #[test]
        fn the_argument_names_the_provider() {
            assert_eq!(MyAdder::compute(&(), PhantomData::<()>, (1, 2)), 3);
        }
    }
}

/// ### What the two axes select
///
/// One function per row of the table, each checked through the member its bundle makes fallible
/// or async.
pub mod what_the_two_axes_select {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{AsyncComputer, Computer, Handler, TryComputer};
    use cgp::prelude::*;

    #[cgp_computer]
    fn double(value: u64) -> u64 {
        value * 2
    }

    #[cgp_computer]
    fn halve(value: u64) -> Result<u64, String> {
        if value % 2 == 0 {
            Ok(value / 2)
        } else {
            Err("odd".to_owned())
        }
    }

    #[cgp_computer]
    async fn triple(value: u64) -> u64 {
        value * 3
    }

    #[cgp_computer]
    async fn third(value: u64) -> Result<u64, String> {
        if value % 3 == 0 {
            Ok(value / 3)
        } else {
            Err("indivisible".to_owned())
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
        }
    }

    #[test]
    fn each_row_picks_its_base_and_bundle() {
        let code = PhantomData::<()>;
        assert_eq!(Double::compute(&App, code, 4), 8);
        assert_eq!(Double::try_compute(&App, code, 4), Ok(8));
        assert_eq!(Halve::compute(&App, code, 3), Err("odd".to_owned()));
        assert_eq!(Halve::try_compute(&App, code, 3), Err("odd".to_owned()));
        assert_eq!(
            futures::executor::block_on(Triple::compute_async(&App, code, 2)),
            6
        );
        assert_eq!(
            futures::executor::block_on(Third::handle(&App, code, 4)),
            Err("indivisible".to_owned())
        );
    }
}

/// ## Examples
///
/// The page's `add` called through all four shapes, the fallible variant, and the generic one.
pub mod examples {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::{AsyncComputer, Computer, Handler, TryComputer};
    use cgp::prelude::*;

    #[cgp_computer]
    fn add(a: u64, b: u64) -> u64 {
        a + b
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
        }
    }

    pub fn demo() {
        // All four are answered by the single `add` definition.
        assert_eq!(Add::compute(&App, PhantomData::<()>, (1, 2)), 3);
        assert_eq!(Add::try_compute(&App, PhantomData::<()>, (1, 2)), Ok(3));

        // These futures resolve to 3 and Ok(3).
        let _future = Add::compute_async(&App, PhantomData::<()>, (1, 2));
        let _future = Add::handle(&App, PhantomData::<()>, (1, 2));
    }

    #[test]
    fn one_function_answers_every_shape() {
        demo();
        assert_eq!(Add::compute(&App, PhantomData::<()>, (1, 2)), 3);
        assert_eq!(Add::try_compute(&App, PhantomData::<()>, (1, 2)), Ok(3));
        assert_eq!(
            futures::executor::block_on(Add::compute_async(&App, PhantomData::<()>, (1, 2))),
            3
        );
        assert_eq!(
            futures::executor::block_on(Add::handle(&App, PhantomData::<()>, (1, 2))),
            Ok(3)
        );
    }

    #[cgp_computer]
    fn add_with_error(a: u64, b: u64) -> Result<u64, String> {
        a.checked_add(b).ok_or_else(|| "Overflow".to_string())
    }

    #[test]
    fn the_fallible_forms_propagate_the_error() {
        assert_eq!(
            AddWithError::try_compute(&App, PhantomData::<()>, (u64::MAX, 1)),
            Err("Overflow".to_owned())
        );
        assert_eq!(
            futures::executor::block_on(AddWithError::handle(
                &App,
                PhantomData::<()>,
                (u64::MAX, 1)
            )),
            Err("Overflow".to_owned())
        );
    }

    #[cgp_computer]
    pub fn add_generic<T: core::ops::Add<Output = T>>(a: T, b: T) -> T {
        a + b
    }

    #[test]
    fn the_generic_provider_covers_each_type() {
        assert_eq!(AddGeneric::compute(&App, PhantomData::<()>, (1u8, 2u8)), 3);
        assert_eq!(AddGeneric::compute(&App, PhantomData::<()>, (1.5f64, 2.0f64)), 3.5);
    }
}

/// ## Under the hood
///
/// The `add` provider whose impl and wiring the page lists, checked with
/// `cargo cgp expand --item reference::macros::cgp_computer::under_the_hood`, and the reference
/// parameter the page describes in prose.
pub mod under_the_hood {
    use core::fmt::Display;
    use core::marker::PhantomData;

    use cgp::extra::handler::{Computer, ComputerRef};
    use cgp::prelude::*;

    #[cgp_computer]
    fn add(a: u64, b: u64) -> u64 {
        a + b
    }

    #[cgp_computer]
    fn to_string_ref<V: Display>(value: &V) -> String {
        value.to_string()
    }

    #[test]
    fn the_reference_input_serves_the_ref_component() {
        assert_eq!(ToStringRef::compute(&(), PhantomData::<()>, &7), "7");
        assert_eq!(ToStringRef::compute_ref(&(), PhantomData::<()>, &7), "7");
        assert_eq!(Add::compute(&(), PhantomData::<()>, (1, 2)), 3);
    }
}

/// ## Common Mistakes
///
/// The compiling half of the `Result` detection entry: a fully qualified `Result` is not
/// recognised, so the value bundle is wired and `try_compute` wraps the whole result in `Ok`.
pub mod common_mistakes {
    use core::marker::PhantomData;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::handler::TryComputer;
    use cgp::prelude::*;

    #[cgp_computer]
    fn checked_halve(value: u64) -> core::result::Result<u64, String> {
        if value % 2 == 0 {
            Ok(value / 2)
        } else {
            Err("odd".to_owned())
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
        }
    }

    #[test]
    fn a_qualified_result_is_treated_as_a_value() {
        assert_eq!(
            CheckedHalve::try_compute(&App, PhantomData::<()>, 3),
            Ok(Err("odd".to_owned()))
        );
    }
}
