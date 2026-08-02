//! Code from `docs/reference/traits/static_format.md` — *`StaticFormat`*.
//!
//! The page's central claim is a negative one: the trait cannot be named from the `cgp` crate, so its
//! effect is reachable only through `Display`. The absence of any `use` for it below is that claim,
//! and the bound in `describe` is the page's advice about what to require instead.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    pub fn describe<Tag: Default + core::fmt::Display>(_tag: PhantomData<Tag>) -> String {
        format!("missing field `{}`", Tag::default())
    }

    #[test]
    fn test_formatting_through_display() {
        let s = <Symbol!("hello")>::default();

        assert_eq!(s.to_string(), "hello");
        assert_eq!(format!("field: {s}"), "field: hello");
    }

    #[test]
    fn test_a_display_bound_is_what_generic_code_requires() {
        assert_eq!(
            describe(PhantomData::<Symbol!("height")>),
            "missing field `height`",
        );
    }
}
