//! Code from `docs/reference/macros/struct.md` — *`Struct!`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`. Type
//! equalities are checked by assigning one `PhantomData` to a binding typed with the other.

/// ## Overview
pub mod overview {
    use cgp::prelude::*;

    #[test]
    fn the_shape_is_the_hand_written_field_list() {
        let _: PhantomData<Struct! { name: String, age: u8 }> =
            PhantomData::<Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>]>;
    }
}

/// ## Usage
pub mod usage {
    use cgp::prelude::*;

    #[test]
    fn the_form_comes_from_the_entries() {
        let _: PhantomData<Struct!(a: u8)> = PhantomData::<Struct! { a: u8 }>;
        let _: PhantomData<Struct! { u8, bool }> = PhantomData::<Struct!(u8, bool)>;
        let _: PhantomData<Struct!(core::marker::PhantomData<u8>, u8)> =
            PhantomData::<Product![Field<Index<0>, PhantomData<u8>>, Field<Index<1>, u8>]>;
    }

    #[test]
    fn trailing_commas_and_raw_names() {
        let _: PhantomData<Struct! { a: u8, }> = PhantomData::<Struct! { a: u8 }>;
        let _: PhantomData<Struct! { r#type: u8 }> =
            PhantomData::<Product![Field<Symbol!("type"), u8>]>;
        let _: PhantomData<Struct! {}> = PhantomData::<Nil>;
    }

    #[test]
    fn the_bodies_that_follow_the_derive() {
        let _: PhantomData<Struct!(u64)> = PhantomData::<u64>;
        let _: PhantomData<Struct!()> = PhantomData::<Nil>;
        let _: PhantomData<Struct! { value: u64 }> =
            PhantomData::<Product![Field<Symbol!("value"), u64>]>;
    }

    #[derive(Debug, PartialEq, HasFields)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    #[test]
    fn a_value_comes_from_to_fields_or_product() {
        let from_struct: Struct! { name: String, age: u8 } = Person {
            name: "Ada".to_owned(),
            age: 36,
        }
        .to_fields();
        let built: Struct! { name: String, age: u8 } =
            product!["Ada".to_owned().into(), 36u8.into()];
        assert_eq!(from_struct, built);
    }
}

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, HasFields)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    pub fn rebuild<T>(fields: Struct! { name: String, age: u8 }) -> T
    where
        T: FromFields<Fields = Struct! { name: String, age: u8 }>,
    {
        T::from_fields(fields)
    }

    #[test]
    fn rebuild_turns_a_shape_into_the_struct() {
        let person: Person = rebuild(product!["Carol".to_owned().into(), 25u8.into()]);
        assert_eq!(person.age, 25);
    }

    pub trait Describe {
        fn describe() -> &'static str;
    }

    impl Describe for Struct! { name: String, age: u8 } {
        fn describe() -> &'static str {
            "a person's shape"
        }
    }

    impl Describe for Struct!(u8, u16) {
        fn describe() -> &'static str {
            "a pair's shape"
        }
    }

    #[test]
    fn a_trait_is_implemented_for_a_shape() {
        assert_eq!(<<Person as HasFields>::Fields as Describe>::describe(), "a person's shape");
        assert_eq!(<Struct!(u8, u16) as Describe>::describe(), "a pair's shape");
    }

    // The page elides the component and providers behind the `open` entries.
    #[cgp_component(ShapeDescriber)]
    pub trait CanDescribeShape<Shape> {
        fn describe_shape(&self) -> &'static str;
    }

    #[cgp_impl(new DescribePoint)]
    impl<Shape> ShapeDescriber<Shape> {
        fn describe_shape(&self) -> &'static str {
            "point"
        }
    }

    #[cgp_impl(new DescribePair)]
    impl<Shape> ShapeDescriber<Shape> {
        fn describe_shape(&self) -> &'static str {
            "pair"
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ShapeDescriberComponent;

            @ShapeDescriberComponent.Struct! { x: f64, y: f64 }: DescribePoint,
            @ShapeDescriberComponent.Struct!(u8, u16): DescribePair,
        }
    }

    #[test]
    fn a_shape_keys_an_open_entry() {
        assert_eq!(CanDescribeShape::<Struct! { x: f64, y: f64 }>::describe_shape(&App), "point");
        assert_eq!(CanDescribeShape::<Struct!(u8, u16)>::describe_shape(&App), "pair");
    }
}

/// ## Under the hood
pub mod under_the_hood {
    use cgp::prelude::*;

    #[test]
    fn named_and_positional_expansions() {
        let _: PhantomData<Struct! { name: String, age: u8 }> = PhantomData::<
            Cons<Field<Symbol!("name"), String>, Cons<Field<Symbol!("age"), u8>, Nil>>,
        >;
        let _: PhantomData<Struct!(u64, String)> =
            PhantomData::<Cons<Field<Index<0>, u64>, Cons<Field<Index<1>, String>, Nil>>>;
    }

    #[test]
    fn the_newtype_and_empty_rules() {
        let _: PhantomData<Struct!(u64)> = PhantomData::<u64>;
        let _: PhantomData<Struct! {}> = PhantomData::<Nil>;
        let _: PhantomData<Struct! { value: u64 }> =
            PhantomData::<Cons<Field<Symbol!("value"), u64>, Nil>>;
    }

    mod without_imports {
        pub type Point = cgp::prelude::Struct! { x: f64, y: f64 };
    }

    #[test]
    fn the_expansion_needs_no_imports() {
        let _: PhantomData<without_imports::Point> = PhantomData::<Struct! { x: f64, y: f64 }>;
    }
}

/// ## Common Mistakes
pub mod common_mistakes {
    use cgp::prelude::*;

    #[test]
    fn a_one_element_positional_list_is_written_with_product() {
        let _: PhantomData<Struct!(u64,)> = PhantomData::<u64>;
        let _: PhantomData<Product![Field<Index<0>, u64>]> =
            PhantomData::<Cons<Field<Index<0>, u64>, Nil>>;
    }
}
