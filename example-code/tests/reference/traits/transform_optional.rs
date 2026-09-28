//! Code from `docs/reference/traits/optional/transform_optional.md` — `TransformOptional`.
//!
//! Pins the Examples program: a generic function that drives the marker across a builder with
//! `TransformMapFields`, turning a partly built core builder into an optional one.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::IsOptional;
    use cgp::core::field::traits::TransformMapFields;
    use cgp::extra::field::impls::{FinalizeOptional, SetOptional, TransformOptional};
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Context {
        pub foo: String,
        pub bar: u64,
    }

    pub fn relax<Builder>(builder: Builder) -> Builder::Output
    where
        Builder: TransformMapFields<TransformOptional, IsOptional>,
    {
        builder.transform_map_fields()
    }

    pub fn demo() {
        let optional =
            relax(Context::builder().build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned()));
        let context = optional
            .set(PhantomData::<Symbol!("bar")>, 42)
            .finalize_optional()
            .unwrap();
        assert_eq!(context.foo, "foo");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
