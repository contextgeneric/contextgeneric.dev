//! Code from `docs/reference/macros/cgp_getter.md` — *`#[cgp_getter]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`.

/// ## Overview
///
/// `Person` wires the getter to a field whose name differs from the method's.
pub mod overview {
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
    fn the_wiring_chooses_the_field() {
        let person = Person {
            first_name: "Ada".to_owned(),
        };
        assert_eq!(person.name(), "Ada");
    }
}

/// ## Usage
///
/// The derived name, the bare override, and the keyed form, each in its own module.
pub mod usage {
    pub mod derived_name {
        use cgp::prelude::*;

        #[cgp_getter]
        pub trait HasName {
            fn name(&self) -> &str;
        }

        #[derive(HasField)]
        pub struct Person {
            pub name: String,
        }

        delegate_components! {
            Person {
                NameGetterComponent: UseFields,
            }
        }

        check_components! {
            Person {
                NameGetterComponent,
            }
        }
    }

    pub mod bare_override {
        use cgp::prelude::*;

        #[cgp_getter(GetName)]
        pub trait HasName {
            fn name(&self) -> &str;
        }

        #[derive(HasField)]
        pub struct Person {
            pub name: String,
        }

        delegate_components! {
            Person {
                GetNameComponent: UseFields,
            }
        }

        check_components! {
            Person {
                GetNameComponent,
            }
        }
    }

    /// A trait without the `Has` prefix, which gets no default and so names its provider.
    pub mod keyed_form {
        use cgp::prelude::*;

        #[cgp_getter {
            provider: DimensionsGetter,
            name: DimensionsComponent,
        }]
        pub trait Dimensions {
            fn width(&self) -> &f64;
            fn height(&self) -> &f64;
        }

        #[derive(HasField)]
        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        delegate_components! {
            Rectangle {
                DimensionsComponent: UseFields,
            }
        }

        check_components! {
            Rectangle {
                DimensionsComponent,
            }
        }
    }
}

/// ### What a context can wire it to
///
/// All three providers, and the direct impl. `WithProvider` is reached through its `WithField`
/// alias, which wraps the field-getter provider `UseField`.
pub mod what_a_context_can_wire_it_to {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct ByTag {
        pub first_name: String,
    }

    #[derive(HasField)]
    pub struct ByMethodName {
        pub name: String,
    }

    #[derive(HasField)]
    pub struct ByFieldGetter {
        pub display_name: String,
    }

    delegate_components! {
        ByTag {
            NameGetterComponent: UseField<Symbol!("first_name")>,
        }
    }

    delegate_components! {
        ByMethodName {
            NameGetterComponent: UseFields,
        }
    }

    delegate_components! {
        ByFieldGetter {
            NameGetterComponent: WithProvider<UseField<Symbol!("display_name")>>,
        }
    }

    check_components! {
        ByTag {
            NameGetterComponent,
        }
    }

    check_components! {
        ByMethodName {
            NameGetterComponent,
        }
    }

    check_components! {
        ByFieldGetter {
            NameGetterComponent,
        }
    }

    #[test]
    fn each_provider_reads_as_described() {
        assert_eq!(
            ByTag {
                first_name: "a".to_owned()
            }
            .name(),
            "a"
        );
        assert_eq!(
            ByMethodName {
                name: "b".to_owned()
            }
            .name(),
            "b"
        );
        assert_eq!(
            ByFieldGetter {
                display_name: "c".to_owned()
            }
            .name(),
            "c"
        );
    }
}

/// ## Examples
///
/// The page's three snippets: `UseField` on `Person`, `UseFields` on `Employee`, and the direct
/// impl on `Anonymous`. The page does not declare `Employee`; it has a `name` field here.
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

    pub fn greet(person: &Person) {
        println!("Hello, {}!", person.name());
    }

    #[derive(HasField)]
    pub struct Employee {
        pub name: String,
    }

    delegate_components! {
        Employee {
            NameGetterComponent: UseFields,
        }
    }

    check_components! {
        Employee {
            NameGetterComponent,
        }
    }

    pub struct Anonymous;

    impl HasName for Anonymous {
        fn name(&self) -> &str {
            "anonymous"
        }
    }

    #[test]
    fn all_three_contexts_answer() {
        let person = Person {
            first_name: "Ada".to_owned(),
        };
        greet(&person);
        assert_eq!(person.name(), "Ada");
        assert_eq!(
            Employee {
                name: "Grace".to_owned()
            }
            .name(),
            "Grace"
        );
        assert_eq!(Anonymous.name(), "anonymous");
    }
}

/// ## Under the hood
///
/// The trait whose three providers the page lists, checked with
/// `cargo cgp expand --item reference::macros::cgp_getter::under_the_hood`, plus a `&mut self` getter
/// and a slice getter, whose `WithProvider` bounds the page describes in prose.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_getter]
    pub trait HasCounter {
        fn counter(&mut self) -> &mut u32;
    }

    #[cgp_getter]
    pub trait HasBytes {
        fn bytes(&self) -> &[u8];
    }

    /// `#[prefix]` registers a getter component into a namespace, as on any component.
    #[cgp_getter]
    #[prefix(@app in DefaultNamespace)]
    pub trait HasTitle {
        fn title(&self) -> &str;
    }
}
