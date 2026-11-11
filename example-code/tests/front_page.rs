//! Runnable check for the code shown on the front page, `src/pages/index.tsx`.
//!
//! The front page belongs to no `docs/` section, so it gets its own test binary, as the Quickstart
//! does. The page shows two blocks: the hero's encoder pair under CGP, and section 2's two wiring
//! lines for an application and its test. The rejected half of the hero, the same pair without CGP,
//! cannot compile, so it is a compile-fail fixture at `tests/compile_fail/front_page/`.
//!
//! The text between each pair of `front-page:` marker comments is exactly what the page shows, so a
//! test can compare it with `index.tsx`. Everything outside the markers is what the page omits: the
//! imports, the email swap's consumer trait, providers, and contexts, and the checks.

// The hero's CGP block. The page omits these two imports.
mod first_screen {
    use cgp::prelude::*;
    use core::fmt::Display;

    // front-page:cgp-block:begin
    #[cgp_component(Encoder)]
    pub trait CanEncode {
        fn encode(&self) -> Vec<u8>;
    }

    #[cgp_impl(new EncodeAsText)]
    #[uses(Display)]
    impl Encoder {
        fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
    }

    #[cgp_impl(new EncodeAsBytes)]
    #[uses(AsRef<[u8]>)]
    impl Encoder {
        fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
    }

    // Each type names the implementation it uses:
    delegate_components! { u64     { EncoderComponent: EncodeAsText } }
    delegate_components! { String  { EncoderComponent: EncodeAsText } }
    delegate_components! { Vec<u8> { EncoderComponent: EncodeAsBytes } }
    // front-page:cgp-block:end

    // The page does not show checks; these prove each wiring resolves rather than merely parses.
    check_components! { u64 { EncoderComponent } }
    check_components! { String { EncoderComponent } }
    check_components! { Vec<u8> { EncoderComponent } }

    // Property: for each wired type, `encode` runs the provider the wiring names. `u64` and `String`
    // take `EncodeAsText`, which encodes the `Display` text; `Vec<u8>` takes `EncodeAsBytes`, which
    // copies the bytes. `String` is the witness: it satisfies both bounds, so its result shows which
    // provider the wiring chose.
    #[test]
    fn each_type_encodes_with_the_provider_it_names() {
        assert_eq!(42u64.encode(), b"42");
        assert_eq!("hi".to_owned().encode(), b"hi");
        assert_eq!(vec![0xffu8, 0x00].encode(), vec![0xff, 0x00]);
    }
}

// Section 2's application shape: one consumer trait, a production provider and a test provider, and
// two contexts that choose between them. The page shows only the two wiring lines.
mod name_each_implementation {
    use cgp::prelude::*;
    use std::cell::RefCell;

    #[cgp_component(EmailSender)]
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    #[cgp_impl(new SendViaSmtp)]
    impl EmailSender {
        fn send_email(&self, #[implicit] smtp_host: &str, to: &str, body: &str) {
            // A real provider would connect to `smtp_host`; the page does not show the body.
            let _ = (smtp_host, to, body);
        }
    }

    #[cgp_impl(new RecordEmails)]
    impl EmailSender {
        fn send_email(&self, #[implicit] outbox: &RefCell<Vec<String>>, to: &str, body: &str) {
            outbox.borrow_mut().push(format!("{to}: {body}"));
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub smtp_host: String,
    }

    #[derive(HasField)]
    pub struct TestApp {
        pub outbox: RefCell<Vec<String>>,
    }

    // front-page:email-wiring:begin
    delegate_components! { App     { EmailSenderComponent: SendViaSmtp  } }
    delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }
    // front-page:email-wiring:end

    check_components! { App { EmailSenderComponent } }
    check_components! { TestApp { EmailSenderComponent } }

    // Property: the same call, `send_email`, runs the provider each context names. `TestApp` records
    // the message, which is how a test observes it; `App`'s provider records nothing, so the call
    // compiling and running for both is the claim the page makes.
    #[test]
    fn each_application_sends_through_the_provider_it_names() {
        let test_app = TestApp { outbox: RefCell::new(Vec::new()) };
        test_app.send_email("ada@example.com", "hello");
        assert_eq!(*test_app.outbox.borrow(), vec!["ada@example.com: hello".to_owned()]);

        let app = App { smtp_host: "smtp.example.com".to_owned() };
        app.send_email("ada@example.com", "hello");
    }
}
