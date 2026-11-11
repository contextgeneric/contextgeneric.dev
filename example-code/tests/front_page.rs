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

// Property: the text the page shows between each pair of `front-page:` markers equals the text this
// crate compiles between the same markers. The page keeps its blocks as template literals in
// `src/pages/index.tsx`, which no compiler reads, so without this check the page could drift from
// the code that proves it.
mod page_text {
    use std::fs;
    use std::path::Path;

    /// The lines strictly between `// front-page:<name>:begin` and `// front-page:<name>:end`.
    fn region<'a>(source: &'a str, name: &str) -> Vec<&'a str> {
        let begin = format!("// front-page:{name}:begin");
        let end = format!("// front-page:{name}:end");
        let lines: Vec<&str> = source.lines().collect();
        let start = lines.iter().position(|l| l.trim() == begin).expect("begin marker") + 1;
        let stop = lines.iter().position(|l| l.trim() == end).expect("end marker");
        lines[start..stop].to_vec()
    }

    /// A Rust region, with the indentation of the module it sits in removed.
    fn rust_text(lines: &[&str]) -> String {
        let indent = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);
        lines
            .iter()
            .map(|l| if l.trim().is_empty() { "" } else { &l[indent..] })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The template literal a TSX region declares, with its escapes undone.
    fn tsx_text(lines: &[&str]) -> String {
        let joined = lines.join("\n");
        let open = joined.find('`').expect("opening backtick") + 1;
        let close = joined.rfind('`').expect("closing backtick");
        joined[open..close].replace("\\`", "`").replace("\\$", "$").replace("\\\\", "\\")
    }

    fn read(relative: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    #[test]
    fn the_page_shows_the_code_this_crate_compiles() {
        let page = read("../src/pages/index.tsx");
        let compiled = read("tests/front_page.rs");
        let rejected = read("tests/compile_fail/front_page/rust_accepts_only_one.rs");
        for (name, source) in [
            ("rejected-block", &rejected),
            ("cgp-block", &compiled),
            ("email-wiring", &compiled),
        ] {
            assert_eq!(
                tsx_text(&region(&page, name)),
                rust_text(&region(source, name)),
                "the page's `{name}` differs from the compiled code"
            );
        }
    }
}
