//! Code from `docs/reference/traits/formatting/static_string.md` — `StaticString`.
//!
//! Pins the Examples program: a symbol decoded eagerly and lazily, multi-byte and empty symbols, and
//! a generic function naming its tag. `StaticString` on a bare `Chars` list is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::StaticString;
    use cgp::prelude::*;

    // A generic routine recovers the name of whatever tag it is given.
    pub fn field_name<Tag: StaticString>(_tag: PhantomData<Tag>) -> &'static str {
        Tag::VALUE
    }

    pub fn demo() {
        // eagerly, as a compile-time constant
        assert_eq!(<Symbol!("hello") as StaticString>::VALUE, "hello");

        // lazily, through `Display`: reconstructed at the point of formatting
        let s = <Symbol!("hello")>::default();
        assert_eq!(s.to_string(), "hello");

        assert_eq!(<Symbol!("世界你好") as StaticString>::VALUE, "世界你好");
        assert_eq!(<Symbol!("") as StaticString>::VALUE, "");

        assert_eq!(field_name(PhantomData::<Symbol!("height")>), "height");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
