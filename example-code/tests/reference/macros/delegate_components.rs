//! Code from `docs/reference/macros/delegate_components.md` — *`delegate_components!`*.
//!
//! The snippets the page rejects, and the parse errors it quotes, live under
//! `tests/compile_fail/reference/macros/`.

/// ## Overview and Usage
///
/// The single-entry table the page opens both sections with. The component, the provider, and the
/// context it wires appear under *Examples*; they are declared here so the snippet has something
/// to name.
pub mod overview {
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

/// ### Defining the target at the same time
///
/// The aggregate-provider bundle. The page names `PerimeterCalculator` and its provider without
/// introducing them. No `check_components!` names the bundle, since the page's point is that a
/// context-side check on it asks the wrong question; `MyApp` verifies it instead.
pub mod defining_the_target_at_the_same_time {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    pub use super::overview::{AreaCalculatorComponent, CanCalculateArea, RectangleArea};

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

    // The prose's generic target: `new` declares it with a `PhantomData` field over its
    // parameters, a lifetime lifted into `Life`.
    delegate_components! {
        <'a, T> new MyComponents<'a, T> {
            AreaCalculatorComponent: RectangleArea,
        }
    }

    pub fn the_generic_target_has_a_phantom_field(
        bundle: MyComponents<'static, u32>,
    ) -> PhantomData<(Life<'static>, u32)> {
        bundle.0
    }

    // The prose's const target: the bare `N` takes its kind from the leading list, so the
    // declared struct is `ArrayTable<const N: usize>`, and its `PhantomData` field leaves `N` out.
    delegate_components! {
        <const N: usize> new ArrayTable<N> {
            AreaCalculatorComponent: RectangleArea,
        }
    }

    pub fn the_const_target_leaves_the_constant_out(table: ArrayTable<3>) -> PhantomData<()> {
        table.0
    }

    #[test]
    fn the_const_target_is_wired() {
        super::the_key_forms::delegates_to::<ArrayTable<3>, AreaCalculatorComponent, RectangleArea>();
    }

    #[test]
    fn the_bundle_answers_through_the_context() {
        let app = MyApp {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(app.area(), 6.0);
        assert_eq!(app.perimeter(), 10.0);
    }
}

/// ### One table for a family of types
///
/// The page shows only the table; the generic context it wires is declared here with the fields
/// `RectangleArea` reads, so one check covers every instantiation.
pub mod one_table_for_a_family_of_types {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    use super::overview::{AreaCalculatorComponent, CanCalculateArea, RectangleArea};

    #[derive(HasField)]
    pub struct MyContext<T> {
        pub width: f64,
        pub height: f64,
        pub marker: PhantomData<T>,
    }

    delegate_components! {
        <T> MyContext<T> {
            AreaCalculatorComponent: RectangleArea,
        }
    }

    check_components! {
        <T> MyContext<T> {
            AreaCalculatorComponent,
        }
    }

    #[test]
    fn every_instantiation_is_wired() {
        let context = MyContext::<String> {
            width: 2.0,
            height: 5.0,
            marker: PhantomData,
        };
        assert_eq!(context.area(), 10.0);
    }
}

/// ### The operators
///
/// The `:`-versus-`->` pair, and the `=>` table pointing two components at one slot. The page
/// names `ScaledAreaCalculator` and `RectangleGeometry` without declaring them.
pub mod the_operators {
    use cgp::prelude::*;

    pub use super::defining_the_target_at_the_same_time::{
        CanCalculatePerimeter, GeometryComponents, PerimeterCalculator, PerimeterCalculatorComponent,
    };
    pub use super::overview::{AreaCalculator, AreaCalculatorComponent, CanCalculateArea, RectangleArea};

    #[cgp_impl(new ScaledAreaCalculator<InnerCalculator>)]
    #[use_provider(InnerCalculator: AreaCalculator)]
    impl<InnerCalculator> AreaCalculator {
        fn area(&self, #[implicit] scale_factor: f64) -> f64 {
            InnerCalculator::area(self) * scale_factor * scale_factor
        }
    }

    delegate_components! {
        new ScaledGeometryComponents {
            AreaCalculatorComponent:
                ScaledAreaCalculator<RectangleArea>,     // the provider *is* this scaled calculator
            PerimeterCalculatorComponent ->
                GeometryComponents,     // the provider is whatever GeometryComponents wires for perimeter
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
            [AreaCalculatorComponent, PerimeterCalculatorComponent]: ScaledGeometryComponents,
        }
    }

    check_components! {
        ScaledRectangle {
            AreaCalculatorComponent,
            PerimeterCalculatorComponent,
        }
    }

    #[test]
    fn only_the_named_entry_is_scaled() {
        let rectangle = ScaledRectangle {
            width: 2.0,
            height: 3.0,
            scale_factor: 2.0,
        };
        assert_eq!(rectangle.area(), 24.0);
        assert_eq!(rectangle.perimeter(), 10.0);
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
    pub struct App {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        App {
            [AreaCalculatorComponent, PerimeterCalculatorComponent] =>
                @shared,

            @shared: RectangleGeometry,
        }
    }

    check_components! {
        App {
            AreaCalculatorComponent,
            PerimeterCalculatorComponent,
        }
    }

    #[test]
    fn one_slot_answers_both_components() {
        let app = App {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(app.area(), 6.0);
        assert_eq!(app.perimeter(), 10.0);
    }
}

/// ### The key forms
///
/// The single key with its own generics, the list key, the element generics, and the path keys with
/// both grouping forms. The page names every component and provider here without declaring them.
pub mod the_key_forms {
    use core::marker::PhantomData;

    use cgp::core::error::{ErrorRaiserComponent, ErrorWrapperComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    pub use super::the_operators::{
        AreaCalculatorComponent, PerimeterCalculatorComponent, RectangleGeometry,
    };

    /// `<Shape> ShapeAreaCalculatorComponent<Shape>: SumAreas`: the key's marker carries the
    /// trait's parameter, so the provider names the component explicitly.
    #[cgp_component {
        provider: ShapeAreaCalculator,
        name: ShapeAreaCalculatorComponent<Shape>,
    }]
    pub trait CanCalculateShapeArea<Shape> {
        fn shape_area(&self, shape: &Shape) -> f64;
    }

    pub struct Square {
        pub side: f64,
    }

    #[cgp_impl(new SumAreas: ShapeAreaCalculatorComponent<Shape>)]
    impl<Shape> ShapeAreaCalculator<Shape> {
        fn shape_area(&self, _shape: &Shape) -> f64 {
            0.0
        }
    }

    pub struct ShapeApp;

    delegate_components! {
        ShapeApp {
            <Shape> ShapeAreaCalculatorComponent<Shape>: SumAreas,
        }
    }

    check_components! {
        ShapeApp {
            ShapeAreaCalculatorComponent<Square>: Square,
        }
    }

    /// The list-key table. `MyComponents` is not declared with `new` on the page, so it is
    /// declared here, and a context delegating to it verifies it.
    #[cgp_component {
        provider: ColorProvider,
        name: ColorComponent,
    }]
    pub trait HasColor {
        fn color(&self) -> &'static str;
    }

    #[cgp_impl(new SolidColor: ColorComponent)]
    impl ColorProvider {
        fn color(&self) -> &'static str {
            "black"
        }
    }

    pub struct MyComponents;

    delegate_components! {
        MyComponents {
            [
                AreaCalculatorComponent,
                PerimeterCalculatorComponent,
            ]: RectangleGeometry,
            ColorComponent: SolidColor,
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
                ColorComponent,
            ]: MyComponents,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorComponent,
            PerimeterCalculatorComponent,
            ColorComponent,
        }
    }

    /// `[WidthKey<T1>, <T2> HeightKey<T1, T2>]: RectangleValue<T1>`: only the second element
    /// introduces `T2`. The keys are plain types rather than component markers, and the table's
    /// `<T1: Clone>` shows a key generic carrying a bound.
    pub struct WidthKey<T>(pub PhantomData<T>);
    pub struct HeightKey<T1, T2>(pub PhantomData<(T1, T2)>);
    pub struct RectangleValue<T>(pub PhantomData<T>);

    pub struct RectangleValues;

    delegate_components! {
        <T1: Clone> RectangleValues {
            [WidthKey<T1>, <T2> HeightKey<T1, T2>]: RectangleValue<T1>,
        }
    }

    /// The path keys the section names: a bracketed group continuing into a second group, and the
    /// braced group of tails joined to a namespace. Nothing redirects into `@app` here, so the
    /// entries are checked by naming their keys directly.
    pub struct Tagged;

    delegate_components! {
        Tagged {
            @app.[AreaCalculatorComponent, PerimeterCalculatorComponent].[u64, String]:
                RectangleGeometry,
        }
    }

    pub fn delegates_to<Table, Key, Value>()
    where
        Table: DelegateComponent<Key, Delegate = Value>,
    {
    }

    #[test]
    fn a_bracketed_path_expands_to_the_product() {
        delegates_to::<Tagged, Path!(@app.AreaCalculatorComponent.u64), RectangleGeometry>();
        delegates_to::<Tagged, Path!(@app.AreaCalculatorComponent.String), RectangleGeometry>();
        delegates_to::<Tagged, Path!(@app.PerimeterCalculatorComponent.u64), RectangleGeometry>();
        delegates_to::<Tagged, Path!(@app.PerimeterCalculatorComponent.String), RectangleGeometry>();
    }

    cgp_namespace! {
        new ExtendedNamespace: DefaultNamespace {
            @cgp.core.error =>
                @app,
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace ExtendedNamespace;

            @app.{
                ErrorRaiserComponent.{&'static str, String},
                ErrorWrapperComponent,
            }: RaiseFrom,
        }
    }

    #[test]
    fn a_braced_path_covers_tails_of_different_lengths() {
        delegates_to::<App, Path!(@app.ErrorRaiserComponent.String), RaiseFrom>();
        delegates_to::<App, Path!(@app.ErrorWrapperComponent), RaiseFrom>();
    }

    /// Every list form accepts being empty, and produces no entries.
    pub struct Empty;

    delegate_components! {
        Empty {
            open {};

            []: RectangleGeometry,
            @AreaCalculatorComponent.{}: RectangleGeometry,
            @AreaCalculatorComponent.[]: RectangleGeometry,
        }
    }
}

/// ### The value forms
///
/// The legacy nested-table value in both shapes the page shows: the `UseDelegate<new … { … }>`
/// dispatch table, and the form whose inner table name carries generics from a per-entry `<T>` on
/// the outer key. The page names every type here without declaring it.
pub mod the_value_forms {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    // `#[derive_delegate]` generates the `UseDelegate` dispatch impl the wrapper resolves through,
    // which is the prerequisite the page points out.
    #[cgp_component(AreaCalculator)]
    #[derive_delegate(UseDelegate<Shape>)]
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

    #[test]
    fn the_inner_table_dispatches_per_shape() {
        let rectangle = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(MyApp.area(&rectangle), 6.0);
    }

    /// The generic inner table: the entry's `<T>` reaches the outer key, the wrapper, and the
    /// generated `struct WidthValue<T>` alike.
    pub struct WidthKey<T>(pub PhantomData<T>);
    pub struct HeightKey;
    pub struct HeightValue<T>(pub PhantomData<T>);

    delegate_components! {
        new MyComponents {
            <T> WidthKey<T>: UseDelegate<new WidthValue<T> {
                HeightKey: HeightValue<T>,
            }>,
        }
    }

    pub fn the_inner_table_is_generic(table: WidthValue<u8>) -> PhantomData<u8> {
        table.0
    }

    /// The prose's const inner table: its list declares the struct, so the `const` keeps its kind.
    pub struct ArrayKey<const N: usize>;

    delegate_components! {
        new ConstComponents {
            <const N: usize> ArrayKey<N>: UseDelegate<new ArrayTable<const N: usize> {
                Rectangle: RectangleArea,
            }>,
        }
    }

    #[test]
    fn the_inner_table_carries_a_const() {
        super::the_key_forms::delegates_to::<ArrayTable<3>, Rectangle, RectangleArea>();
        super::the_key_forms::delegates_to::<ConstComponents, ArrayKey<3>, UseDelegate<ArrayTable<3>>>();
    }
}

/// ### Choosing a provider per type: the `open` statement
///
/// The generic component, the `open` header with per-value path keys, and the two-parameter
/// component dispatched on its input alone and on both parameters. The page names the providers
/// and shapes without declaring them.
pub mod choosing_a_provider_per_type_the_open_statement {
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

    /// The braced spelling of the header is the same statement.
    pub struct BracedApp;

    delegate_components! {
        BracedApp {
            open { AreaCalculatorComponent };

            @AreaCalculatorComponent.Rectangle: RectangleArea,
        }
    }

    check_components! {
        BracedApp {
            AreaCalculatorComponent: Rectangle,
        }
    }

    /// The key-forms section's `@SomeComponent.<'a, T> &'a T: SomeProvider`: a path segment
    /// carrying its own generics.
    #[cgp_impl(new AnyReferenceArea)]
    impl<'a, T> AreaCalculator<&'a T> {
        fn area(&self, _shape: &&'a T) -> f64 {
            0.0
        }
    }

    pub struct ReferenceApp;

    delegate_components! {
        ReferenceApp {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.<'a, T> &'a T: AnyReferenceArea,
        }
    }

    check_components! {
        ReferenceApp {
            AreaCalculatorComponent: <'a> &'a Rectangle,
        }
    }

    pub mod later_parameters {
        use cgp::extra::handler::{CanCompute, Computer, ComputerComponent};
        use cgp::prelude::*;

        pub struct Area;
        pub struct Perimeter;

        pub struct Circle {
            pub radius: f64,
        }

        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        #[cgp_impl(new DescribeCircle)]
        impl<Code> Computer<Code, Circle> {
            type Output = f64;

            fn compute(&self, _code: PhantomData<Code>, circle: Circle) -> f64 {
                circle.radius
            }
        }

        #[cgp_impl(new RectangleArea)]
        impl Computer<Area, Rectangle> {
            type Output = f64;

            fn compute(&self, _code: PhantomData<Area>, rectangle: Rectangle) -> f64 {
                rectangle.width * rectangle.height
            }
        }

        #[cgp_impl(new RectanglePerimeter)]
        impl Computer<Perimeter, Rectangle> {
            type Output = f64;

            fn compute(&self, _code: PhantomData<Perimeter>, rectangle: Rectangle) -> f64 {
                2.0 * (rectangle.width + rectangle.height)
            }
        }

        pub struct MyApp;

        delegate_components! {
            MyApp {
                open ComputerComponent;

                @ComputerComponent.<Code> Code.Circle: DescribeCircle,
                @ComputerComponent.Area.Rectangle: RectangleArea,
                @ComputerComponent.Perimeter.Rectangle: RectanglePerimeter,
            }
        }

        check_components! {
            MyApp {
                ComputerComponent: [
                    (Area, Circle),
                    (Perimeter, Circle),
                    (Area, Rectangle),
                    (Perimeter, Rectangle),
                ],
            }
        }

        #[test]
        fn the_input_or_both_parameters_select_the_provider() {
            let rectangle = || Rectangle {
                width: 2.0,
                height: 3.0,
            };

            assert_eq!(MyApp.compute(PhantomData::<Area>, Circle { radius: 1.5 }), 1.5);
            assert_eq!(MyApp.compute(PhantomData::<Perimeter>, Circle { radius: 1.5 }), 1.5);
            assert_eq!(MyApp.compute(PhantomData::<Area>, rectangle()), 6.0);
            assert_eq!(MyApp.compute(PhantomData::<Perimeter>, rectangle()), 10.0);
        }
    }
}

/// ### Statements come first
///
/// The combined block, which is the page's demonstration that the forms mix. It opens the generic
/// area component while forwarding the plain perimeter component into `GeometryComponents`; the
/// page names the style components and every provider without declaring them.
pub mod statements_come_first {
    use cgp::prelude::*;

    pub use super::defining_the_target_at_the_same_time::{
        CanCalculatePerimeter, GeometryComponents, PerimeterCalculatorComponent,
    };
    use super::the_key_forms::{ColorComponent, ColorProvider};

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle;
    pub struct Circle;
    pub struct Square;
    pub struct Triangle;

    #[cgp_impl(new ShapeArea)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            1.0
        }
    }

    #[cgp_impl(new PolygonArea)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            2.0
        }
    }

    #[cgp_component {
        provider: LabelProvider,
        name: LabelComponent,
    }]
    pub trait HasLabel {
        fn label(&self) -> &'static str;
    }

    #[cgp_impl(new DefaultStyle: ColorComponent)]
    impl ColorProvider {
        fn color(&self) -> &'static str {
            "black"
        }
    }

    #[cgp_impl(DefaultStyle: LabelComponent)]
    impl LabelProvider {
        fn label(&self) -> &'static str {
            "shape"
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! {
        App {
            open AreaCalculatorComponent;

            PerimeterCalculatorComponent -> GeometryComponents,

            [ColorComponent, LabelComponent]: DefaultStyle,

            @AreaCalculatorComponent.{Rectangle, Circle}:
                ShapeArea,

            @AreaCalculatorComponent.[Square, Triangle]:
                PolygonArea,
        }
    }

    check_components! {
        App {
            AreaCalculatorComponent: [Rectangle, Circle, Square, Triangle],
            PerimeterCalculatorComponent,
            ColorComponent,
            LabelComponent,
        }
    }

    #[test]
    fn every_form_contributes_to_one_table() {
        let app = App {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(app.area(&Circle), 1.0);
        assert_eq!(app.area(&Triangle), 2.0);
        assert_eq!(app.perimeter(), 10.0);
    }
}

/// ## Examples
///
/// The two contexts answering one trait through different providers, a function generic over the
/// trait serving both, and the per-type table on an environmental context.
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
        Square {
            AreaCalculatorComponent,
        }
    }

    /// The page's claim that a function generic over the trait serves both contexts.
    pub fn doubled_area<Shape: CanCalculateArea>(shape: &Shape) -> f64 {
        shape.area() * 2.0
    }

    #[test]
    fn each_context_answers_through_its_own_provider() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        print_area(&rectangle);

        assert_eq!(doubled_area(&rectangle), 24.0);
        assert_eq!(doubled_area(&Square { side: 5.0 }), 50.0);
    }

    /// The per-type table. `MyApp` wires the generic `CanCalculateArea<Shape>` from *Usage*, so it
    /// sits in its own module; the page names the shapes and `CurveArea` without declaring them.
    pub mod per_type {
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

        pub struct Ellipse {
            pub major: f64,
            pub minor: f64,
        }

        pub trait HasSemiAxes {
            fn semi_axes(&self) -> (f64, f64);
        }

        impl HasSemiAxes for Circle {
            fn semi_axes(&self) -> (f64, f64) {
                (self.radius, self.radius)
            }
        }

        impl HasSemiAxes for Ellipse {
            fn semi_axes(&self) -> (f64, f64) {
                (self.major, self.minor)
            }
        }

        #[cgp_impl(new RectangleArea)]
        impl AreaCalculator<Rectangle> {
            fn area(&self, shape: &Rectangle) -> f64 {
                shape.width * shape.height
            }
        }

        #[cgp_impl(new CurveArea)]
        impl<Shape: HasSemiAxes> AreaCalculator<Shape> {
            fn area(&self, shape: &Shape) -> f64 {
                let (a, b) = shape.semi_axes();
                core::f64::consts::PI * a * b
            }
        }

        pub struct MyApp;

        delegate_components! {
            MyApp {
                open AreaCalculatorComponent;

                @AreaCalculatorComponent.Rectangle: RectangleArea,
                @AreaCalculatorComponent.{Circle, Ellipse}: CurveArea,
            }
        }

        check_components! {
            MyApp {
                AreaCalculatorComponent: [Rectangle, Circle, Ellipse],
            }
        }

        #[test]
        fn each_shape_reaches_its_provider() {
            let rectangle = Rectangle {
                width: 2.0,
                height: 3.0,
            };
            assert_eq!(MyApp.area(&rectangle), 6.0);
            assert_eq!(MyApp.area(&Circle { radius: 1.0 }), core::f64::consts::PI);
        }
    }
}

/// ## Under the hood
///
/// The contexts whose expansions the page lists, checked with `cargo cgp expand --item` on each
/// item: `super::overview::Rectangle` for the plain pair, `super::the_key_forms::MyComponents` for
/// the list key, `super::the_operators::ScaledGeometryComponents` for `->`, `super::the_operators::App`
/// for `=>`, and the items below for `open`, its `=>` spelling, and the namespace statement.
pub mod under_the_hood {
    use cgp::prelude::*;

    pub use super::choosing_a_provider_per_type_the_open_statement::{
        AreaCalculatorComponent, Rectangle, RectangleArea,
    };

    pub struct MyApp;

    delegate_components! {
        MyApp {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.Rectangle: RectangleArea,
        }
    }

    check_components! {
        MyApp {
            AreaCalculatorComponent: Rectangle,
        }
    }

    /// `AreaCalculatorComponent => @AreaCalculatorComponent,`, whose impl the page says is the
    /// `open` header's byte for byte.
    pub struct RedirectApp;

    delegate_components! {
        RedirectApp {
            AreaCalculatorComponent => @AreaCalculatorComponent,

            @AreaCalculatorComponent.Rectangle: RectangleArea,
        }
    }

    check_components! {
        RedirectApp {
            AreaCalculatorComponent: Rectangle,
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace DefaultNamespace;
        }
    }
}

/// ## Common Mistakes
///
/// The fix the page gives for a bare `N` in a nested table's list: write the parameter with its
/// kind. The rejected snippets are the trybuild fixtures.
pub mod common_mistakes {
    use cgp::prelude::*;

    pub use super::overview::{AreaCalculatorComponent, RectangleArea};

    pub struct ArrayKey<const N: usize>;

    delegate_components! {
        new MyComponents {
            <const N: usize> ArrayKey<N>: UseDelegate<new ArrayTable<const N: usize> {
                u32: RectangleArea,
            }>,
        }
    }

    #[test]
    fn the_const_inner_table_is_wired() {
        super::the_key_forms::delegates_to::<ArrayTable<3>, u32, RectangleArea>();
        super::the_key_forms::delegates_to::<MyComponents, ArrayKey<3>, UseDelegate<ArrayTable<3>>>();
    }
}
