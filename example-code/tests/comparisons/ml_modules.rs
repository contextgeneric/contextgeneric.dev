//! Code from `docs/comparisons/ml-modules.md` — *ML modules and modular implicits*.

/// ## Components are signatures; providers are structures
pub mod components_are_signatures {
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

    /// ## Higher-order providers are functors
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
    fn the_functor_is_applied_in_the_wiring() {
        let rectangle = ScaledRectangle {
            width: 2.0,
            height: 5.0,
            scale_factor: 3.0,
        };
        assert_eq!(rectangle.area(), 90.0);
    }
}

/// ## Abstract-type components are a signature's abstract types
pub mod abstract_type_components {
    use core::fmt::Debug;

    use cgp::prelude::*;

    // The page shows CGP's own `HasErrorType`; declaring it locally shows the shape the library's
    // definition has, and the local name shadows the prelude's within this module.
    #[cgp_type]
    pub trait HasErrorType {
        type Error: Debug;
    }
}

/// ## `delegate_components!` replaces manual functor application
pub mod delegate_components_replaces_functor_application {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    #[cgp_component(EmailSender)]
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    #[cgp_impl(new SendViaSmtp)]
    impl EmailSender {
        fn send_email(&self, #[implicit] smtp_server: &str, to: &str, body: &str) {
            // The page elides the body; a real provider opens a connection to `smtp_server`.
            let _ = (smtp_server, to, body);
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub smtp_server: String,
    }

    delegate_components! {
        App {
            EmailSenderComponent: SendViaSmtp,
            ErrorTypeProviderComponent: UseType<anyhow::Error>,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { EmailSenderComponent, ErrorTypeProviderComponent } }
    }

    #[test]
    fn the_table_is_resolved_in_any_order() {
        let app = App {
            smtp_server: "smtp.example.com".to_owned(),
        };
        app.send_email("ada@example.com", "hello");
    }
}
