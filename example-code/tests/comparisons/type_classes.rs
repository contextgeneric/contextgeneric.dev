//! Code from `docs/comparisons/type-classes.md` — *Type classes*.

/// ## A component is a class; a provider is a first-class instance
pub mod a_component_is_a_class {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    delegate_components! { Rectangle { AreaCalculatorComponent: RectangleArea } }

    mod check_rectangle {
        use super::*;
        check_components! { Rectangle { AreaCalculatorComponent } }
    }

    #[test]
    fn the_instance_is_named_in_the_wiring() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert_eq!(rectangle.area(), 12.0);
    }
}

/// ## Overlapping providers need no specificity rule
/// ## Incoherent choices made explicit and local
pub mod overlapping_instances {
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
            @EncoderComponent.String: EncodeWithDisplay,
            @EncoderComponent.Vec<u8>: EncodeAsHex,
        }
    }

    delegate_components! {
        AppB {
            open EncoderComponent;
            @EncoderComponent.String: EncodeBytes,
            @EncoderComponent.Vec<u8>: EncodeBytes,
        }
    }

    mod check_app_a {
        use super::*;
        check_components! { AppA { EncoderComponent: [String, Vec<u8>] } }
    }

    mod check_app_b {
        use super::*;
        check_components! { AppB { EncoderComponent: [String, Vec<u8>] } }
    }

    #[test]
    fn two_local_scopes_each_coherent() {
        let bytes = vec![0xffu8, 0x00];
        assert_eq!(AppA.encode(&bytes), b"ff00".to_vec());
        assert_eq!(AppB.encode(&bytes), vec![0xff, 0x00]);

        let text = String::from("hi");
        assert_eq!(AppA.encode(&text), b"hi".to_vec());
        assert_eq!(AppB.encode(&text), b"hi".to_vec());
    }
}
