//! Code from `docs/reference/providers/dispatch/match_with_field_handlers.md` — `MatchWithFieldHandlers`.
//!
//! Pins the Examples program: one handler over `Field<Tag, Value>` reads the variant name from the
//! tag, and the same wiring serves a second enum with no new arm.

/// ## Usage
///
/// `MatchWithFieldHandlersRef` hands the provider a `Field<Tag, &Value>`, so the same generic
/// provider serves a borrowed enum.
pub mod usage {
    use core::fmt::Display;

    use cgp::core::field::traits::StaticString;
    use cgp::extra::dispatch::MatchWithFieldHandlersRef;
    use cgp::extra::handler::CanCompute;
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub enum Reading {
        Temperature(i32),
        Label(String),
    }

    #[cgp_computer]
    pub fn describe<Tag, Value>(field: Field<Tag, Value>) -> String
    where
        Tag: StaticString,
        Value: Display,
    {
        format!("{}: {}", Tag::VALUE, field.value)
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: MatchWithFieldHandlersRef<Describe>,
        }
    }

    check_components! {
        App {
            ComputerComponent: <'a> ((), &'a Reading),
        }
    }

    #[test]
    fn the_borrowed_form_tags_a_borrowed_payload() {
        let reading = Reading::Temperature(21);

        assert_eq!(App.compute(PhantomData::<()>, &reading), "Temperature: 21");
    }
}

/// ## Examples
pub mod examples {
    use core::fmt::Display;

    use cgp::core::field::traits::StaticString;
    use cgp::extra::dispatch::MatchWithFieldHandlers;
    use cgp::extra::handler::CanCompute;
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub enum Reading {
        Temperature(i32),
        Label(String),
    }

    #[derive(CgpData)]
    pub enum Event {
        Started(u64),
        Stopped(bool),
    }

    #[cgp_computer]
    pub fn describe<Tag, Value>(field: Field<Tag, Value>) -> String
    where
        Tag: StaticString,
        Value: Display,
    {
        format!("{}: {}", Tag::VALUE, field.value)
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: MatchWithFieldHandlers<Describe>,
        }
    }

    check_components! {
        App {
            ComputerComponent: [((), Reading), ((), Event)],
        }
    }

    pub fn demo() {
        let code = PhantomData::<()>;

        assert_eq!(
            App.compute(code, Reading::Temperature(21)),
            "Temperature: 21"
        );
        assert_eq!(
            App.compute(code, Reading::Label("north".to_owned())),
            "Label: north"
        );
        assert_eq!(App.compute(code, Event::Stopped(true)), "Stopped: true");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
