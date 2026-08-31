//! Code from `docs/reference/types/either.md` — *`Either`*.
//!
//! `Either`/`Void` are what `Sum!` expands to. This pins the `Either` chain a derive assigns to an enum
//! and the branch a nested `Either` value selects.

/// ## Examples
///
/// An enum's shape is an `Either` chain of `Field` entries terminated by `Void`, and a `Sum!` value
/// selects one branch by its nesting depth.
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub enum Shape {
        Circle(f64),
        Rectangle { width: f64, height: f64 },
    }

    // The page shows the generated `Fields` as an `Either` chain; the coercion checks it.
    pub fn assert_shape_fields(
        fields: <Shape as HasFields>::Fields,
    ) -> Either<
        Field<Symbol!("Circle"), f64>,
        Either<
            Field<
                Symbol!("Rectangle"),
                Product![Field<Symbol!("width"), f64>, Field<Symbol!("height"), f64>],
            >,
            Void,
        >,
    > {
        fields
    }

    pub type Token = Sum![u32, String, bool];

    pub fn assert_token_is_either(token: Token) -> Either<u32, Either<String, Either<bool, Void>>> {
        token
    }

    #[test]
    fn test_a_value_selects_a_branch_by_depth() {
        // the String branch
        let t: Token = Either::Right(Either::Left("hi".to_string()));
        match t {
            Either::Right(Either::Left(s)) => assert_eq!(s, "hi"),
            _ => panic!("wrong branch"),
        }
    }
}
