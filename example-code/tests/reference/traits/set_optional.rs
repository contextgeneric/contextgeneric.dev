//! Code from `docs/reference/traits/optional/set_optional.md` — `SetOptional`.
//!
//! Pins the Examples program: fields set in any order, and one re-set with the replaced value returned.

/// ## Examples
pub mod examples {
    use cgp::extra::field::impls::{FinalizeOptional, HasOptionalBuilder, SetOptional};
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Context {
        pub foo: String,
        pub bar: u64,
    }

    pub fn demo() {
        let builder = Context::optional_builder()
            .set(PhantomData::<Symbol!("bar")>, 42)
            .set(PhantomData::<Symbol!("foo")>, "foo".to_owned());

        // Setting an already-set field hands back what it replaced.
        let (replaced, builder) =
            builder.set_optional(PhantomData::<Symbol!("foo")>, "bar".to_owned());
        assert_eq!(replaced, Some("foo".to_owned()));

        let context = builder.finalize_optional().unwrap();
        assert_eq!(context.foo, "bar");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
