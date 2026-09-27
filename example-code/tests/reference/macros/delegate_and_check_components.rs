//! Code from `docs/reference/macros/delegate_and_check_components.md` —
//! *`delegate_and_check_components!`*.
//!
//! The snippets the page rejects, and the errors it quotes, live under
//! `tests/compile_fail/reference/macros/`. Each snippet that wires `ScaledRectangle` or `MyApp`
//! gets its own module, since one context cannot be wired twice.

/// ## Overview
///
/// The name component pair the opening snippet wires. The page declares neither trait there; the
/// getter appears under *Examples* and the type component is declared here.
pub mod overview {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasNameType {
        type Name;
    }

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct MyContext {
        pub name: String,
    }

    delegate_and_check_components! {
        MyContext {
            NameTypeProviderComponent: UseType<String>,
            NameGetterComponent: UseField<Symbol!("name")>,
        }
    }

    #[test]
    fn the_wired_getter_reads_the_field() {
        let context = MyContext {
            name: "Alice".to_owned(),
        };
        assert_eq!(context.name(), "Alice");
    }
}

/// ## Usage
///
/// The scaled-area context the page's attribute examples share. The page names the component,
/// `ScaledArea`, and `RectangleArea` without declaring them.
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

    delegate_and_check_components! {
        ScaledRectangle {
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    #[test]
    fn the_scaled_area_is_wired_and_checked() {
        let rectangle = ScaledRectangle {
            width: 2.0,
            height: 3.0,
            scale_factor: 2.0,
        };
        assert_eq!(rectangle.area(), 24.0);
    }
}

/// ### Naming the check trait
///
/// The table-level override. The same context is redeclared so it can be wired again.
pub mod naming_the_check_trait {
    use cgp::prelude::*;

    use super::usage::{AreaCalculatorComponent, RectangleArea, ScaledArea};

    #[derive(HasField)]
    pub struct ScaledRectangle {
        pub width: f64,
        pub height: f64,
        pub scale_factor: f64,
    }

    delegate_and_check_components! {
        #[check_trait(TestScaledRectangle)]
        ScaledRectangle {
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    /// The override names the trait, so it can be referred to.
    fn the_trait_is_named<T: TestScaledRectangle<AreaCalculatorComponent, ()>>() {}

    #[test]
    fn the_named_trait_holds() {
        the_trait_is_named::<ScaledRectangle>();
    }
}

/// ### Components with generic parameters
///
/// `#[check_params(Rectangle, Circle)]` on an entry wired to a provider generic over the shape.
/// The page names the component, the shapes, and `ShapeArea` without declaring them.
pub mod components_with_generic_parameters {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub trait HasBoundingBox {
        fn bounding_box(&self) -> (f64, f64);
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    pub struct Circle {
        pub radius: f64,
    }

    impl HasBoundingBox for Rectangle {
        fn bounding_box(&self) -> (f64, f64) {
            (self.width, self.height)
        }
    }

    impl HasBoundingBox for Circle {
        fn bounding_box(&self) -> (f64, f64) {
            (2.0 * self.radius, 2.0 * self.radius)
        }
    }

    #[cgp_impl(new ShapeArea)]
    impl<Shape: HasBoundingBox> AreaCalculator<Shape> {
        fn area(&self, shape: &Shape) -> f64 {
            let (width, height) = shape.bounding_box();
            width * height
        }
    }

    pub struct MyApp;

    delegate_and_check_components! {
        MyApp {
            #[check_params(Rectangle, Circle)]
            AreaCalculatorComponent: ShapeArea, // a provider generic over the shape
        }
    }

    fn each_parameter_is_checked<
        T: __CanUseMyApp<AreaCalculatorComponent, Rectangle>
            + __CanUseMyApp<AreaCalculatorComponent, Circle>,
    >() {
    }

    #[test]
    fn both_shapes_are_checked() {
        each_parameter_is_checked::<MyApp>();
        assert_eq!(MyApp.area(&Circle { radius: 1.0 }), 4.0);
    }
}

/// ### Skipping one entry's check
pub mod skipping_one_entrys_check {
    use cgp::prelude::*;

    use super::usage::{AreaCalculatorComponent, RectangleArea, ScaledArea};

    #[derive(HasField)]
    pub struct ScaledRectangle {
        pub width: f64,
        pub height: f64,
        pub scale_factor: f64,
    }

    delegate_and_check_components! {
        ScaledRectangle {
            #[skip_check]
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    // The skipped entry is still wired; a standalone block verifies it.
    check_components! {
        ScaledRectangle {
            AreaCalculatorComponent,
        }
    }
}

/// ### Attributes on a list key merge
///
/// The list-level and element-level `#[check_params]` combining per element. The page names
/// `RotatorComponent` and `ShapeProvider` without declaring them.
pub mod attributes_on_a_list_key_merge {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    #[cgp_component(Rotator)]
    pub trait CanRotate<Shape> {
        fn rotate(&self, shape: &Shape, degrees: f64) -> f64;
    }

    pub struct Rectangle;
    pub struct Circle;

    #[cgp_impl(new ShapeProvider)]
    impl<Shape> AreaCalculator<Shape> {
        fn area(&self, _shape: &Shape) -> f64 {
            1.0
        }
    }

    #[cgp_impl(ShapeProvider)]
    impl<Shape> Rotator<Shape> {
        fn rotate(&self, _shape: &Shape, degrees: f64) -> f64 {
            degrees
        }
    }

    pub struct MyApp;

    delegate_and_check_components! {
        MyApp {
            #[check_params(Rectangle)]
            [
                #[check_params(Circle)]
                AreaCalculatorComponent,
                RotatorComponent,
            ]: ShapeProvider,
        }
    }

    fn the_merged_checks_exist<
        T: __CanUseMyApp<AreaCalculatorComponent, Rectangle>
            + __CanUseMyApp<AreaCalculatorComponent, Circle>
            + __CanUseMyApp<RotatorComponent, Rectangle>,
    >() {
    }

    #[test]
    fn each_element_gets_its_merged_parameters() {
        the_merged_checks_exist::<MyApp>();
    }
}

/// ### Two forms that quietly check nothing
///
/// The empty `#[check_params()]`, an all-skipped table, and a key carrying only generics, which the
/// page says is checked at unit parameters and fails for a component whose marker carries the
/// trait's parameter (a trybuild fixture). The working `#[check_params(I)]` form is below.
pub mod two_forms_that_quietly_check_nothing {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    #[cgp_component(PerimeterCalculator)]
    pub trait CanCalculatePerimeter {
        fn perimeter(&self) -> f64;
    }

    #[cgp_impl(new NeedsWidth)]
    impl PerimeterCalculator {
        fn perimeter(&self, #[implicit] width: f64) -> f64 {
            width
        }
    }

    // `App` has no `width` field, so a real check would fail; the empty list skips it silently,
    // and the table still emits `__CanUseApp` with no impls.
    pub struct App;

    delegate_and_check_components! {
        App {
            #[check_params()]
            PerimeterCalculatorComponent: NeedsWidth,
        }
    }

    #[cgp_component {
        name: FooKeyComponent<I>,
        provider: FooProvider,
    }]
    pub trait CanFoo<I> {
        fn foo(&self, tag: PhantomData<I>) -> u8;
    }

    #[cgp_impl(new AnyFoo: FooKeyComponent<I>)]
    impl<I> FooProvider<I> {
        fn foo(&self, _tag: PhantomData<I>) -> u8 {
            0
        }
    }

    pub struct FooApp;

    delegate_and_check_components! {
        FooApp {
            #[check_params(I)]
            <I> FooKeyComponent<I>: AnyFoo,
        }
    }

    #[test]
    fn the_key_generic_is_the_parameter() {
        assert_eq!(FooApp.foo(PhantomData::<u64>), 0);
    }
}

/// ### What is wired, and what is checked
///
/// One table carrying every row of the page's coverage table. Only the first three rows are
/// checked by the derivation; a standalone block checks the rest.
pub mod what_is_wired_and_what_is_checked {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    #[cgp_component(PerimeterCalculator)]
    pub trait CanCalculatePerimeter {
        fn perimeter(&self) -> f64;
    }

    #[cgp_component(ColorGetter)]
    pub trait HasColor {
        fn color(&self) -> &'static str;
    }

    #[cgp_component(LabelGetter)]
    pub trait HasLabel {
        fn label(&self) -> &'static str;
    }

    pub struct Rectangle;

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator<Rectangle> {
        fn area(&self, _shape: &Rectangle) -> f64 {
            1.0
        }
    }

    #[cgp_impl(new FixedPerimeter)]
    impl PerimeterCalculator {
        fn perimeter(&self) -> f64 {
            4.0
        }
    }

    #[cgp_impl(new DefaultStyle)]
    impl ColorGetter {
        fn color(&self) -> &'static str {
            "black"
        }
    }

    #[cgp_impl(DefaultStyle)]
    impl LabelGetter {
        fn label(&self) -> &'static str {
            "shape"
        }
    }

    delegate_components! {
        new StyleComponents {
            LabelGetterComponent: DefaultStyle,
        }
    }

    pub struct App;

    delegate_and_check_components! {
        App {
            open AreaCalculatorComponent;

            PerimeterCalculatorComponent: FixedPerimeter,
            LabelGetterComponent -> StyleComponents,
            ColorGetterComponent => @style,

            @style: DefaultStyle,
            @AreaCalculatorComponent.Rectangle: RectangleArea,
        }
    }

    check_components! {
        App {
            AreaCalculatorComponent: Rectangle,
            ColorGetterComponent,
        }
    }
}

/// ## Examples
///
/// The getter context, and the mixed table whose skipped entry a per-layer `#[check_providers]`
/// block verifies. The page names `RectanglePerimeter` and reuses the scaled-area providers.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct MyContext {
        pub name: String,
    }

    delegate_and_check_components! {
        MyContext {
            NameGetterComponent: UseField<Symbol!("name")>,
        }
    }

    pub mod mixed {
        use cgp::prelude::*;

        pub use crate::reference::macros::delegate_and_check_components::usage::{
            AreaCalculatorComponent, CanCalculateArea, RectangleArea, ScaledArea,
        };

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

        #[derive(HasField)]
        pub struct ScaledRectangle {
            pub width: f64,
            pub height: f64,
            pub scale_factor: f64,
        }

        delegate_and_check_components! {
            ScaledRectangle {
                PerimeterCalculatorComponent:
                    RectanglePerimeter,

                #[skip_check]
                AreaCalculatorComponent:
                    ScaledArea<RectangleArea>,
            }
        }

        check_components! {
            #[check_providers(RectangleArea, ScaledArea<RectangleArea>)]
            ScaledRectangle {
                AreaCalculatorComponent,
            }
        }

        #[test]
        fn both_entries_are_wired() {
            let rectangle = ScaledRectangle {
                width: 2.0,
                height: 3.0,
                scale_factor: 2.0,
            };
            assert_eq!(rectangle.area(), 24.0);
            assert_eq!(rectangle.perimeter(), 10.0);
        }
    }
}

/// ## Under the hood
///
/// The input whose expansion the page lists, checked with
/// `cargo cgp expand --item reference::macros::delegate_and_check_components::under_the_hood`, the
/// generic table, and the generic-marker key with its parameters.
pub mod under_the_hood {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    pub use super::overview::{HasName, NameGetterComponent, NameTypeProviderComponent};

    #[derive(HasField)]
    pub struct MyContext {
        pub name: String,
    }

    delegate_and_check_components! {
        #[check_trait(CheckMyContext)]
        MyContext {
            NameTypeProviderComponent: UseType<String>,
            NameGetterComponent: UseField<Symbol!("name")>,
        }
    }

    #[derive(HasField)]
    pub struct GenericContext<T> {
        pub name: String,
        pub marker: PhantomData<T>,
    }

    delegate_and_check_components! {
        <T> GenericContext<T> {
            NameGetterComponent: UseField<Symbol!("name")>,
        }
    }

    #[cgp_type]
    pub trait HasBarType {
        type Bar;
    }

    #[cgp_getter {
        name: BarGetterAtComponent<I>,
        provider: BarGetterAt,
    }]
    pub trait HasBarAt<I, J>: HasBarType {
        fn bar(&self, _tag: PhantomData<(I, J)>) -> &Self::Bar;
    }

    #[derive(HasField)]
    pub struct BarContext {
        pub dummy: (),
    }

    delegate_and_check_components! {
        BarContext {
            BarTypeProviderComponent: UseType<()>,

            #[check_params((I, Index<0>))]
            <I> BarGetterAtComponent<I>: UseField<Symbol!("dummy")>,
        }
    }
}

/// ## Common Mistakes
///
/// The compiling half of the nested-table entry: an attribute on an inner key is dropped without
/// an error. The failing snippets are the trybuild fixtures.
pub mod common_mistakes {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    #[derive_delegate(UseDelegate<Shape>)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct Rectangle;

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator<Rectangle> {
        fn area(&self, _shape: &Rectangle) -> f64 {
            1.0
        }
    }

    pub struct MyApp;

    delegate_and_check_components! {
        MyApp {
            #[check_params(Rectangle)]
            AreaCalculatorComponent:
                UseDelegate<new AreaCalculatorComponents {
                    #[skip_check]
                    Rectangle: RectangleArea,
                }>,
        }
    }
}
