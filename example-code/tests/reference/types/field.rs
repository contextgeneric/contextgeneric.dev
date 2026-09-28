//! Code from `docs/reference/types/field.md` — *`Field`*.
//!
//! A `Field` pairs a value with a type-level tag. The Examples program reads a struct's shape as a
//! list of `Field` entries, builds one entry with `.into()`, shows that the tag adds no size and no
//! output, and shows a one-field tuple struct, whose shape is its field's type with no `Field` around
//! it.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    #[derive(HasFields)]
    pub struct Point(pub u32, pub u32);

    #[derive(HasFields)]
    pub struct Meters(pub u32);

    pub fn demo() {
        // A struct's shape is a list of `Field` entries, each tagged by its name.
        let fields: Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>] = Person {
            name: "Alice".to_owned(),
            age: 30,
        }
        .to_fields();

        let Cons(name, Cons(age, Nil)) = fields;
        assert_eq!(name.value, "Alice");
        assert_eq!(age.value, 30);

        // A tuple struct's entries are tagged by position.
        let Cons(x, Cons(y, Nil)): Product![Field<Index<0>, u32>, Field<Index<1>, u32>] =
            Point(3, 4).to_fields();
        assert_eq!((x.value, y.value), (3, 4));

        // A one-field tuple struct's shape is the field's type itself.
        let inner: u32 = Meters(7).to_fields();
        assert_eq!(inner, 7);

        // One entry, built from its value; the annotation supplies the tag.
        let entry: Field<Symbol!("name"), String> = "Bob".to_owned().into();
        assert_eq!(entry.value, "Bob");

        // The tag adds no size, and `Debug` prints the value alone.
        assert_eq!(
            core::mem::size_of::<Field<Symbol!("name"), String>>(),
            core::mem::size_of::<String>()
        );
        assert_eq!(format!("{entry:?}"), "\"Bob\"");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
