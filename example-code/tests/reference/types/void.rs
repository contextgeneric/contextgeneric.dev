//! Code from `docs/reference/types/void.md` — *`Void`*.
//!
//! `Void` is uninhabited, so there is no value to build. Naming the empty `Sum!` as `Void` and coercing
//! it is the check.

/// ## Examples
///
/// An `Either` chain ends in `Void`, and the empty sum is `Void` alone.
pub mod examples {
    use cgp::prelude::*;

    pub type Token = Either<u32, Either<bool, Void>>;

    pub type Empty = Sum![];

    // No value of either type is constructed: the coercion checks the type equality.
    #[allow(dead_code)]
    pub fn assert_empty_is_void(empty: Empty) -> Void {
        empty
    }
}
