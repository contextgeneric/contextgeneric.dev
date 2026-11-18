//! Code from `docs/reference/traits/optional/has_optional_builder.md` — `HasOptionalBuilder`.
//!
//! Pins the Examples program: an all-optional builder set in any order, finished by each of the two
//! endings.

/// ## Examples
pub mod examples {
    use cgp::extra::field::impls::{
        CanFinalizeWithDefault, FinalizeOptional, HasOptionalBuilder, SetOptional,
    };
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Context {
        pub foo: String,
        pub bar: u64,
    }

    pub fn demo() {
        let context = Context::optional_builder()
            .set(PhantomData::<Symbol!("bar")>, 42)
            .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
            .finalize_optional();
        assert_eq!(
            context,
            Ok(Context {
                foo: "foo".to_owned(),
                bar: 42,
            })
        );

        // The same kind of builder, with `bar` left unset, finalized each way.
        let defaulted = Context::optional_builder()
            .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
            .finalize_with_default();
        assert_eq!(defaulted.bar, 0);

        let missing = Context::optional_builder()
            .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
            .finalize_optional();
        assert_eq!(missing, Err("bar"));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
