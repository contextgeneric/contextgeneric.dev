//! Code from `docs/reference/traits/field-access/field_getter.md` — `FieldGetter`.
//!
//! Pins the Examples program: one getter component answered through `WithProvider` by two
//! `FieldGetter` providers, `UseField` reading a differently named field and a hand-written
//! provider reading a nested one.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::WithField;
    use cgp::prelude::*;

    #[cgp_getter(NameGetter)]
    pub trait HasName {
        fn name(&self) -> &String;
    }

    #[derive(HasField)]
    pub struct Person {
        pub first_name: String,
    }

    delegate_components! {
        Person {
            NameGetterComponent: WithField<Symbol!("first_name")>,
        }
    }

    pub struct Profile {
        pub display_name: String,
    }

    #[derive(HasField)]
    pub struct Account {
        pub profile: Profile,
    }

    pub struct ReadDisplayName;

    impl<Context, Tag> FieldGetter<Context, Tag> for ReadDisplayName
    where
        Context: HasField<Symbol!("profile"), Value = Profile>,
    {
        type Value = String;

        fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &String {
            &context.get_field(PhantomData).display_name
        }
    }

    delegate_components! {
        Account {
            NameGetterComponent: WithProvider<ReadDisplayName>,
        }
    }

    check_components! {
        Person {
            NameGetterComponent,
        }
    }

    check_components! {
        Account {
            NameGetterComponent,
        }
    }

    pub fn demo() {
        let person = Person {
            first_name: "Ada".to_owned(),
        };
        let account = Account {
            profile: Profile {
                display_name: "ada99".to_owned(),
            },
        };

        assert_eq!(person.name(), "Ada");
        assert_eq!(account.name(), "ada99");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
