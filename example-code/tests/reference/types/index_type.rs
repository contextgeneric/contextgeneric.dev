//! Code from `docs/reference/types/index_type.md` — *`Index`*.
//!
//! `Index<N>` tags a tuple-struct field by its position. The Examples program reads both fields of a
//! tuple struct by their indices and prints the number an `Index` stands for. A position the struct
//! lacks, and a `Symbol!` of a digit, are trybuild fixtures under
//! `tests/compile_fail/reference/types/`.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasField)]
    pub struct Pair(pub u32, pub String);

    pub fn demo() {
        let pair = Pair(7, "hi".to_owned());

        assert_eq!(*pair.get_field(PhantomData::<Index<0>>), 7);
        assert_eq!(pair.get_field(PhantomData::<Index<1>>), "hi");

        // `Display` and `Debug` both print the number.
        assert_eq!(Index::<2>.to_string(), "2");
        assert_eq!(format!("{:?}", Index::<2>), "2");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
