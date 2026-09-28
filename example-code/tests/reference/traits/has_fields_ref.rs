//! Code from `docs/reference/traits/shape/has_fields_ref.md` — `HasFieldsRef`.
//!
//! Pins the Examples program: the borrowed shape of a struct at a given lifetime, including a
//! field that is itself a reference and so gains a second one.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Config {
        pub host: String,
        pub port: u16,
    }

    #[derive(HasFields)]
    pub struct Greeting<'a> {
        pub name: &'a str,
    }

    pub fn assert_shape_ref<'a, T, FieldsRef>()
    where
        T: HasFieldsRef<FieldsRef<'a> = FieldsRef> + 'a,
    {
    }

    pub fn demo() {
        assert_shape_ref::<
            'static,
            Config,
            Product![
                Field<Symbol!("host"), &'static String>,
                Field<Symbol!("port"), &'static u16>,
            ],
        >();
        assert_shape_ref::<
            'static,
            Greeting<'static>,
            Product![
                Field<Symbol!("name"), &'static &'static str>,
            ],
        >();
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
