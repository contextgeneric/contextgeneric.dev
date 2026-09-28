//! Code from `docs/reference/traits/optional/transform_map_default.md` — `TransformMapDefault`.
//!
//! Pins the Examples program: a generic function that drives the marker across a builder with
//! `TransformMapFields`, then finalizes the all-present result.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::TransformMapFields;
    use cgp::extra::field::impls::TransformMapDefault;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Context {
        pub foo: String,
        pub bar: u64,
    }

    pub fn fill_defaults<Builder>(builder: Builder) -> Builder::Output
    where
        Builder: TransformMapFields<TransformMapDefault, IsPresent>,
    {
        builder.transform_map_fields()
    }

    pub fn demo() {
        let full = fill_defaults(
            Context::builder().build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned()),
        );
        let context = full.finalize_build();
        assert_eq!(context.bar, 0);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
