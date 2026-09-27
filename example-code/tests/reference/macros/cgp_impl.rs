//! Code from `docs/reference/macros/cgp_impl.md` — *`#[cgp_impl]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`, one fixture
//! per *Common Mistakes* entry that fails to compile.

/// ## Overview
///
/// The hand-written provider impl and the `#[cgp_impl]` block that replaces it. The page names
/// `HasDimensions` without declaring it; it is an auto getter over two `f64` fields here, and each
/// form gets its own module since both declare `RectangleArea`.
pub mod overview {
    pub mod hand_written {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_auto_getter]
        pub trait HasDimensions {
            fn width(&self) -> f64;
            fn height(&self) -> f64;
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
    }

    pub mod with_cgp_impl {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_auto_getter]
        pub trait HasDimensions {
            fn width(&self) -> f64;
            fn height(&self) -> f64;
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

        #[test]
        fn the_provider_reads_the_dimensions() {
            let rect = Rectangle {
                width: 2.0,
                height: 3.0,
            };
            assert_eq!(rect.area(), 6.0);
        }
    }
}

/// ## Usage
///
/// The generic higher-order provider, wired over a base calculator the page names only in prose.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
        }
    }

    #[cgp_impl(new ScaledAreaCalculator<InnerCalculator>)]
    #[use_provider(InnerCalculator: AreaCalculator)]
    impl<InnerCalculator> AreaCalculator {
        fn area(&self, #[implicit] scale_factor: f64) -> f64 {
            let base_area = InnerCalculator::area(self);
            base_area * scale_factor * scale_factor
        }
    }

    #[derive(HasField)]
    pub struct ScaledRectangle {
        pub width: f64,
        pub height: f64,
        pub scale_factor: f64,
    }

    delegate_components! {
        ScaledRectangle {
            AreaCalculatorComponent: ScaledAreaCalculator<RectangleArea>,
        }
    }

    check_components! {
        ScaledRectangle {
            AreaCalculatorComponent,
        }
    }

    #[test]
    fn the_wrapper_scales_the_inner_area() {
        let rect = ScaledRectangle {
            width: 3.0,
            height: 4.0,
            scale_factor: 2.0,
        };
        assert_eq!(rect.area(), 48.0);
    }
}

/// ### Declaring the provider struct separately
///
/// One provider implementing two components, with `new` on one block only, and a higher-order
/// provider whose struct is declared by hand so its inner parameter can default to `UseContext`.
/// The page does not declare `PerimeterCalculator`; it is a second component here.
pub mod declaring_the_provider_struct_separately {
    pub mod one_struct_two_components {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_component(PerimeterCalculator)]
        pub trait CanCalculatePerimeter {
            fn perimeter(&self) -> f64;
        }

        #[cgp_impl(new RectangleGeometry)]
        impl AreaCalculator {
            fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
                width * height
            }
        }

        #[cgp_impl(RectangleGeometry)]
        impl PerimeterCalculator {
            fn perimeter(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
                2.0 * (width + height)
            }
        }

        #[derive(HasField)]
        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        delegate_components! {
            Rectangle {
                [
                    AreaCalculatorComponent,
                    PerimeterCalculatorComponent,
                ]: RectangleGeometry,
            }
        }

        check_components! {
            Rectangle {
                [
                    AreaCalculatorComponent,
                    PerimeterCalculatorComponent,
                ],
            }
        }

        #[test]
        fn one_provider_serves_both_components() {
            let rect = Rectangle {
                width: 2.0,
                height: 3.0,
            };
            assert_eq!(rect.area(), 6.0);
            assert_eq!(rect.perimeter(), 10.0);
        }
    }

    pub mod defaulted_inner_provider {
        use core::marker::PhantomData;

        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        pub struct ScaledAreaCalculator<InnerCalculator = UseContext>(PhantomData<InnerCalculator>);

        #[cgp_impl(ScaledAreaCalculator<InnerCalculator>)]
        #[use_provider(InnerCalculator: AreaCalculator)]
        impl<InnerCalculator> AreaCalculator {
            fn area(&self, #[implicit] scale_factor: f64) -> f64 {
                let base_area = InnerCalculator::area(self);
                base_area * scale_factor * scale_factor
            }
        }

        #[cgp_impl(new RectangleArea)]
        impl AreaCalculator {
            fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
                width * height
            }
        }

        #[derive(HasField)]
        pub struct ScaledRectangle {
            pub width: f64,
            pub height: f64,
            pub scale_factor: f64,
        }

        delegate_components! {
            ScaledRectangle {
                AreaCalculatorComponent: ScaledAreaCalculator<RectangleArea>,
            }
        }

        check_components! {
            ScaledRectangle {
                AreaCalculatorComponent,
            }
        }

        /// The default applies where the inner provider is left out.
        pub type DefaultScaled = ScaledAreaCalculator;

        #[test]
        fn the_hand_declared_struct_wires() {
            let rect = ScaledRectangle {
                width: 3.0,
                height: 4.0,
                scale_factor: 2.0,
            };
            assert_eq!(rect.area(), 48.0);
            let _: DefaultScaled = ScaledAreaCalculator(PhantomData::<UseContext>);
        }
    }
}

/// ### Companion attributes
///
/// The comma-separated `#[uses(HasName, CanRaiseError<String>)]` list the page recommends, on a
/// provider the page does not show. `CanGreet` and its context are declared here.
pub mod companion_attributes {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_component(Greeter)]
    #[use_type(HasErrorType.Error)]
    pub trait CanGreet {
        fn greet(&self) -> Result<String, Error>;
    }

    #[cgp_impl(new GreetNonEmpty)]
    #[uses(HasName, CanRaiseError<String>)]
    #[use_type(HasErrorType.Error)]
    impl Greeter {
        fn greet(&self) -> Result<String, Error> {
            if self.name().is_empty() {
                return Err(Self::raise_error("empty name".to_owned()));
            }
            Ok(format!("Hello, {}!", self.name()))
        }
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    delegate_components! {
        Person {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<String>,
            @ErrorRaiserComponent.String: RaiseFrom,
            GreeterComponent: GreetNonEmpty,
        }
    }

    check_components! {
        Person {
            GreeterComponent,
        }
    }

    #[test]
    fn both_dependencies_resolve() {
        let person = Person {
            name: "World".to_owned(),
        };
        assert_eq!(person.greet(), Ok("Hello, World!".to_owned()));
    }
}

/// ### Implementing the consumer trait directly
///
/// The `#[cgp_impl(Self)]` block forwarding to a provider. `Rectangle` does not wire
/// `AreaCalculatorComponent`, since the direct impl would overlap the consumer blanket impl.
pub mod implementing_the_consumer_trait_directly {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_impl(Self)]
    #[use_provider(RectangleArea: AreaCalculator)]
    impl CanCalculateArea for Rectangle {
        fn area(&self) -> f64 {
            RectangleArea::area(self)
        }
    }

    /// The `Self` form still lowers `#[implicit]` arguments, since the companion attributes run
    /// before the macro chooses between the two forms.
    #[cgp_component(PerimeterCalculator)]
    pub trait CanCalculatePerimeter {
        fn perimeter(&self) -> f64;
    }

    #[cgp_impl(Self)]
    impl CanCalculatePerimeter for Rectangle {
        fn perimeter(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            2.0 * (width + height)
        }
    }

    #[test]
    fn the_direct_impls_apply() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.area(), 6.0);
        assert_eq!(rect.perimeter(), 10.0);
    }
}

/// ## Examples
///
/// The page's example, with the `check_components!` block it shows.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
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

    pub fn print_area(rect: &Rectangle) {
        println!("area = {}", rect.area());
    }

    #[test]
    fn the_wired_area_is_computed() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        print_area(&rect);
        assert_eq!(rect.area(), 6.0);
    }
}

/// ## Under the hood
///
/// The inputs whose expansions the page lists, checked with
/// `cargo cgp expand --test reference_tests --item reference::macros::cgp_impl::under_the_hood`.
/// The page does not declare `FooProvider`; its component is declared here.
pub mod under_the_hood {
    pub mod explicit_context {
        use cgp::prelude::*;

        #[cgp_component(FooProvider)]
        pub trait CanFoo {
            fn foo(&self, value: u32) -> String;
        }

        #[cgp_impl(new ValueToString)]
        impl<Context> FooProvider for Context {
            fn foo(&self, value: u32) -> String {
                value.to_string()
            }
        }

        pub struct App;

        delegate_components! {
            App {
                FooProviderComponent: ValueToString,
            }
        }

        check_components! {
            App {
                FooProviderComponent,
            }
        }

        #[test]
        fn the_provider_formats_the_value() {
            assert_eq!(App.foo(7), "7");
        }
    }

    /// A concrete context named in the header, whose receiver becomes `__rectangle__`.
    pub mod concrete_context {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        #[cgp_impl(new RectangleOnlyArea)]
        impl AreaCalculator for Rectangle {
            fn area(&self) -> f64 {
                self.width * self.height
            }
        }

        delegate_components! {
            Rectangle {
                AreaCalculatorComponent: RectangleOnlyArea,
            }
        }

        check_components! {
            Rectangle {
                AreaCalculatorComponent,
            }
        }
    }

    /// A provider supplying its own `type Output`, which the rewrite leaves as `Self::Output`.
    pub mod local_associated_type {
        use cgp::prelude::*;

        #[cgp_component(OutputProducer)]
        pub trait CanProduceOutput {
            type Output;

            fn produce(&self) -> Self::Output;
        }

        #[cgp_impl(new ProduceLabel)]
        impl OutputProducer {
            type Output = String;

            fn produce(&self) -> Self::Output {
                "label".to_owned()
            }
        }

        pub struct App;

        delegate_components! {
            App {
                OutputProducerComponent: ProduceLabel,
            }
        }

        check_components! {
            App {
                OutputProducerComponent,
            }
        }

        #[test]
        fn the_local_type_resolves() {
            assert_eq!(App.produce(), "label");
        }
    }
}

/// ## Common Mistakes
///
/// The compiling halves of the entries: the qualified path that names a provider's own const, and
/// the `Self` form with `new` and a component override, which the macro ignores. `RateLimiter` and
/// its context are declared here from the page's inline names.
pub mod common_mistakes {
    pub mod provider_const {
        use cgp::prelude::*;

        #[cgp_component(RateLimiter)]
        pub trait CanRateLimit {
            const LIMIT: u64;

            fn allows(&self, count: u64) -> bool;
        }

        #[cgp_impl(new AllowUnderLimit)]
        impl RateLimiter {
            const LIMIT: u64 = 100;

            fn allows(&self, count: u64) -> bool {
                count < <AllowUnderLimit as RateLimiter<Self>>::LIMIT
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
        fn the_qualified_const_resolves() {
            assert!(App.allows(99));
            assert!(!App.allows(100));
            assert_eq!(<App as CanRateLimit>::LIMIT, 100);
        }
    }

    pub mod self_form_ignores_new_and_override {
        use cgp::prelude::*;

        #[cgp_component(AreaCalculator)]
        pub trait CanCalculateArea {
            fn area(&self) -> f64;
        }

        #[cgp_component(PerimeterCalculator)]
        pub trait CanCalculatePerimeter {
            fn perimeter(&self) -> f64;
        }

        pub struct Square;

        #[cgp_impl(new Self)]
        impl CanCalculateArea for Square {
            fn area(&self) -> f64 {
                1.0
            }
        }

        #[cgp_impl(Self: SomeComponent)]
        impl CanCalculatePerimeter for Square {
            fn perimeter(&self) -> f64 {
                4.0
            }
        }

        #[test]
        fn both_are_plain_direct_impls() {
            assert_eq!(Square.area(), 1.0);
            assert_eq!(Square.perimeter(), 4.0);
        }
    }
}
