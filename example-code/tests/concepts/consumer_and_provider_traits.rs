//! Code from `docs/concepts/consumer-and-provider-traits.md` — *Consumer and provider traits*.
//!
//! Module names follow the page's headings. Code the page **rejects** is carried as a
//! `compile_fail` doctest, and the two listings the page presents as *generated* code are rebuilt by
//! hand in [`how_a_call_finds_its_provider`], which is a weaker check than it looks — see the note
//! there.

/// ## What an ordinary Rust trait does, and where it stops
///
/// The arrangement the page opens on: two application types, two behaviours, no CGP. It compiles,
/// which is what the paragraph after it says.
///
/// The section's second block does not, and is the reason the first one does not scale. The two
/// marker traits it bounds on are undeclared on the page, so they are declared here; without them
/// the failure would be an unresolved name rather than the overlap it is meant to show:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/consumer_and_provider_traits_what_an_ordinary_rust_trait_does.rs`.
pub mod what_an_ordinary_rust_trait_does {
    use core::cell::RefCell;

    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    pub struct App {
        pub smtp_server: String,
    }

    pub struct TestApp {
        pub sent_emails: RefCell<Vec<String>>,
    }

    impl CanSendEmail for App {
        fn send_email(&self, to: &str, body: &str) {
            // open a connection to self.smtp_server and send
        }
    }

    impl CanSendEmail for TestApp {
        fn send_email(&self, to: &str, body: &str) {
            self.sent_emails.borrow_mut().push(format!("{to}: {body}"));
        }
    }
}

/// ## Splitting one trait in two
///
/// The trait pair written out by hand, and two implementations targeting named types. The page
/// elides both bodies as `/* ... */`, which would leave the impls empty and the required method
/// missing, so they are filled in here.
pub mod splitting_one_trait_in_two {
    // what callers use
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    // what implementations target
    pub trait EmailSender<Context> {
        fn send_email(context: &Context, to: &str, body: &str);
    }

    // The page names these two providers without declaring them.
    pub struct SendViaSmtp;
    pub struct RecordEmails;

    impl<Context> EmailSender<Context> for SendViaSmtp {
        fn send_email(context: &Context, to: &str, body: &str) {}
    }

    impl<Context> EmailSender<Context> for RecordEmails {
        fn send_email(context: &Context, to: &str, body: &str) {}
    }
}

/// ## How a call finds its provider
///
/// The section's three listings — the wiring table, the consumer blanket impl, and the provider
/// blanket impl — assembled into one program. Because the last two are what `#[cgp_component]`
/// *generates*, they are written by hand here rather than derived, which is the point: the page
/// claims a component is these pieces, and a hand-rolled component built from exactly them resolves
/// a call and passes a check.
///
/// **This proves the listings are structurally sound, not that they are textually current.** The
/// authority on what the macro emits is `cargo cgp expand --lib --item …`, which is also what the
/// page's own note about reserved names and the spelled-out delegate refers to. Check the text
/// against `expand`; check the shape against this module.
pub mod how_a_call_finds_its_provider {
    use cgp::prelude::*;

    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    pub trait EmailSender<Context> {
        fn send_email(context: &Context, to: &str, body: &str);
    }

    // The key. The page calls it "a marker type generated alongside the two traits".
    pub struct EmailSenderComponent;

    pub struct SendViaSmtp;
    pub struct RecordEmails;

    impl<Context> EmailSender<Context> for SendViaSmtp {
        fn send_email(context: &Context, to: &str, body: &str) {}
    }

    impl<Context> EmailSender<Context> for RecordEmails {
        fn send_email(context: &Context, to: &str, body: &str) {}
    }

    // Generated for every provider, and not shown on the page, which says only that it is
    // "generated, never written by hand". Without it the provider blanket impl below has an
    // unsatisfiable bound.
    impl<Context> IsProviderFor<EmailSenderComponent, Context, ()> for SendViaSmtp {}
    impl<Context> IsProviderFor<EmailSenderComponent, Context, ()> for RecordEmails {}

    pub struct App;
    pub struct TestApp;

    delegate_components! { App     { EmailSenderComponent: SendViaSmtp } }
    delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }

    // The first generated implementation: a context that implements the provider trait for itself
    // gets the consumer trait.
    impl<Context> CanSendEmail for Context
    where
        Context: EmailSender<Context>,
    {
        fn send_email(&self, to: &str, body: &str) {
            Context::send_email(self, to, body)
        }
    }

    // The second: anything with a table entry inherits the provider trait from what the entry names.
    impl<Context, Provider> EmailSender<Context> for Provider
    where
        Provider: DelegateComponent<EmailSenderComponent>
            + IsProviderFor<EmailSenderComponent, Context, ()>,
        Provider::Delegate: EmailSender<Context>,
    {
        fn send_email(context: &Context, to: &str, body: &str) {
            Provider::Delegate::send_email(context, to, body)
        }
    }

    /// The three-step resolution the page traces, asserted as a bound rather than described.
    #[test]
    fn a_call_resolves_through_the_table() {
        fn resolves<Context: CanSendEmail>() {}

        resolves::<App>();
        resolves::<TestApp>();

        App.send_email("a@b.c", "hi");
    }
}

/// ## Writing it
///
/// The same component as [`how_a_call_finds_its_provider`], written the way it is actually written:
/// the two traits, the marker, and both blanket impls come from `#[cgp_component]`, and each
/// provider is a `#[cgp_impl]` block in consumer-trait shape.
pub mod writing_it {
    use core::cell::RefCell;

    use cgp::prelude::*;

    #[cgp_component(EmailSender)]
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    #[cgp_impl(new SendViaSmtp)]
    impl EmailSender {
        fn send_email(&self, #[implicit] smtp_server: &str, to: &str, body: &str) {
            // open a connection to `smtp_server` and send the message
        }
    }

    #[cgp_impl(new RecordEmails)]
    impl EmailSender {
        fn send_email(
            &self,
            #[implicit] sent_emails: &RefCell<Vec<String>>,
            to: &str,
            body: &str,
        ) {
            sent_emails.borrow_mut().push(format!("{to}: {body}"));
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub smtp_server: String,
    }

    #[derive(HasField)]
    pub struct TestApp {
        pub sent_emails: RefCell<Vec<String>>,
    }

    delegate_components! { App     { EmailSenderComponent: SendViaSmtp } }
    delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }

    mod check_app {
        use super::*;
        check_components! { App { EmailSenderComponent } }
    }

    mod check_test_app {
        use super::*;
        check_components! { TestApp { EmailSenderComponent } }
    }

    /// The page's claim that the implicit arguments leave the public signature alone, so
    /// `app.send_email(to, body)` still takes two arguments.
    #[test]
    fn each_provider_reads_a_different_field_and_neither_reaches_the_signature() {
        let test_app = TestApp {
            sent_emails: RefCell::new(Vec::new()),
        };
        test_app.send_email("a@b.c", "hi");

        assert_eq!(test_app.sent_emails.borrow().as_slice(), ["a@b.c: hi"]);
    }
}
