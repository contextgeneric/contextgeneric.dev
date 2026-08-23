//! Code from `docs/reference/providers/dispatch/match_with_field_handlers.md` — `MatchWithFieldHandlers`.

/// ## Usage and Examples
///
/// `MatchWithFieldHandlers` builds the per-variant list from the enum's fields like
/// `MatchWithValueHandlers`, but hands each payload to the provider as a `Field<Tag, Value>` with the
/// variant tag still attached. Here one handler over `Field<Tag, Value>` serves every variant, and the
/// same wiring serves a wider enum with no new arm.
pub mod matching_with_the_tag_attached {
    use core::convert::Infallible;
    use core::fmt::Display;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::dispatch::MatchWithFieldHandlers;
    use cgp::extra::handler::ComputerComponent;
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum Reading {
        Temperature(u64),
        Label(String),
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum ExtendedReading {
        Temperature(u64),
        Label(String),
        Flag(bool),
    }

    // The payload arrives as a `Field<Tag, Value>`, so the handler still has the variant tag.
    #[cgp_computer]
    pub fn field_to_string<Tag, Value>(Field { value, .. }: Field<Tag, Value>) -> String
    where
        Value: Display,
    {
        value.to_string()
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<Infallible>,
            ComputerComponent: MatchWithFieldHandlers<FieldToString>,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ComputerComponent: [((), Reading), ((), ExtendedReading)],
            }
        }
    }

    #[test]
    fn one_handler_serves_every_variant_of_two_enums() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(app.compute(code, Reading::Temperature(21)), "21");
        assert_eq!(app.compute(code, Reading::Label("north".to_owned())), "north");
        assert_eq!(app.compute(code, ExtendedReading::Flag(true)), "true");
    }
}
