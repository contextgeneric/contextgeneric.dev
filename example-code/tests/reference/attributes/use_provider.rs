//! Code from `docs/reference/attributes/use_provider.md` — *`#[use_provider]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/attributes/`.

/// ## Usage
///
/// Several traits on one provider, several stacked providers, a lifetime-carrying provider
/// trait, and the trailing-`+` and empty-list forms the grammar allows.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_component(PerimeterCalculator)]
    pub trait CanCalculatePerimeter {
        fn perimeter(&self) -> f64;
    }

    #[cgp_impl(new RectangleShape)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
        }
    }

    #[cgp_impl(RectangleShape)]
    impl PerimeterCalculator {
        fn perimeter(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            2.0 * (width + height)
        }
    }

    #[cgp_impl(new AreaPlusPerimeter<Inner>)]
    #[use_provider(Inner: AreaCalculator + PerimeterCalculator)]
    impl<Inner> AreaCalculator {
        fn area(&self) -> f64 {
            Inner::area(self) + Inner::perimeter(self)
        }
    }

    #[cgp_impl(new AreaTimesPerimeter<A, P>)]
    #[use_provider(A: AreaCalculator)]
    #[use_provider(P: PerimeterCalculator)]
    impl<A, P> AreaCalculator {
        fn area(&self) -> f64 {
            A::area(self) * P::perimeter(self)
        }
    }

    #[cgp_impl(new TrailingPlus<Inner>)]
    #[use_provider(Inner: AreaCalculator +)]
    impl<Inner> AreaCalculator {
        fn area(&self) -> f64 {
            Inner::area(self)
        }
    }

    #[cgp_impl(new Constant<Inner>)]
    #[use_provider(Inner:)]
    impl<Inner> AreaCalculator {
        fn area(&self) -> f64 {
            1.0
        }
    }

    #[cgp_component(Namer)]
    pub trait CanName<'a> {
        fn name(&self, name: &'a str) -> &'a str;
    }

    #[cgp_impl(new SameName)]
    impl<'a> Namer<'a> {
        fn name(&self, name: &'a str) -> &'a str {
            name
        }
    }

    #[cgp_impl(new ForwardName<Inner>)]
    #[use_provider(Inner: Namer<'a>)]
    impl<'a, Inner> Namer<'a> {
        fn name(&self, name: &'a str) -> &'a str {
            Inner::name(self, name)
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }


    delegate_components! {
        Rectangle {
            AreaCalculatorComponent: AreaPlusPerimeter<RectangleShape>,
            PerimeterCalculatorComponent: RectangleShape,
            NamerComponent: ForwardName<SameName>,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorComponent,
            PerimeterCalculatorComponent,
        }
    }

    #[test]
    fn every_form_composes() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.area(), 16.0);
        assert_eq!(
            <AreaTimesPerimeter<RectangleShape, RectangleShape> as AreaCalculator<Rectangle>>::area(
                &rect
            ),
            60.0
        );
        assert_eq!(
            <TrailingPlus<RectangleShape> as AreaCalculator<Rectangle>>::area(&rect),
            6.0
        );
        assert_eq!(<Constant<()> as AreaCalculator<Rectangle>>::area(&rect), 1.0);
        assert_eq!(rect.name("ada"), "ada");
    }
}

/// ## Examples
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

    #[cgp_impl(new ScaledArea<InnerCalculator>)]
    #[use_provider(InnerCalculator: AreaCalculator)]
    impl<InnerCalculator> AreaCalculator {
        fn area(&self, #[implicit] scale_factor: f64) -> f64 {
            let base_area = InnerCalculator::area(self);
            base_area * scale_factor * scale_factor
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
        pub scale_factor: f64,
    }

    delegate_components! {
        Rectangle {
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorComponent,
        }
    }

    pub type ScaledRectangle = ScaledArea<RectangleArea>;

    #[cgp_fn]
    #[use_provider(RectangleArea: AreaCalculator)]
    pub fn rect_area(&self) -> f64 {
        RectangleArea::area(self)
    }

    #[test]
    fn the_wrapper_scales_the_inner_area() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
            scale_factor: 2.0,
        };
        assert_eq!(rect.area(), 24.0);
        assert_eq!(<ScaledRectangle as AreaCalculator<Rectangle>>::area(&rect), 24.0);
        assert_eq!(rect.rect_area(), 6.0);
    }
}

/// ## When to use it
///
/// A wrapper whose parameter defaults to `UseContext`, declared by hand, falls back to the
/// context's own wiring. The page names `IterSum` without defining it; here it sums the areas of
/// a context's shapes, reading each through the context's wiring for that shape.
pub mod when_to_use_it {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }

    pub struct IterSum<Inner = UseContext>(pub PhantomData<Inner>);

    #[cgp_impl(IterSum<Inner>)]
    #[use_provider(Inner: AreaCalculator<Shape>)]
    impl<Shape, Inner> AreaCalculator<Vec<Shape>> {
        fn area(&self, shapes: &Vec<Shape>) -> f64 {
            shapes.iter().map(|shape| Inner::area(self, shape)).sum()
        }
    }

    pub struct Square(pub f64);

    #[cgp_impl(new SquareArea)]
    impl AreaCalculator<Square> {
        fn area(&self, square: &Square) -> f64 {
            square.0 * square.0
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open AreaCalculatorComponent;

            @AreaCalculatorComponent.Square: SquareArea,
            @AreaCalculatorComponent.Vec<Square>: IterSum,
        }
    }

    check_components! {
        App {
            AreaCalculatorComponent: [Square, Vec<Square>],
        }
    }

    #[test]
    fn the_default_reads_the_context_wiring() {
        assert_eq!(App.area(&vec![Square(1.0), Square(2.0)]), 5.0);
    }
}

/// ## Under the hood
pub mod under_the_hood {
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

    #[cgp_fn]
    #[use_provider(RectangleArea: AreaCalculator)]
    pub fn rect_area(&self) -> f64 {
        RectangleArea::area(self)
    }
}
