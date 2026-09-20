//! Code from `docs/comparisons/reflection.md` — *Reflection and compile-time introspection*.
//!
//! The page's worked example is a self-contained field writer modeled on `cgp-serde`'s
//! `SerializeFields` provider, so it can be compiled here without a `serde` dependency.

/// ## A type's shape becomes a type, not a descriptor
pub mod a_types_shape_becomes_a_type {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Config {
        pub host: String,
        pub port: u16,
    }

    #[test]
    fn the_shape_is_a_type_level_list() {
        fn assert_fields<T: HasFields<Fields = Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>]>>(
        ) {
        }
        assert_fields::<Config>();
    }
}

/// ## Reflection-driven serialization in the trait system
pub mod reflection_driven_serialization {
    use core::fmt::Debug;

    use cgp::core::field::traits::StaticString;
    use cgp::prelude::*;

    #[cgp_component(ValueWriter)]
    pub trait CanWriteValue<Value> {
        fn write_value(&self, value: &Value) -> String;
    }

    #[cgp_impl(new WriteWithDebug)]
    impl<Value: Debug> ValueWriter<Value> {
        fn write_value(&self, value: &Value) -> String {
            format!("{value:?}")
        }
    }

    #[cgp_impl(new WriteFields)]
    impl<Value> ValueWriter<Value>
    where
        Value: HasFields,
        Value::Fields: FieldsWriter<Self, Value>,
    {
        fn write_value(&self, value: &Value) -> String {
            format!("{{{}}}", Value::Fields::write_fields(self, value))
        }
    }

    pub trait FieldsWriter<Context, Value> {
        fn write_fields(context: &Context, value: &Value) -> String;
    }

    impl<Context, Value, Tag, FieldValue, Rest> FieldsWriter<Context, Value>
        for Cons<Field<Tag, FieldValue>, Rest>
    where
        Tag: StaticString,                        // the field name, as a const &'static str
        Value: HasField<Tag, Value = FieldValue>, // read this field from the value
        Context: CanWriteValue<FieldValue>,       // write the field through the context's wiring
        Rest: FieldsWriter<Context, Value>,       // recurse on the remaining fields
    {
        fn write_fields(context: &Context, value: &Value) -> String {
            let field_value = value.get_field(PhantomData);
            let entry = format!("\"{}\": {}", Tag::VALUE, context.write_value(field_value));
            let rest = Rest::write_fields(context, value);
            if rest.is_empty() {
                entry
            } else {
                format!("{entry}, {rest}")
            }
        }
    }

    impl<Context, Value> FieldsWriter<Context, Value> for Nil {
        fn write_fields(_context: &Context, _value: &Value) -> String {
            String::new()
        }
    }

    #[derive(HasField, HasFields)]
    pub struct Config {
        pub host: String,
        pub port: u16,
    }

    pub struct App;

    delegate_components! {
        App {
            open ValueWriterComponent;

            @ValueWriterComponent.[String, u16]: WriteWithDebug,
            @ValueWriterComponent.Config: WriteFields,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { ValueWriterComponent: Config } }
    }

    #[test]
    fn one_writer_walks_any_derived_struct() {
        let config = Config {
            host: "localhost".to_owned(),
            port: 8080,
        };
        assert_eq!(
            App.write_value(&config),
            r#"{"host": "localhost", "port": 8080}"#
        );
    }
}
