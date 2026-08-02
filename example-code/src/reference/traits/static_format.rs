//! Code from `docs/reference/traits/static_format.md` — *`StaticFormat`, `StaticString` &
//! `ConcatPath`*.
//!
//! The page's snippets, run. The load-bearing check here is the import path and reachability claim:
//! `StaticString` resolves from `cgp::core::field::traits` and `ConcatPath` from the prelude, while
//! `StaticFormat` is reachable only through `Display` — the page says the trait cannot be named from
//! the `cgp` crate, and the absence of any `use` for it below is that claim.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[test]
    fn test_both_recovery_routes() {
        use cgp::core::field::traits::StaticString;

        // lazily, through Display
        let s = <Symbol!("hello")>::default();
        assert_eq!(s.to_string(), "hello");

        // eagerly, as a compile-time constant
        assert_eq!(<Symbol!("hello") as StaticString>::VALUE, "hello");
    }

    #[test]
    fn test_multi_byte_and_empty() {
        use cgp::core::field::traits::StaticString;

        assert_eq!(<Symbol!("世界你好") as StaticString>::VALUE, "世界你好");
        assert_eq!(<Symbol!("") as StaticString>::VALUE, "");
    }

    /// `ConcatPath` joins two paths at the type level. Paths are unsized markers, so there is no
    /// value to build — naming the joined type and coercing it to the expected one is the check.
    pub type Outer = Path!(@a.b);
    pub type Inner = Path!(@c.d);

    pub type Joined = <Outer as ConcatPath<Inner>>::Output;

    #[allow(dead_code)]
    pub fn assert_joined(path: PhantomData<&Joined>) -> PhantomData<&Path!(@a.b.c.d)> {
        path
    }
}
