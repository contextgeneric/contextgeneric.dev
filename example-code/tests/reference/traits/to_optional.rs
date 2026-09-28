//! Code from `docs/reference/traits/optional/to_optional.md` — `ToOptional`.
//!
//! Pins the Examples program: a partly built core builder relaxed to optional, and a complete value
//! relaxed by way of `into_builder`. Converting an already-optional builder is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::extra::field::impls::{FinalizeOptional, SetOptional, ToOptional};
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Context {
        pub foo: String,
        pub bar: u64,
    }

    pub fn demo() {
        // `foo` is set before the conversion and survives it as `Some`.
        let context = Context::builder()
            .build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned())
            .to_optional()
            .set(PhantomData::<Symbol!("bar")>, 42)
            .finalize_optional()
            .unwrap();
        assert_eq!(context.foo, "foo");

        // Every field of a complete value becomes `Some`, so one can be overwritten.
        let context = context
            .into_builder()
            .to_optional()
            .set(PhantomData::<Symbol!("bar")>, 7)
            .finalize_optional()
            .unwrap();
        assert_eq!(context.bar, 7);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
