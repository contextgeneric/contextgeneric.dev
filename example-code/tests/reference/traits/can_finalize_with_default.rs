//! Code from `docs/reference/traits/optional/can_finalize_with_default.md` — `CanFinalizeWithDefault`.
//!
//! Pins the Examples program: an optional builder and a core builder, each finalized with the unset
//! field defaulted. A field whose type has no `Default` is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::extra::field::impls::{CanFinalizeWithDefault, HasOptionalBuilder, SetOptional};
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Context {
        pub foo: String,
        pub bar: u64,
    }

    pub fn demo() {
        let from_optional = Context::optional_builder()
            .set(PhantomData::<Symbol!("foo")>, "foo".to_owned())
            .finalize_with_default();
        assert_eq!(from_optional.bar, 0);

        // A core builder works too: its absent field is `IsNothing` rather than `None`.
        let from_core = Context::builder()
            .build_field(PhantomData::<Symbol!("foo")>, "foo".to_owned())
            .finalize_with_default();
        assert_eq!(from_core, from_optional);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
