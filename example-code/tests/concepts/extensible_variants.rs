//! Code from `docs/concepts/extensible-variants.md` — *Extensible variants*.

/// ## An enum as a list of named variants
pub mod an_enum_as_a_list_of_named_variants {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct Circle {
        pub radius: u64,
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct Rectangle {
        pub width: u64,
        pub height: u64,
    }

    /// The shape of an enum is a *sum* of named variants, where a record's was a product of named
    /// fields.
    #[test]
    fn the_shape_is_a_type() {
        fn assert_shape<T>()
        where
            T: HasFields<
                Fields = Sum![
                    Field<Symbol!("Circle"), Circle>,
                    Field<Symbol!("Rectangle"), Rectangle>,
                ],
            >,
        {
        }

        assert_shape::<Shape>();
    }
}

/// ## One handler per variant, dispatched by name
pub mod one_handler_per_variant {
    use core::convert::Infallible;
    use core::fmt::Display;

    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::extra::dispatch::MatchWithFieldHandlers;
    use cgp::extra::handler::{Computer, ComputerComponent};
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum Reading {
        Temperature(u64),
        Label(String),
    }

    // The enum gained a third variant and this handler did not change.
    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum ExtendedReading {
        Temperature(u64),
        Label(String),
        Flag(bool),
    }

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
    fn the_same_handler_serves_two_enums() {
        use cgp::extra::handler::CanCompute;

        let app = App;
        let code = PhantomData::<()>;

        assert_eq!(app.compute(code, Reading::Temperature(21)), "21");
        assert_eq!(app.compute(code, Reading::Label("north".to_owned())), "north");

        // The wider enum works through the same wiring, with no new arm anywhere.
        assert_eq!(app.compute(code, ExtendedReading::Flag(true)), "true");
    }
}

/// ## Widening and narrowing between enums that share variants
pub mod widening_and_narrowing {
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

    #[test]
    fn a_narrow_value_lifts_into_a_wider_enum() {
        use cgp::core::field::impls::CanUpcast;

        let narrow = Reading::Temperature(21);
        let wide: ExtendedReading = narrow.upcast(PhantomData::<ExtendedReading>);

        assert_eq!(wide, ExtendedReading::Temperature(21));
    }

    #[test]
    fn narrowing_succeeds_only_for_a_variant_the_target_has() {
        use cgp::core::field::impls::CanDowncast;

        let wide = ExtendedReading::Label("north".to_owned());
        let narrow: Result<Reading, _> = wide.downcast(PhantomData::<Reading>);
        assert_eq!(narrow.ok(), Some(Reading::Label("north".to_owned())));

        let wide = ExtendedReading::Flag(true);
        let narrow: Result<Reading, _> = wide.downcast(PhantomData::<Reading>);
        assert!(narrow.is_err());
    }
}
