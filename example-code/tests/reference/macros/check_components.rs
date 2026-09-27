//! Code from `docs/reference/macros/check_components.md` — *`check_components!`*.
//!
//! The snippets the page rejects, and the errors it quotes, live under
//! `tests/compile_fail/reference/macros/`. The *Examples* section's first program is one of them,
//! since the page shows it failing.

/// ## Overview and Usage
///
/// The one-entry table both sections open with, against a `Person` that does carry the `name`
/// field. The greeter is the one *Examples* shows.
pub mod overview {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self, #[implicit] name: &str) -> String {
            format!("Hello, {name}!")
        }
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    delegate_components! {
        Person {
            GreeterComponent: GreetHello,
        }
    }

    check_components! {
        Person {
            GreeterComponent,
        }
    }

    // An invocation with no table, which the page says is accepted.
    check_components! {}
}

/// ### Components with generic parameters
///
/// A bare parameter, a tuple of two, the bracketed product, and a parameter carrying its own
/// generics. The page names the components and shapes without declaring them.
pub mod components_with_generic_parameters {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    #[cgp_component(Rotator)]
    pub trait CanRotate<Shape> {
        fn rotate(&self, shape: &Shape, degrees: f64) -> f64;
    }

    #[cgp_component(TransformCalculator)]
    pub trait CanTransform<Shape, Factor> {
        fn transform(&self, shape: &Shape, factor: Factor) -> f64;
    }

    pub struct Rectangle;
    pub struct Circle;

    #[cgp_impl(new AnyShape)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            1.0
        }
    }

    #[cgp_impl(AnyShape)]
    impl<Shape> Rotator<Shape> {
        fn rotate(&self, _shape: &Shape, degrees: f64) -> f64 {
            degrees
        }
    }

    #[cgp_impl(AnyShape)]
    impl<Shape> TransformCalculator<Shape, f64> {
        fn transform(&self, _shape: &Shape, factor: f64) -> f64 {
            factor
        }
    }

    pub struct MyApp;

    delegate_components! {
        MyApp {
            [
                AreaCalculatorComponent,
                RotatorComponent,
                TransformCalculatorComponent,
            ]: AnyShape,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent: Rectangle,                 // one parameter
            TransformCalculatorComponent: (Rectangle, f64),     // two, as a tuple
        }
    }

    check_components! {
        #[check_trait(CheckMyAppProduct)]
        MyApp {
            [AreaCalculatorComponent, RotatorComponent]: [Rectangle, Circle],
        }
    }

    check_components! {
        #[check_trait(CheckMyAppReference)]
        MyApp {
            AreaCalculatorComponent: <'a> &'a Rectangle,
        }
    }
}

/// ### Naming the check trait
///
/// The override, and the reference context that needs one because it has no name to derive.
pub mod naming_the_check_trait {
    use cgp::prelude::*;

    pub use super::overview::{GreeterComponent, Person};

    check_components! {
        #[check_trait(CheckPersonGreeting)]
        Person {
            GreeterComponent,
        }
    }

    check_components! {
        #[check_trait(CheckPersonReference)]
        <'a> &'a Person {
        }
    }
}

/// ### Checking each provider instead of the context
///
/// The per-layer check of a scaled-area stack. The page names `ScaledRectangle`, `RectangleArea`,
/// and `ScaledArea` without declaring them.
pub mod checking_each_provider_instead_of_the_context {
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

    #[cgp_impl(new ScaledArea<InnerCalculator>)]
    #[use_provider(InnerCalculator: AreaCalculator)]
    impl<InnerCalculator> AreaCalculator {
        fn area(&self, #[implicit] scale_factor: f64) -> f64 {
            InnerCalculator::area(self) * scale_factor * scale_factor
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
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    check_components! {
        #[check_trait(CheckScaledProviders)]
        #[check_providers(
            RectangleArea,
            ScaledArea<RectangleArea>,
        )]
        ScaledRectangle {
            AreaCalculatorComponent,
        }
    }

    /// The generic-table case the page warns about, checked on a concrete instantiation instead.
    #[derive(HasField)]
    pub struct Gen<T> {
        pub width: f64,
        pub height: f64,
        pub marker: core::marker::PhantomData<T>,
    }

    delegate_components! {
        <T> Gen<T> {
            AreaCalculatorComponent: RectangleArea,
        }
    }

    check_components! {
        #[check_trait(CheckGenProviders)]
        #[check_providers(RectangleArea)]
        Gen<u32> {
            AreaCalculatorComponent,
        }
    }
}

/// ## Examples
///
/// The second example: a generic component checked at two shapes. The page shows only the
/// component and the check; the context's wiring and the providers are declared here. The first
/// example fails by design and is a trybuild fixture.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    pub struct Circle {
        pub radius: f64,
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator<Rectangle> {
        fn area(&self, shape: &Rectangle) -> f64 {
            shape.width * shape.height
        }
    }

    #[cgp_impl(new CircleArea)]
    impl AreaCalculator<Circle> {
        fn area(&self, shape: &Circle) -> f64 {
            core::f64::consts::PI * shape.radius * shape.radius
        }
    }

    pub struct MyApp;

    delegate_components! {
        MyApp {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.Rectangle: RectangleArea,
            @AreaCalculatorComponent.Circle: CircleArea,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }
}

/// ## Under the hood
///
/// The table generics and `where` clause merged onto an impl, and the unsized parameter the
/// `?Sized` bound admits. The page names `FooComponent` and `ReferenceGetterComponent` without
/// declaring them.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_component(FooProvider)]
    pub trait CanFoo<T> {
        fn foo(&self, value: T) -> u8;
    }

    #[cgp_impl(new AnyFoo)]
    impl<T> FooProvider<T> {
        fn foo(&self, _value: T) -> u8 {
            0
        }
    }

    #[cgp_component(ReferenceGetter)]
    pub trait CanGetReference<'a, T: ?Sized + 'a> {
        fn get_reference(&self) -> Option<&'a T>;
    }

    #[cgp_impl(new NoReference)]
    impl<'a, T: ?Sized + 'a> ReferenceGetter<'a, T> {
        fn get_reference(&self) -> Option<&'a T> {
            None
        }
    }

    pub struct Context;

    delegate_components! {
        Context {
            FooProviderComponent: AnyFoo,
            ReferenceGetterComponent: NoReference,
        }
    }

    check_components! {
        <'a, I> Context where I: Clone {
            FooProviderComponent: &'a I,
        }
    }

    check_components! {
        #[check_trait(CheckUnsized)]
        Context {
            ReferenceGetterComponent: <'a> (Life<'a>, str),
        }
    }
}
