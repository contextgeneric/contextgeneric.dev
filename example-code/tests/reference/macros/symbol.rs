//! Code from `docs/reference/macros/symbol.md` — *`Symbol!`*.
//!
//! The snippets the page rejects, and the errors it quotes, live under
//! `tests/compile_fail/reference/macros/`. Type equalities are checked by assigning one
//! `PhantomData` to a binding typed with the other.

/// ## Overview and Usage
///
/// The literal forms the page lists, and the field bound it shows twice.
pub mod overview_and_usage {
    use cgp::prelude::*;

    pub type Name = Symbol!("name");
    pub type FirstName = Symbol!("first_name");
    pub type Empty = Symbol!("");
    pub type Unicode = Symbol!("世界");

    pub fn reads_a_name<Context>(context: &Context) -> &String
    where
        Context: HasField<Symbol!("name"), Value = String>,
    {
        context.get_field(PhantomData::<Symbol!("name")>)
    }

    /// Any string literal the parser accepts decodes to the same type: a raw string and an
    /// escape spell the characters they denote.
    #[test]
    fn every_string_literal_form_is_accepted() {
        let _: PhantomData<Symbol!("a\nb")> = PhantomData::<Symbol!(r"a
b")>;
        let _: PhantomData<Symbol!("raw")> = PhantomData::<Symbol!(r#"raw"#)>;
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    #[test]
    fn the_tag_selects_the_field() {
        let person = Person {
            name: "Ada".to_owned(),
        };
        assert_eq!(reads_a_name(&person), "Ada");
    }
}

/// ### The tag for a tuple field
pub mod the_tag_for_a_tuple_field {
    use cgp::prelude::*;

    #[derive(HasField)]
    pub struct Point(pub f64, pub f64);

    #[test]
    fn a_position_is_keyed_by_index() {
        let point = Point(1.0, 2.0);
        assert_eq!(*point.get_field(PhantomData::<Index<1>>), 2.0);
    }
}

/// ### Raw identifiers
pub mod raw_identifiers {
    use cgp::prelude::*;

    #[derive(HasField)]
    pub struct Token {
        pub r#type: u8,
    }

    #[test]
    fn the_raw_prefix_is_stripped() {
        let token = Token { r#type: 7 };
        assert_eq!(*token.get_field(PhantomData::<Symbol!("type")>), 7);
    }
}

/// ## Examples
///
/// The getter wired to a differently-named field, the hand-written field bound, and the runtime
/// string. The page names `Greeter` without declaring it.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct Person {
        pub first_name: String,
    }

    delegate_components! {
        Person {
            NameGetterComponent: UseField<Symbol!("first_name")>,
        }
    }

    check_components! {
        Person {
            NameGetterComponent,
        }
    }

    #[test]
    fn the_tag_names_the_field_to_read() {
        let person = Person {
            first_name: "Ada".to_owned(),
        };
        assert_eq!(person.name(), "Ada");
    }

    pub mod hand_written_bound {
        use cgp::prelude::*;

        #[cgp_component(Greeter)]
        pub trait CanGreet {
            fn greet(&self);
        }

        #[cgp_impl(new GreetHello)]
        impl Greeter
        where
            Self: HasField<Symbol!("name"), Value = String>,
        {
            fn greet(&self) {
                println!("Hello, {}!", self.get_field(PhantomData::<Symbol!("name")>));
            }
        }

        #[derive(HasField)]
        pub struct Person {
            pub name: String,
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
    }

    #[test]
    fn a_type_level_string_prints_its_text() {
        let s = <Symbol!("hello")>::default();
        assert_eq!(s.to_string(), "hello");
    }

    #[test]
    fn a_type_level_string_is_also_a_constant() {
        use cgp::core::field::traits::StaticString;

        assert_eq!(<Symbol!("hello") as StaticString>::VALUE, "hello");
    }
}

/// ## Under the hood
///
/// The expansion the page shows, and the byte length a multi-byte string records.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[test]
    fn the_expansion_is_a_length_and_a_character_list() {
        let _: PhantomData<Symbol!("abc")> =
            PhantomData::<Symbol<3, Chars<'a', Chars<'b', Chars<'c', Nil>>>>>;
        let _: PhantomData<Symbol!("")> = PhantomData::<Symbol<0, Nil>>;
        let _: PhantomData<Symbol!("世界你好")> =
            PhantomData::<Symbol<12, Chars<'世', Chars<'界', Chars<'你', Chars<'好', Nil>>>>>>;
    }
}
