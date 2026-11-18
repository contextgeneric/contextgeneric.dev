//! Code from `docs/reference/traits/optional/finalize_optional.md` — `FinalizeOptional`.
//!
//! Pins the Examples program: success, one missing field, and two missing fields, which report the
//! last in declaration order. Finalizing a core builder this way is a trybuild fixture.

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
        let complete = Context::optional_builder()
            .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
            .set(PhantomData::<Symbol!("bar")>, 42)
            .finalize_optional();
        assert_eq!(
            complete,
            Ok(Context {
                foo: "foo".to_owned(),
                bar: 42,
            })
        );

        let missing_bar = Context::optional_builder()
            .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
            .finalize_optional();
        assert_eq!(missing_bar, Err("bar"));

        // With both unset, the walk from the last field back stops at `bar`.
        assert_eq!(Context::optional_builder().finalize_optional(), Err("bar"));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
