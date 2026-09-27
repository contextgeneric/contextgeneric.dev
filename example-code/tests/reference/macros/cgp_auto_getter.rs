//! Code from `docs/reference/macros/cgp_auto_getter.md` — *`#[cgp_auto_getter]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`.

/// ## Overview
///
/// The opening getter and a context with the field it reads.
pub mod overview {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    #[test]
    fn any_struct_with_the_field_gets_the_getter() {
        let person = Person {
            name: "Ada".to_owned(),
        };
        assert_eq!(person.name(), "Ada");
    }
}

/// ## Usage
///
/// The two-method trait, each method reading its own field.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasDimensions {
        fn width(&self) -> &f64;
        fn height(&self) -> &f64;
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[test]
    fn each_method_reads_its_field() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(*rect.width() * *rect.height(), 6.0);
    }
}

/// ### How the return type decides the read
///
/// One getter per row of the table, then the mutable mirrors, which the page lists in prose.
pub mod how_the_return_type_decides_the_read {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasForms {
        fn by_ref(&self) -> &u32;
        fn text(&self) -> &str;
        fn bytes(&self) -> &[u8];
        fn maybe(&self) -> Option<&u32>;
        fn maybe_text(&self) -> Option<&str>;
        fn borrowed(&self) -> MRef<'_, u32>;
        fn owned(&self) -> (u8, u8);
    }

    #[cgp_auto_getter]
    pub trait HasMutableForms {
        fn by_ref(&mut self) -> &mut u32;
        fn text(&mut self) -> &mut str;
        fn bytes(&mut self) -> &mut [u8];
        fn maybe(&mut self) -> Option<&mut u32>;
        fn maybe_text(&mut self) -> Option<&mut str>;
    }

    #[derive(HasField)]
    pub struct Forms {
        pub by_ref: u32,
        pub text: String,
        pub bytes: Vec<u8>,
        pub maybe: Option<u32>,
        pub maybe_text: Option<String>,
        pub borrowed: u32,
        pub owned: (u8, u8),
    }

    #[test]
    fn each_form_reads_as_the_table_says() {
        let mut forms = Forms {
            by_ref: 1,
            text: "t".to_owned(),
            bytes: vec![1, 2],
            maybe: Some(3),
            maybe_text: Some("m".to_owned()),
            borrowed: 4,
            owned: (5, 6),
        };
        assert_eq!(*HasForms::by_ref(&forms), 1);
        assert_eq!(HasForms::text(&forms), "t");
        assert_eq!(HasForms::bytes(&forms), &[1, 2]);
        assert_eq!(HasForms::maybe(&forms), Some(&3));
        assert_eq!(HasForms::maybe_text(&forms), Some("m"));
        assert!(matches!(forms.borrowed(), MRef::Ref(&4)));
        assert_eq!(forms.owned(), (5, 6));

        *HasMutableForms::by_ref(&mut forms) += 1;
        HasMutableForms::text(&mut forms).make_ascii_uppercase();
        HasMutableForms::bytes(&mut forms)[0] = 9;
        *HasMutableForms::maybe(&mut forms).unwrap() += 1;
        HasMutableForms::maybe_text(&mut forms).unwrap().make_ascii_uppercase();
        assert_eq!(forms.by_ref, 2);
        assert_eq!(forms.text, "T");
        assert_eq!(forms.bytes, vec![9, 2]);
        assert_eq!(forms.maybe, Some(4));
        assert_eq!(forms.maybe_text, Some("M".to_owned()));
    }
}

/// ### Reading a field of another type
///
/// The typed-reference getter. The page names `HasFooType` and `HasBarType` without declaring
/// them; they are abstract types here, and the field `foo_bar` lives on the `Foo` type.
pub mod reading_a_field_of_another_type {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasFooType {
        type Foo;
    }

    #[cgp_type]
    pub trait HasBarType {
        type Bar;
    }

    #[cgp_auto_getter]
    pub trait HasFooBar: HasFooType + HasBarType {
        fn foo_bar(foo: &Self::Foo) -> &Self::Bar;
    }

    #[derive(HasField)]
    pub struct Foo {
        pub foo_bar: u32,
    }

    pub struct App;

    delegate_components! {
        App {
            FooTypeProviderComponent: UseType<Foo>,
            BarTypeProviderComponent: UseType<u32>,
        }
    }

    #[test]
    fn the_getter_reads_the_other_type() {
        let foo = Foo { foo_bar: 7 };
        assert_eq!(*App::foo_bar(&foo), 7);
    }
}

/// ### An optional `PhantomData` argument
///
/// The page names `Foo` as both the tag and the field type; it is a unit struct here.
pub mod an_optional_phantomdata_argument {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    #[derive(Debug, PartialEq)]
    pub struct Foo;

    #[cgp_auto_getter]
    pub trait HasFoo {
        fn foo(&self, _tag: PhantomData<Foo>) -> &Foo;
    }

    #[derive(HasField)]
    pub struct Holder {
        pub foo: Foo,
    }

    #[test]
    fn the_phantom_argument_is_forwarded() {
        assert_eq!(Holder { foo: Foo }.foo(PhantomData), &Foo);
    }
}

/// ### A type inferred from the field
pub mod a_type_inferred_from_the_field {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasName {
        type Name: Display;

        fn name(&self) -> &Self::Name;
    }

    #[derive(HasField)]
    pub struct Numbered {
        pub name: u32,
    }

    #[test]
    fn the_field_decides_the_type() {
        assert_eq!(Numbered { name: 7 }.name().to_string(), "7");
    }
}

/// ### Companion attributes and trait generics
///
/// `#[extend]` and `#[use_type]` apply, becoming context bounds on the impl, and a trait's own
/// generic parameter is carried onto the impl. The page states these in prose.
pub mod companion_attributes_and_trait_generics {
    use core::fmt::Debug;

    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasScalarType {
        type Scalar;
    }

    #[cgp_auto_getter]
    #[use_type(HasScalarType.Scalar)]
    pub trait HasSide {
        fn side(&self) -> &Scalar;
    }

    /// `#[extend]`, which the page states in prose, on a getter of its own.
    #[cgp_auto_getter]
    #[extend(Debug)]
    pub trait HasLabel {
        fn label(&self) -> &str;
    }

    #[cgp_auto_getter]
    pub trait HasValue<T> {
        fn value(&self) -> &T;
    }

    #[derive(Debug, HasField)]
    pub struct Square {
        pub side: f64,
        pub value: u8,
        pub label: String,
    }

    delegate_components! {
        Square {
            ScalarTypeProviderComponent: UseType<f64>,
        }
    }

    #[test]
    fn the_bounds_and_generics_apply() {
        let square = Square {
            side: 2.0,
            value: 3,
            label: "s".to_owned(),
        };
        assert_eq!(*square.side(), 2.0);
        assert_eq!(square.label(), "s");
        assert_eq!(*HasValue::<u8>::value(&square), 3);
    }
}

/// ## Examples
///
/// The page's getter, the provider that depends on it by name, and the hand-written impl for a
/// context that stores the name elsewhere. The provider is wired and checked on `Person` here.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    pub fn greet(person: &Person) {
        println!("Hello, {}!", person.name());
    }

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }

    #[cgp_impl(new GreetHello)]
    #[uses(HasName)]
    impl Greeter {
        fn greet(&self) {
            println!("Hello, {}!", self.name());
        }
    }

    delegate_components! {
        Person {
            GreeterComponent: GreetHello,
        }
    }

    check_components! {
        Person {
            GreeterComponent,
        }
    }

    pub struct Employee {
        pub full_name: String,
    }

    impl HasName for Employee {
        fn name(&self) -> &str {
            &self.full_name
        }
    }

    #[test]
    fn both_contexts_have_names() {
        let person = Person {
            name: "Ada".to_owned(),
        };
        greet(&person);
        person.greet();
        let employee = Employee {
            full_name: "Grace Hopper".to_owned(),
        };
        assert_eq!(employee.name(), "Grace Hopper");
    }
}

/// ## Common Mistakes
///
/// The compiling halves: `#[prefix]` on an auto getter is accepted and registers nothing, the
/// workaround for the `Option` known issue, returning `Option<&mut T>` from a `&mut self` getter,
/// and the two replacements for an `Option<&[T]>` return.
pub mod common_mistakes {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    #[prefix(@app in DefaultNamespace)]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_auto_getter]
    pub trait HasMaybe {
        fn maybe(&mut self) -> Option<&mut u32>;
    }

    /// The two ways around the unsupported `Option<&[T]>` return.
    #[cgp_auto_getter]
    pub trait HasBytes {
        fn bytes(&self) -> Option<&Vec<u8>>;
    }

    #[cgp_auto_getter]
    pub trait HasRawBytes {
        fn raw_bytes(&self) -> &Option<Vec<u8>>;
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
        pub maybe: Option<u32>,
        pub bytes: Option<Vec<u8>>,
        pub raw_bytes: Option<Vec<u8>>,
    }

    #[test]
    fn the_prefix_is_dropped_and_the_workaround_compiles() {
        let mut person = Person {
            name: "Ada".to_owned(),
            maybe: Some(1),
            bytes: Some(vec![1]),
            raw_bytes: None,
        };
        assert_eq!(person.name(), "Ada");
        assert_eq!(person.maybe(), Some(&mut 1));
        assert_eq!(person.bytes().map(|b| b.as_slice()), Some(&[1u8][..]));
        assert_eq!(person.raw_bytes(), &None);
    }
}
