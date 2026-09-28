//! Code from `docs/reference/traits/builder/finalize_build.md` — `FinalizeBuild`.
//!
//! Pins the Examples program: a generic finalize over any complete builder, including a fieldless
//! record. The incomplete build is the `has_builder` trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, BuildField)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(Debug, PartialEq, BuildField)]
    pub struct Empty {}

    pub fn assemble<Partial, Target>(partial: Partial) -> Target
    where
        Partial: FinalizeBuild<Target = Target>,
    {
        partial.finalize_build()
    }

    pub fn demo() {
        let person: Person = assemble(
            Person::builder()
                .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
                .build_field(PhantomData::<Symbol!("last_name")>, "Chen".to_owned()),
        );
        assert_eq!(person.last_name, "Chen");

        // A fieldless record's builder is complete from the start.
        let empty: Empty = assemble(Empty::builder());
        assert_eq!(empty, Empty {});
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
