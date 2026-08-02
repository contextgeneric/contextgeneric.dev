//! Code from `docs/reference/traits/static_string.md` — *`StaticString`*.
//!
//! The page's snippets, run. The load-bearing check is the import path — the trait is not in the
//! prelude — and that the eager and lazy routes agree, including on multi-byte and empty symbols.

/// ## Examples
pub mod examples {
    #[test]
    fn test_both_recovery_routes() {
        use cgp::core::field::traits::StaticString;
        use cgp::prelude::*;

        // eagerly, as a compile-time constant
        assert_eq!(<Symbol!("hello") as StaticString>::VALUE, "hello");

        // lazily, through Display
        let s = <Symbol!("hello")>::default();
        assert_eq!(s.to_string(), "hello");
    }

    #[test]
    fn test_multi_byte_and_empty() {
        use cgp::core::field::traits::StaticString;
        use cgp::prelude::*;

        assert_eq!(<Symbol!("世界你好") as StaticString>::VALUE, "世界你好");
        assert_eq!(<Symbol!("") as StaticString>::VALUE, "");
    }
}
