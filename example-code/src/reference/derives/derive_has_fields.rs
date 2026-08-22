//! Code from `docs/reference/derives/derive_has_fields.md` — *`#[derive(HasFields)]`*.
//!
//! The page's *Under the hood* section shows the five impls the derive generates; those are not
//! repeated here, since re-declaring them beside the derive would be a coherence conflict and
//! `cargo cgp expand` is the check on that section. What this file pins is the `Fields` type the page
//! claims for each input shape, asserted by round-tripping a value through it.

/// ## Overview
///
/// The `Fields` type the page opens on, checked by naming it explicitly.
pub mod what_its_for {
    use cgp::prelude::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    #[derive(HasFields)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    // The page writes this type out; asserting the equality is what checks it.
    pub type ExpectedFields =
        Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>];

    pub fn assert_fields(fields: <Person as HasFields>::Fields) -> ExpectedFields {
        fields
    }

    #[test]
    fn test_round_trip() {
        let person1 = Person {
            name: "Alice".to_owned(),
            age: 30,
        };

        let fields = person1.clone().to_fields();
        let person2 = Person::from_fields(fields);

        assert_eq!(person1, person2);
    }
}

/// ## Using it
///
/// Every struct and enum shape the page lists. The four struct shapes come first, then the four
/// variant shapes in the single enum the page uses for them.
pub mod using_it {
    use cgp::prelude::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    #[derive(HasFields)]
    pub struct Named {
        pub name: String,
        pub age: u8,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    #[derive(HasFields)]
    pub struct Tuple(pub u32, pub u32);

    #[derive(Clone, Debug, Eq, PartialEq)]
    #[derive(HasFields)]
    pub struct Unit;

    /// The newtype special case: `Fields` is the inner type directly.
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[derive(HasFields)]
    pub struct Newtype(pub String);

    #[derive(Clone, Debug, Eq, PartialEq)]
    #[derive(HasFields)]
    pub struct Generic<T> {
        pub value: T,
    }

    /// The four variant shapes, all accepted by this derive and by no other in the family.
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[derive(HasFields)]
    pub enum Shape {
        Empty,
        Circle(u32),
        Rectangle(u32, u32),
        Triangle { base: u32, height: u32 },
    }

    // The `Fields` type the page's table claims for each variant, written out so the compiler checks
    // the mapping rather than the reader checking it by eye.
    pub type ExpectedShapeFields = Sum![
        Field<Symbol!("Empty"), Nil>,
        Field<Symbol!("Circle"), u32>,
        Field<Symbol!("Rectangle"), Product![Field<Index<0>, u32>, Field<Index<1>, u32>]>,
        Field<
            Symbol!("Triangle"),
            Product![Field<Symbol!("base"), u32>, Field<Symbol!("height"), u32>],
        >,
    ];

    pub fn assert_shape_fields(fields: <Shape as HasFields>::Fields) -> ExpectedShapeFields {
        fields
    }

    #[test]
    fn test_a_unit_struct_has_the_empty_product() {
        let unit1 = Unit;

        let fields = unit1.clone().to_fields();
        assert_eq!(fields, Nil);

        assert_eq!(Unit::from_fields(fields), unit1);
    }

    #[test]
    fn test_a_newtype_passes_its_inner_type_through() {
        let newtype1 = Newtype("Alice".to_owned());

        // Not a one-element product: the field list *is* the `String`.
        let fields: String = newtype1.clone().to_fields();
        assert_eq!(fields, "Alice");

        assert_eq!(Newtype::from_fields(fields), newtype1);
    }

    #[test]
    fn test_every_struct_shape_round_trips() {
        let named = Named {
            name: "Alice".to_owned(),
            age: 30,
        };
        assert_eq!(Named::from_fields(named.clone().to_fields()), named);

        let tuple = Tuple(3, 4);
        assert_eq!(Tuple::from_fields(tuple.clone().to_fields()), tuple);

        let generic = Generic { value: 42_u32 };
        assert_eq!(Generic::from_fields(generic.clone().to_fields()), generic);
    }

    #[test]
    fn test_every_variant_shape_round_trips() {
        for shape in [
            Shape::Empty,
            Shape::Circle(2),
            Shape::Rectangle(3, 4),
            Shape::Triangle {
                base: 6,
                height: 5,
            },
        ] {
            assert_eq!(Shape::from_fields(shape.clone().to_fields()), shape);
        }
    }
}

/// ## Examples
///
/// The `Config` pairing the page shows, the borrowed form, and the enum whose shape is a sum.
pub mod examples {
    use cgp::prelude::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    #[derive(HasField, HasFields)]
    pub struct Config {
        pub host: String,
        pub port: u16,
    }

    /// The payload types the page names without declaring.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Circle {
        pub radius: f64,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[derive(Clone, Debug, PartialEq)]
    #[derive(HasFields)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    /// The page's closing claim: generic code bounds on the shape rather than on the type, so one
    /// function applies to any record that derives it. The page does not show a recursion, so this
    /// stops at naming the shape.
    pub fn shape_of<T>(value: T) -> T::Fields
    where
        T: HasFields + ToFields,
    {
        value.to_fields()
    }

    #[test]
    fn test_the_page_snippets() {
        let config = Config {
            host: "localhost".to_owned(),
            port: 8080,
        };

        let fields = config.clone().to_fields();
        let config_again = Config::from_fields(fields);
        assert_eq!(config, config_again);

        // The borrowing form, which reads without consuming.
        let fields_ref = config.to_fields_ref();
        assert_eq!(fields_ref.0.value, &"localhost".to_owned());
    }

    #[test]
    fn test_an_enums_shape_is_a_sum() {
        let shape = Shape::Circle(Circle { radius: 2.0 });

        assert_eq!(Shape::from_fields(shape.clone().to_fields()), shape);
    }
}
