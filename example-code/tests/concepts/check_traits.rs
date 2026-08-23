//! Code from `docs/concepts/check-traits.md` — *Checking your wiring*.

/// ## A context can be wrong and still compile
///
/// `RecordEmails` needs a `sent_emails` field and `BrokenApp` has none. Every line here compiles,
/// because nothing has asked whether the provider's requirements hold for this context yet.
pub mod a_context_can_be_wrong_and_still_compile {
    use core::cell::RefCell;

    use cgp::prelude::*;

    #[cgp_component(EmailSender)]
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    #[cgp_impl(new RecordEmails)]
    impl EmailSender {
        fn send_email(&self, #[implicit] sent_emails: &RefCell<Vec<String>>, to: &str, body: &str) {
            sent_emails.borrow_mut().push(format!("{to}: {body}"));
        }
    }

    #[derive(HasField)]
    pub struct BrokenApp {
        pub smtp_server: String,
    }

    // Accepted. The table records a choice; it does not verify one.
    delegate_components! { BrokenApp { EmailSenderComponent: RecordEmails } }
}

/// ## Where the failure surfaces instead
///
/// Calling the capability is the first thing that forces the question, so the error arrives at the
/// call rather than at the wiring — and names a field requirement the caller never wrote.
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/check_traits_where_the_failure_surfaces_1.rs`.
///
/// A check placed beside the wiring forces the same question at a line you chose. This one fails,
/// which is the point of it:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/check_traits_where_the_failure_surfaces_2.rs`.
pub mod where_the_failure_surfaces {}

/// ## A check is an empty impl of a trait that demands something
///
/// The hand-written form, so the generated one is not mysterious: a trait whose supertrait is the
/// requirement, and an impl with nothing in it.
pub mod a_check_is_an_empty_impl {
    use cgp::prelude::*;

    #[cgp_component(EmailSender)]
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    #[cgp_impl(new RecordEmails)]
    impl EmailSender {
        fn send_email(&self, #[implicit] sent_emails: &Vec<String>, to: &str, body: &str) {}
    }

    #[derive(HasField)]
    pub struct App {
        pub sent_emails: Vec<String>,
    }

    delegate_components! { App { EmailSenderComponent: RecordEmails } }

    // The plain-Rust version: nothing to prove in the body, so it compiles exactly when the
    // supertrait holds.
    trait CanUseApp: CanSendEmail {}
    impl CanUseApp for App {}

    // What `check_components!` writes instead, which reports the missing requirement rather than
    // only that the capability is unavailable.
    mod generated_form {
        use super::*;
        check_components! {
            App {
                EmailSenderComponent,
            }
        }
    }
}

/// ## Checking a stack one layer at a time
///
/// `#[check_providers]` asserts against each provider rather than against the context, so a broken
/// layer of a nested stack errors on its own line.
pub mod checking_a_stack_one_layer_at_a_time {
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

    #[cgp_impl(new ScaledArea<Inner>)]
    #[use_provider(Inner: AreaCalculator)]
    impl<Inner> AreaCalculator {
        fn area(&self, #[implicit] scale: f64) -> f64 {
            Inner::area(self) * scale * scale
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub width: f64,
        pub height: f64,
        pub scale: f64,
    }

    delegate_components! {
        App {
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    mod check_each_layer {
        use super::*;
        check_components! {
            #[check_providers(
                RectangleArea,
                ScaledArea<RectangleArea>,
            )]
            App {
                AreaCalculatorComponent,
            }
        }
    }

    #[test]
    fn the_stack_resolves() {
        let app = App {
            width: 3.0,
            height: 4.0,
            scale: 2.0,
        };
        assert_eq!(app.area(), 48.0);
    }
}
