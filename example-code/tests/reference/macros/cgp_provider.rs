//! Code from `docs/reference/macros/cgp_provider.md` — *`#[cgp_provider]` & `#[cgp_new_provider]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`.

/// ## Usage
///
/// The component override. The component's marker is named `AreaComponent` through the `name:`
/// key, so the default `AreaCalculatorComponent` would name nothing and the override is required.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_component {
        name: AreaComponent,
        provider: AreaCalculator,
    }]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    pub struct UnitArea;

    #[cgp_provider(AreaComponent)]
    impl<Context> AreaCalculator<Context> for UnitArea {
        fn area(_context: &Context) -> f64 {
            1.0
        }
    }

    pub struct Square;

    delegate_components! {
        Square {
            AreaComponent: UnitArea,
        }
    }

    check_components! {
        Square {
            AreaComponent,
        }
    }

    #[test]
    fn the_overridden_component_wires() {
        assert_eq!(Square.area(), 1.0);
    }
}

/// ### The struct `#[cgp_new_provider]` declares
///
/// The unit and generic shapes, plus the two the page states only in prose: several parameters
/// share one `PhantomData` tuple, and a lifetime is lifted into `Life<'a>`. Checked with
/// `cargo cgp expand --item reference::macros::cgp_provider::the_struct_cgp_new_provider_declares`.
pub mod the_struct_cgp_new_provider_declares {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    #[cgp_component(Labeler)]
    pub trait CanLabel<Code> {
        fn label(&self, code: PhantomData<Code>) -> String;
    }

    #[cgp_new_provider]
    impl<Context, Code> Labeler<Context, Code> for FixedLabel {
        fn label(_context: &Context, _code: PhantomData<Code>) -> String {
            "fixed".to_owned()
        }
    }

    #[cgp_new_provider]
    impl<Context, Code, InCode> Labeler<Context, Code> for LabelFor<InCode> {
        fn label(_context: &Context, _code: PhantomData<Code>) -> String {
            core::any::type_name::<InCode>().to_owned()
        }
    }

    #[cgp_new_provider]
    impl<Context, Code, A, B> Labeler<Context, Code> for LabelPair<A, B> {
        fn label(_context: &Context, _code: PhantomData<Code>) -> String {
            format!("{}+{}", core::any::type_name::<A>(), core::any::type_name::<B>())
        }
    }

    #[cgp_new_provider]
    impl<'a, Context, Code> Labeler<Context, Code> for LabelWithLife<'a> {
        fn label(_context: &Context, _code: PhantomData<Code>) -> String {
            "life".to_owned()
        }
    }

    #[test]
    fn the_structs_have_the_stated_shapes() {
        let _: FixedLabel = FixedLabel;
        let _: LabelFor<u8> = LabelFor(PhantomData::<u8>);
        let _: LabelPair<u8, u16> = LabelPair(PhantomData::<(u8, u16)>);
        let _: LabelWithLife<'static> = LabelWithLife(PhantomData::<Life<'static>>);
    }
}

/// ## Examples
///
/// The three spellings of the same provider, each in its own module since each declares
/// `RectangleArea`, and each wired and checked on `Rectangle`.
pub mod examples {
    pub mod with_cgp_provider {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_auto_getter]
        pub trait HasDimensions {
            fn width(&self) -> &f64;
            fn height(&self) -> &f64;
        }

        pub struct RectangleArea;

        #[cgp_provider]
        impl<Context> AreaCalculator<Context> for RectangleArea
        where
            Context: HasDimensions,
        {
            fn area(context: &Context) -> f64 {
                context.width() * context.height()
            }
        }

        #[derive(HasField)]
        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        delegate_components! {
            Rectangle {
                AreaCalculatorComponent: RectangleArea,
            }
        }

        check_components! {
            Rectangle {
                AreaCalculatorComponent,
            }
        }

        #[test]
        fn the_provider_computes_the_area() {
            let rect = Rectangle {
                width: 2.0,
                height: 3.0,
            };
            assert_eq!(rect.area(), 6.0);
        }
    }

    pub mod with_cgp_new_provider {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_auto_getter]
        pub trait HasDimensions {
            fn width(&self) -> &f64;
            fn height(&self) -> &f64;
        }

        #[cgp_new_provider]
        impl<Context> AreaCalculator<Context> for RectangleArea
        where
            Context: HasDimensions,
        {
            fn area(context: &Context) -> f64 {
                context.width() * context.height()
            }
        }

        #[derive(HasField)]
        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        delegate_components! {
            Rectangle {
                AreaCalculatorComponent: RectangleArea,
            }
        }

        check_components! {
            Rectangle {
                AreaCalculatorComponent,
            }
        }
    }

    pub mod with_cgp_impl {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_auto_getter]
        pub trait HasDimensions {
            fn width(&self) -> &f64;
            fn height(&self) -> &f64;
        }

        #[cgp_impl(new RectangleArea)]
        #[uses(HasDimensions)]
        impl AreaCalculator {
            fn area(&self) -> f64 {
                self.width() * self.height()
            }
        }

        #[derive(HasField)]
        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        delegate_components! {
            Rectangle {
                AreaCalculatorComponent: RectangleArea,
            }
        }

        check_components! {
            Rectangle {
                AreaCalculatorComponent,
            }
        }
    }
}

/// ## Under the hood
///
/// The `ComputerRef` provider whose marker impl the page lists, a lifetime-carrying provider trait,
/// and the higher-order `Scaled<Inner>` whose inner bound gains its marker counterpart, both in the
/// `where` clause and inline. Checked with
/// `cargo cgp expand --item reference::macros::cgp_provider::under_the_hood`.
pub mod under_the_hood {
    pub mod computer_ref {
        use core::fmt::Display;
        use core::marker::PhantomData;

        use cgp::extra::handler::{ComputerRef, ComputerRefComponent};
        use cgp::prelude::*;

        pub struct FirstNameToString;

        #[cgp_provider]
        impl<Context, Code, Input> ComputerRef<Context, Code, Input> for FirstNameToString
        where
            Context: HasField<Symbol!("first_name"), Value: Display>,
        {
            type Output = String;

            fn compute_ref(context: &Context, _code: PhantomData<Code>, _input: &Input) -> String {
                context.get_field(PhantomData).to_string()
            }
        }
    }

    pub mod lifetime_provider_trait {
        use cgp::prelude::*;

        #[cgp_component(ReferenceGetter)]
        pub trait HasReference<'a, T: 'a> {
            fn reference(&self) -> &'a T;
        }

        #[cgp_new_provider]
        impl<'a, Context, T: 'a> ReferenceGetter<'a, Context, T> for Unreachable {
            fn reference(_context: &Context) -> &'a T {
                unimplemented!("only the marker impl is inspected")
            }
        }
    }

    pub mod higher_order {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_new_provider]
        impl<Context, Inner> AreaCalculator<Context> for Scaled<Inner>
        where
            Inner: AreaCalculator<Context>,
        {
            fn area(context: &Context) -> f64 {
                Inner::area(context) * 2.0
            }
        }

        #[cgp_new_provider]
        impl<Context, Inner: AreaCalculator<Context>> AreaCalculator<Context> for Doubled<Inner> {
            fn area(context: &Context) -> f64 {
                Inner::area(context) * 2.0
            }
        }

        #[cgp_impl(new UnitArea)]
        impl AreaCalculator {
            fn area(&self) -> f64 {
                1.0
            }
        }

        pub struct App;

        delegate_components! {
            App {
                AreaCalculatorComponent: Scaled<Doubled<UnitArea>>,
            }
        }

        check_components! {
            App {
                AreaCalculatorComponent,
            }
        }

        #[test]
        fn the_stack_computes() {
            assert_eq!(App.area(), 4.0);
        }
    }
}

/// ## Common Mistakes
///
/// The disambiguated associated const, which is the compiling half of the `E0034` entry.
/// `RateLimiter` and `App` are declared here from the page's inline names.
pub mod common_mistakes {
    use cgp::prelude::*;

    #[cgp_component(RateLimiter)]
    pub trait CanRateLimit {
        const LIMIT: u64;

        fn allowed(&self) -> bool;
    }

    pub struct AllowUnderLimit;

    #[cgp_provider]
    impl<Context> RateLimiter<Context> for AllowUnderLimit {
        const LIMIT: u64 = 100;

        fn allowed(_context: &Context) -> bool {
            <Self as RateLimiter<Context>>::LIMIT > 0
        }
    }

    pub struct App;

    delegate_components! {
        App {
            RateLimiterComponent: AllowUnderLimit,
        }
    }

    check_components! {
        App {
            RateLimiterComponent,
        }
    }

    #[test]
    fn the_const_reads_through_both_traits() {
        assert!(App.allowed());
        assert_eq!(<App as CanRateLimit>::LIMIT, 100);
    }
}
