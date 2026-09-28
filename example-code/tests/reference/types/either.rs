//! Code from `docs/reference/types/either.md` — *`Either`*.
//!
//! `Either`/`Void` are what `Sum!` expands to. The Examples program reads an enum's shape as an
//! `Either` chain of `Field` entries, matches it by nesting depth with a `Void` arm that closes the
//! match, and builds a standalone `Sum!` value.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub enum Shape {
        Circle(f64),
        Rectangle { width: f64, height: f64 },
    }

    pub fn area(shape: Shape) -> f64 {
        match shape.to_fields() {
            Either::Left(circle) => core::f64::consts::PI * circle.value * circle.value,
            Either::Right(Either::Left(rectangle)) => {
                let Cons(width, Cons(height, Nil)) = rectangle.value;
                width.value * height.value
            }
            Either::Right(Either::Right(void)) => match void {},
        }
    }

    pub type Token = Sum![u32, String, bool];

    pub fn demo() {
        assert_eq!(
            area(Shape::Rectangle {
                width: 2.0,
                height: 3.0
            }),
            6.0
        );

        // The `String` branch sits one `Right` deep.
        let token: Token = Either::Right(Either::Left("hi".to_owned()));
        assert_eq!(token, Either::Right(Either::Left("hi".to_owned())));

        // `Sum!` is the right-nested `Either` chain ending in `Void`.
        let _: PhantomData<Either<u32, Either<String, Either<bool, Void>>>> = PhantomData::<Token>;
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
