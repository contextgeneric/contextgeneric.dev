//! Code from `docs/reference/traits/field-access/map_field.md` — `MapField`.
//!
//! Pins the Examples program: a generic function that reads a field of a field with `map_field`,
//! and the `ChainGetters` wiring that performs the same descent. The same read written as two
//! chained `get_field` calls is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::ChainGetters;
    use cgp::core::field::traits::MapField;
    use cgp::prelude::*;

    pub fn inner_name<Context, Inner>(context: &Context) -> &String
    where
        Context: HasField<Symbol!("inner"), Value = Inner>,
        Inner: HasField<Symbol!("name"), Value = String>,
    {
        context.map_field(PhantomData::<Symbol!("inner")>, |inner| {
            inner.get_field(PhantomData::<Symbol!("name")>)
        })
    }

    #[cgp_getter(NameGetter)]
    pub trait HasName {
        fn name(&self) -> &String;
    }

    #[derive(HasField)]
    pub struct Inner {
        pub name: String,
    }

    #[derive(HasField)]
    pub struct Outer {
        pub inner: Inner,
    }

    delegate_components! {
        Outer {
            NameGetterComponent: WithProvider<
                ChainGetters<Product![
                    UseField<Symbol!("inner")>,
                    UseField<Symbol!("name")>,
                ]>,
            >,
        }
    }

    check_components! {
        Outer {
            NameGetterComponent,
        }
    }

    pub fn demo() {
        let outer = Outer {
            inner: Inner {
                name: "Alice".to_owned(),
            },
        };

        assert_eq!(inner_name(&outer), "Alice");
        assert_eq!(outer.name(), "Alice");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
