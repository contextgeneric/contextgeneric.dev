//! Code from `docs/reference/types/index_type.md` — *`Index`*.
//!
//! `Index<N>` tags a tuple-struct field by its position. The page reads a field by its index and prints
//! the number a tag stands for; both run here.

/// ## Examples
///
/// Deriving `HasField` on a tuple struct is what makes the positional `get_field` below resolve, and an
/// `Index` prints its number through `Display`.
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasField)]
    pub struct Pair(pub u32, pub String);

    #[test]
    fn test_a_tuple_field_is_read_by_its_index() {
        let pair = Pair(7, "hi".to_string());
        assert_eq!(*pair.get_field(PhantomData::<Index<0>>), 7);
    }

    #[test]
    fn test_an_index_prints_its_number() {
        assert_eq!(Index::<2>.to_string(), "2");
    }
}
