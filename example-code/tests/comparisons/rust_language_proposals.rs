//! Code from `docs/comparisons/rust-language-proposals.md` — *Rust's own proposals*.
//!
//! The specialization snippet the page quotes from RFC 1210 is rejected on stable Rust; it is the
//! trybuild fixture `tests/compile_fail/comparisons/rust_language_proposals_specialization.rs`.

/// ## A named impl is a provider
pub mod a_named_impl_is_a_provider {
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

    /// ## An incoherent bound resolves through the context
    pub struct ApiServer;
    pub struct Firmware;

    delegate_components! {
        ApiServer {
            open EncoderComponent;
            @EncoderComponent.String: EncodeWithDisplay,
        }
    }

    delegate_components! {
        Firmware {
            open EncoderComponent;
            @EncoderComponent.String: EncodeBytes,
        }
    }

    mod check_api_server {
        use super::*;
        check_components! { ApiServer { EncoderComponent: String } }
    }

    mod check_firmware {
        use super::*;
        check_components! { Firmware { EncoderComponent: String } }
    }

    #[test]
    fn two_contexts_name_two_overlapping_impls_for_one_type() {
        let value = String::from("hello");
        assert_eq!(ApiServer.encode(&value), b"hello".to_vec());
        assert_eq!(Firmware.encode(&value), b"hello".to_vec());
    }
}

/// ## Passing an impl explicitly is a higher-order provider
pub mod passing_an_impl_explicitly {
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

    #[cgp_impl(new ScaledArea<InnerCalculator>)]
    #[use_provider(InnerCalculator: AreaCalculator)]
    impl<InnerCalculator> AreaCalculator {
        fn area(&self, #[implicit] scale_factor: f64) -> f64 {
            InnerCalculator::area(self) * scale_factor * scale_factor
        }
    }

    #[derive(HasField)]
    pub struct ScaledRectangle {
        pub width: f64,
        pub height: f64,
        pub scale_factor: f64,
    }

    delegate_components! {
        ScaledRectangle {
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    mod check_scaled_rectangle {
        use super::*;
        check_components! { ScaledRectangle { AreaCalculatorComponent } }
    }

    #[test]
    fn the_inner_impl_is_named_in_the_wiring() {
        let rectangle = ScaledRectangle {
            width: 3.0,
            height: 4.0,
            scale_factor: 2.0,
        };
        assert_eq!(rectangle.area(), 48.0);
    }
}

/// ## A `with` clause becomes a context field
pub mod a_capability_is_a_context_field {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self, #[implicit] name: &str) -> String {
            format!("Hello, {name}!")
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    delegate_components! {
        App {
            GreeterComponent: GreetHello,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { GreeterComponent } }
    }

    #[test]
    fn constructing_the_context_binds_the_value() {
        let app = App {
            name: "World".to_owned(),
        };
        assert_eq!(app.greet(), "Hello, World!");
    }
}
