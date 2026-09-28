//! Code from `docs/reference/traits/field-access/field_mapper.md` — `FieldMapper`.
//!
//! Pins the Examples program: a two-step chain provider written with `FieldMapper::map_field`,
//! wired as a getter, and a direct `map_field` call through `UseContext`.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::FieldMapper;
    use cgp::prelude::*;

    pub struct ThenGet<First, Second>(pub PhantomData<(First, Second)>);

    impl<Context, Tag, First, Second, Mid, Value> FieldGetter<Context, Tag> for ThenGet<First, Second>
    where
        First: FieldMapper<Context, Tag, Value = Mid>,
        Second: FieldGetter<Mid, Tag, Value = Value>,
    {
        type Value = Value;

        fn get_field(context: &Context, tag: PhantomData<Tag>) -> &Value {
            First::map_field(context, tag, |mid| Second::get_field(mid, tag))
        }
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
            NameGetterComponent:
                WithProvider<ThenGet<UseField<Symbol!("inner")>, UseField<Symbol!("name")>>>,
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

        assert_eq!(outer.name(), "Alice");

        let name = <UseContext as FieldMapper<Outer, Symbol!("inner")>>::map_field(
            &outer,
            PhantomData,
            |inner| inner.get_field(PhantomData::<Symbol!("name")>),
        );
        assert_eq!(name, "Alice");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
