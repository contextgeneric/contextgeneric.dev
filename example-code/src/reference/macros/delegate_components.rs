//! Code from `docs/reference/macros/delegate_components.md` — *`delegate_components!`*.

/// ## Overview
///
/// The page opens on the single-entry table. The component, the provider, and the context it wires
/// are shown further down the page under *Examples*; they are declared here so the opening snippet
/// has something to name.
pub mod what_its_for {
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
}

/// ## Defining the target at the same time
///
/// The aggregate-provider bundle. `PerimeterCalculator` and its provider are named by the page
/// without being introduced, so they are declared here.
///
/// No `check_components!` block appears: the page's own point is that a context-side check on a
/// bundle asks the wrong question. The wiring is verified through `MyApp` below instead.
pub mod defining_the_target_at_the_same_time {
    use cgp::prelude::*;

    use super::what_its_for::{AreaCalculatorComponent, RectangleArea};

    #[cgp_component(PerimeterCalculator)]
    pub trait CanCalculatePerimeter {
        fn perimeter(&self) -> f64;
    }

    #[cgp_impl(new RectanglePerimeter)]
    impl PerimeterCalculator {
        fn perimeter(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            2.0 * (width + height)
        }
    }

    delegate_components! {
        new GeometryComponents {
            AreaCalculatorComponent: RectangleArea,
            PerimeterCalculatorComponent: RectanglePerimeter,
        }
    }

    /// A real context delegating to the bundle, which is how a bundle is actually verified.
    #[derive(HasField)]
    pub struct MyApp {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        MyApp {
            [AreaCalculatorComponent, PerimeterCalculatorComponent]: GeometryComponents,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent,
            PerimeterCalculatorComponent,
        }
    }
}

/// ## One table for a family of types
///
/// The page shows only the `delegate_components!` block; the generic context it wires is declared
/// here so the table has a target.
pub mod one_table_for_a_family_of_types {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    use super::what_its_for::{AreaCalculatorComponent, RectangleArea};

    pub struct MyContext<T>(pub PhantomData<T>);

    delegate_components! {
        <T> MyContext<T> {
            AreaCalculatorComponent: RectangleArea,
        }
    }
}

/// ## The three operators
///
/// The `:`-versus-`->` pair the page contrasts. `FooComponents` is named by the page without being
/// introduced, so it is declared here with the two entries the `->` line reaches into.
pub mod the_three_operators {
    use cgp::prelude::*;

    delegate_components! {
        new FooComponents {
            Index<0>: u64,
            Index<1>: String,
        }
    }

    delegate_components! {
        new BarComponents {
            Index<0>:
                FooComponents,     // the provider *is* FooComponents
            Index<1> ->
                FooComponents,     // the provider is whatever FooComponents wires for Index<1>
        }
    }

    /// The `=>` snippet from the same section: two components pointed at one shared slot.
    ///
    /// `Bar`, `Baz`, and `DummyImpl` are named by the page without being introduced.
    #[cgp_component(BarProvider)]
    pub trait Bar {
        fn bar(&self);
    }

    #[cgp_component(BazProvider)]
    pub trait Baz {
        fn baz(&self);
    }

    #[cgp_impl(new DummyImpl)]
    impl BarProvider {
        fn bar(&self) {}
    }

    #[cgp_impl(DummyImpl)]
    impl BazProvider {
        fn baz(&self) {}
    }

    pub struct App;

    delegate_components! {
        App {
            [BarProviderComponent, BazProviderComponent] =>
                @shared,

            @shared: DummyImpl,
        }
    }

    check_components! {
        App {
            BarProviderComponent,
            BazProviderComponent,
        }
    }
}

/// ## The three key forms
///
/// The list-key table, plus the single key carrying its own generics that the surrounding prose
/// mentions inline. The components and providers are named by the page without being introduced.
///
/// The page's `{…}`-group snippet is not reproduced here: it wires `ExtendedNamespace` and the error
/// components, which belong to the `cgp_namespace!` page. The two grouping forms are exercised
/// against a local component under [`choosing_a_provider_per_type_the_open_statement`] instead.
pub mod the_three_key_forms {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    #[cgp_component(FooProvider)]
    pub trait Foo {
        fn foo(&self);
    }

    #[cgp_component(BarProvider)]
    pub trait Bar {
        fn bar(&self);
    }

    #[cgp_component(BazProvider)]
    pub trait Baz {
        fn baz(&self);
    }

    #[cgp_impl(new FooBarProvider)]
    impl FooProvider {
        fn foo(&self) {}
    }

    #[cgp_impl(FooBarProvider)]
    impl BarProvider {
        fn bar(&self) {}
    }

    #[cgp_impl(new BazProvider2)]
    impl BazProvider {
        fn baz(&self) {}
    }

    delegate_components! {
        new MyComponents {
            [
                FooProviderComponent,
                BarProviderComponent,
            ]: FooBarProvider,
            BazProviderComponent: BazProvider2,
        }
    }

    /// The `<T> BazKey<T>: BazProvider` shape the prose names: a key carrying its own generic
    /// parameter, which the key must mention or the generated impl leaves it unconstrained. The
    /// page also says a bracketed element may carry generics, so one does here.
    ///
    /// The key types are declared rather than derived, since a generic key is a type-level lookup
    /// key rather than a `#[cgp_component]` marker, which is never generic.
    pub struct FooKey<T>(pub PhantomData<T>);
    pub struct BarKey<T>(pub PhantomData<T>);
    pub struct BazKey<T1, T2>(pub PhantomData<(T1, T2)>);
    pub struct FooValue;
    pub struct BarValue<T>(pub PhantomData<T>);

    delegate_components! {
        <T1: Clone>
        GenericKeyComponents {
            FooKey<T1>: FooValue,
            [
                BarKey<T1>,
                <T2> BazKey<T1, T2>,
            ]:
                BarValue<T1>,
        }
    }

    pub struct GenericKeyComponents;
}

/// ## The two value forms
///
/// The legacy nested-table value, in both shapes the page's admonition shows: the plain
/// `UseDelegate<new … { … }>` dispatch table, and the form whose inner table name carries generics
/// threaded in from a per-entry `<T>` on the outer key.
///
/// The page names `AreaCalculatorComponents`, `RectangleArea`, and `CircleArea` for the first and
/// `BarKey`/`BarValue`/`BazKey`/`BazValue` for the second; all are declared here. The keys are plain
/// types rather than component markers, which is what an inner dispatch table is keyed on.
pub mod the_two_value_forms {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    // `#[derive_delegate]` is what generates the `UseDelegate` dispatch impl, and the legacy form
    // does not resolve without it — which is the concrete difference the page draws between this
    // form and the `open` statement, whose redirect every `#[cgp_component]` already generates.
    #[cgp_component(AreaCalculator)]
    #[derive_delegate(UseDelegate<Shape>)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle;
    pub struct Circle;

    #[cgp_impl(new RectangleArea)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            0.0
        }
    }

    #[cgp_impl(new CircleArea)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            0.0
        }
    }

    pub struct MyApp;

    delegate_components! {
        MyApp {
            AreaCalculatorComponent:
                UseDelegate<new AreaCalculatorComponents {
                    Rectangle: RectangleArea,
                    Circle: CircleArea,
                }>,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }

    /// The generic inner table: the entry's `<T>` reaches the outer key, the wrapper, and the
    /// generated `struct BarValue<T>;` alike.
    pub struct BarKey<T>(pub PhantomData<T>);
    pub struct BazKey;
    pub struct BazValue<T>(pub PhantomData<T>);

    delegate_components! {
        new GenericInnerComponents {
            <T> BarKey<T>: UseDelegate<new BarValue<T> {
                BazKey: BazValue<T>,
            }>,
        }
    }
}

/// ## Choosing a provider per type: the `open` statement
///
/// The `open` header with its per-value path keys, extended to cover both grouping forms and a
/// generic path segment — the three path-key shapes the section describes. The page shows them
/// against `AreaCalculator`; the shapes are the point, so one component carries all of them here.
pub mod choosing_a_provider_per_type_the_open_statement {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle;
    pub struct Circle;
    pub struct Ellipse;
    pub struct Square;
    pub struct Triangle;

    #[cgp_impl(new RectangleArea)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            0.0
        }
    }

    #[cgp_impl(CurveArea)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            0.0
        }
    }

    pub struct CurveArea;

    #[cgp_impl(PolygonArea)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            0.0
        }
    }

    pub struct PolygonArea;

    pub struct MyApp;

    delegate_components! {
        MyApp {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.Rectangle: RectangleArea,
            @AreaCalculatorComponent.Circle: CurveArea,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent: [Rectangle, Circle],
        }
    }

    /// The braced group, the bracketed group, and a segment carrying its own generics, on one
    /// context — the forms the *three key forms* section contrasts.
    pub struct GroupedApp;

    delegate_components! {
        GroupedApp {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.{Circle, Ellipse}:
                CurveArea,

            @AreaCalculatorComponent.[Square, Triangle]:
                PolygonArea,

            @AreaCalculatorComponent.<'a, T> &'a T:
                RectangleArea,
        }
    }

    check_components! {
        GroupedApp {
            AreaCalculatorComponent: [
                Circle,
                Ellipse,
                Square,
                Triangle,
                <'a> &'a Rectangle,
            ],
        }
    }
}

/// ## Statements come first
///
/// The combined block, which is the page's demonstration that the forms mix. `BarBundle` and the
/// providers are named by the page without being introduced.
pub mod statements_come_first {
    use cgp::prelude::*;

    use super::choosing_a_provider_per_type_the_open_statement::{
        AreaCalculatorComponent, Circle, PolygonArea, Rectangle, Square, Triangle,
    };

    #[cgp_component(BarProvider)]
    pub trait Bar {
        fn bar(&self);
    }

    #[cgp_component(BazProvider)]
    pub trait Baz {
        fn baz(&self);
    }

    #[cgp_component(QuuxProvider)]
    pub trait Quux {
        fn quux(&self);
    }

    #[cgp_impl(new DummyBar)]
    impl BarProvider {
        fn bar(&self) {}
    }

    #[cgp_impl(new DummyBaz)]
    impl BazProvider {
        fn baz(&self) {}
    }

    #[cgp_impl(DummyBaz)]
    impl QuuxProvider {
        fn quux(&self) {}
    }

    delegate_components! {
        new BarBundle {
            BarProviderComponent: DummyBar,
        }
    }

    /// The page names `ShapeArea` and `PolygonArea` as the two shape providers; `ShapeArea` is
    /// declared here and `PolygonArea` is reused from the `open` section.
    #[cgp_impl(new ShapeArea)]
    impl<Shape> super::choosing_a_provider_per_type_the_open_statement::AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            0.0
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open AreaCalculatorComponent;

            BarProviderComponent -> BarBundle,

            [BazProviderComponent, QuuxProviderComponent]: DummyBaz,

            @AreaCalculatorComponent.{Rectangle, Circle}:
                ShapeArea,

            @AreaCalculatorComponent.[Square, Triangle]:
                PolygonArea,
        }
    }

    check_components! {
        App {
            AreaCalculatorComponent: [Rectangle, Circle, Square, Triangle],
            BarProviderComponent,
            BazProviderComponent,
            QuuxProviderComponent,
        }
    }
}

/// ## Examples
///
/// The two-context payoff the page builds to: the same capability answered differently by two
/// contexts, and a generic function serving both.
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

    pub fn print_area(rect: &Rectangle) {
        println!("area = {}", rect.area());
    }

    #[cgp_impl(new SquareArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] side: f64) -> f64 {
            side * side
        }
    }

    #[derive(HasField)]
    pub struct Square {
        pub side: f64,
    }

    delegate_components! {
        Square {
            AreaCalculatorComponent: SquareArea,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorComponent,
        }
    }

    check_components! {
        #[check_trait(__CheckSquare2)]
        Square {
            AreaCalculatorComponent,
        }
    }

    /// The page's claim that one generic function serves both contexts.
    pub fn total_area(shapes: &[&dyn Fn() -> f64]) -> f64 {
        shapes.iter().map(|f| f()).sum()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn each_context_answers_through_its_own_provider() {
            let rectangle = Rectangle {
                width: 3.0,
                height: 4.0,
            };
            let square = Square { side: 5.0 };

            assert_eq!(rectangle.area(), 12.0);
            assert_eq!(square.area(), 25.0);
        }
    }
}

/// ## Gotchas
///
/// The page's rejected snippet: a braced path group followed by more path. A braced group holds
/// whole tails and therefore ends the path, so the trailing `.bool` sits where the mapping's
/// operator should be and the macro reports `expected ':'`.
///
/// The bracketed form, which the page offers as the fix, is exercised above under
/// [`choosing_a_provider_per_type_the_open_statement`].
///
/// ```compile_fail
/// use cgp::prelude::*;
///
/// #[cgp_component(FooProvider)]
/// pub trait Foo<T> {
///     fn foo(&self, value: &T);
/// }
///
/// #[cgp_impl(new DummyFoo)]
/// impl<T> FooProvider<T> {
///     fn foo(&self, _value: &T) {}
/// }
///
/// pub struct App;
///
/// // error: expected `:`
/// delegate_components! {
///     App {
///         open FooProviderComponent;
///
///         @FooProviderComponent.{String, u32}.bool: DummyFoo,
///     }
/// }
/// ```
pub mod gotchas {}
