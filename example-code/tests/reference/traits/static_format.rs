//! Code from `docs/reference/traits/formatting/static_format.md` — `StaticFormat`.
//!
//! Pins the Examples program: formatting a type-level string through `Display`, a `Display` bound in
//! generic code, and the narrow case for the trait itself, a wrapper with no symbol value to format.

/// ## Examples
pub mod examples {
    use core::fmt::{self, Display, Formatter};

    use cgp::core::base::traits::StaticFormat;
    use cgp::prelude::*;

    // A `Display` bound is all that formatting a name needs.
    pub fn describe<Tag: Default + Display>(_tag: PhantomData<Tag>) -> String {
        format!("missing field `{}`", Tag::default())
    }

    // A wrapper holds only `PhantomData`, so it formats its tag through the trait.
    pub struct FieldName<Tag>(pub PhantomData<Tag>);

    impl<Tag: StaticFormat> Display for FieldName<Tag> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            Tag::fmt(f)
        }
    }

    pub fn demo() {
        let s = <Symbol!("hello")>::default();
        assert_eq!(s.to_string(), "hello");
        assert_eq!(format!("field: {s}"), "field: hello");

        assert_eq!(
            describe(PhantomData::<Symbol!("height")>),
            "missing field `height`"
        );

        let name = FieldName(PhantomData::<Symbol!("width")>);
        assert_eq!(format!("[{name}]"), "[width]");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
