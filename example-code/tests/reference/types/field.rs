//! Code from `docs/reference/types/field.md` — *`Field`*.
//!
//! A `Field` pairs a value with a type-level tag. This pins the `Fields` shape a derive assigns, where
//! each entry is a `Field`, and the `.into()` construction the page shows.

/// ## Examples
///
/// Each entry of a struct's shape is a `Field` tagged by its name; naming the shape checks the mapping.
/// A single `Field` is built from its value with `.into()`, the tag supplied by the type annotation,
/// and a tuple-struct field is tagged by an `Index` rather than a `Symbol!`.
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    pub type ExpectedFields = Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>];

    pub fn assert_fields(fields: <Person as HasFields>::Fields) -> ExpectedFields {
        fields
    }

    // The positional form the page names: `Index<0>` in place of a `Symbol!`.
    pub type PositionalField = Field<Index<0>, u32>;

    #[test]
    fn test_a_field_is_built_from_its_value() {
        let name: Field<Symbol!("name"), String> = "Alice".to_string().into();
        assert_eq!(name.value, "Alice");
    }
}
