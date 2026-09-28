//! Code from `docs/reference/traits/builder/partial_data.md` — `PartialData`.
//!
//! Pins the Examples program: a generic function that names the destination of a partial value at
//! two different configurations.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, BuildField)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    pub fn target_name<Partial>(_partial: &Partial) -> &'static str
    where
        Partial: PartialData,
    {
        core::any::type_name::<Partial::Target>()
    }

    pub fn demo() {
        let empty = Person::builder();
        assert!(target_name(&empty).ends_with("Person"));

        let half = empty.build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());
        assert!(target_name(&half).ends_with("Person"));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
