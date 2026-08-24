//! Code from `docs/reference/types/mref.md` — *`MRef`*.
//!
//! `MRef` is an ordinary runtime value, not a type-level marker. The page builds one both ways and
//! promotes it to an owned value; this runs that.

/// ## Examples
///
/// A borrowed value and a freshly built one have the same type and are read the same way, and
/// `get_or_clone` promotes either to ownership.
pub mod examples {
    use cgp::prelude::*;

    #[test]
    fn test_borrowed_and_owned_are_read_the_same_way() {
        let stored = String::from("hello");

        // a context lending a stored value:
        let borrowed: MRef<'_, String> = MRef::from(&stored);
        assert_eq!(&*borrowed, "hello");

        // a provider returning a freshly built value through the same type:
        let made: MRef<'_, String> = MRef::from(String::from("world"));
        assert_eq!(made.as_ref(), "world");

        // promote either to an owned value when ownership is required:
        let owned: String = borrowed.get_or_clone();
        assert_eq!(owned, "hello");
    }
}
