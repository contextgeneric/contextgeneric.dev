//! Code from `docs/comparisons/implicit-parameters.md` — *Implicit parameters*.

/// ## Implicit arguments read named context fields
pub mod implicit_arguments_are_implicit_value_parameters {
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[test]
    fn the_context_supplies_the_arguments() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert_eq!(rectangle.rectangle_area(), 12.0);
    }
}

/// ## Abstract types are determined by the context
pub mod abstract_types_are_implicit_type_parameters {
    use core::fmt::Debug;

    use cgp::prelude::*;

    // The page shows CGP's own `HasErrorType`; the local declaration shadows the prelude's here.
    #[cgp_type]
    pub trait HasErrorType {
        type Error: Debug;
    }
}

/// ## Components and wiring make instance selection explicit
pub mod components_and_wiring_are_type_classes {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_component(Encoder)]
    pub trait CanEncode<Value> {
        fn encode(&self, value: &Value) -> Vec<u8>;
    }

    #[cgp_impl(new EncodeWithDisplay)]
    impl<Value: Display> Encoder<Value> {
        fn encode(&self, value: &Value) -> Vec<u8> {
            value.to_string().into_bytes()
        }
    }

    #[cgp_impl(new EncodeBytes)]
    impl<Value: AsRef<[u8]>> Encoder<Value> {
        fn encode(&self, value: &Value) -> Vec<u8> {
            value.as_ref().to_vec()
        }
    }

    #[cgp_impl(new EncodeAsHex)]
    impl<Value: AsRef<[u8]>> Encoder<Value> {
        fn encode(&self, value: &Value) -> Vec<u8> {
            value
                .as_ref()
                .iter()
                .flat_map(|byte| format!("{byte:02x}").into_bytes())
                .collect()
        }
    }

    pub struct AppA;
    pub struct AppB;

    delegate_components! {
        AppA {
            open EncoderComponent;
            @EncoderComponent.Vec<u8>: EncodeAsHex,
        }
    }

    delegate_components! {
        AppB {
            open EncoderComponent;
            @EncoderComponent.Vec<u8>: EncodeBytes,
        }
    }

    mod check_app_a {
        use super::*;
        check_components! { AppA { EncoderComponent: Vec<u8> } }
    }

    mod check_app_b {
        use super::*;
        check_components! { AppB { EncoderComponent: Vec<u8> } }
    }

    #[test]
    fn the_same_type_two_encodings_two_contexts() {
        let bytes = vec![0xabu8];
        assert_eq!(AppA.encode(&bytes), b"ab".to_vec());
        assert_eq!(AppB.encode(&bytes), vec![0xab]);
    }
}
