//! Code from `docs/reference/attributes/implicit.md` — *`#[implicit]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/attributes/`.

/// ## Overview
pub mod overview {
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    // The page names `rect` without declaring its type.
    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[test]
    fn the_caller_passes_nothing() {
        let rect = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(rect.rectangle_area(), 6.0);
    }
}

/// ## Usage, How the argument's type decides the read, and Mutable arguments
///
/// The Usage snippet as a `#[cgp_fn]`, the raw-identifier rule, and one function per row of the
/// access table, each run against a context holding the field type the row names.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }

    #[cgp_fn]
    pub fn kind(&self, #[implicit] r#type: &str) -> String {
        r#type.to_owned()
    }

    #[cgp_fn]
    pub fn read_owned(&self, #[implicit] pair: (f64, f64)) -> f64 {
        pair.0 + pair.1
    }

    #[cgp_fn]
    pub fn read_ref(&self, #[implicit] tags: &Vec<String>) -> usize {
        tags.len()
    }

    #[cgp_fn]
    pub fn read_str(&self, #[implicit] name: &str) -> usize {
        name.len()
    }

    #[cgp_fn]
    pub fn read_slice(&self, #[implicit] bytes: &[u8]) -> usize {
        bytes.len()
    }

    #[cgp_fn]
    pub fn read_option_ref(&self, #[implicit] limit: Option<&u32>) -> u32 {
        limit.copied().unwrap_or(0)
    }

    #[cgp_fn]
    pub fn read_option_str(&self, #[implicit] nickname: Option<&str>) -> usize {
        nickname.map_or(0, str::len)
    }

    #[cgp_fn]
    pub fn read_mref(&self, #[implicit] name: MRef<'_, String>) -> usize {
        name.len()
    }

    #[cgp_fn]
    pub fn bump(&mut self, #[implicit] counter: &mut u64) {
        *counter += 1;
    }

    #[cgp_fn]
    pub fn shout_name(&mut self, #[implicit] name: &mut str) {
        name.make_ascii_uppercase();
    }

    #[cgp_fn]
    pub fn zero_bytes(&mut self, #[implicit] bytes: &mut [u8]) {
        bytes.fill(0);
    }

    #[cgp_fn]
    pub fn raise_limit(&mut self, #[implicit] limit: Option<&mut u32>) {
        if let Some(limit) = limit {
            *limit += 1;
        }
    }

    #[cgp_fn]
    pub fn shout_nickname(&mut self, #[implicit] nickname: Option<&mut str>) {
        if let Some(nickname) = nickname {
            nickname.make_ascii_uppercase();
        }
    }

    /// A `&mut self` receiver with several immutable implicit arguments.
    #[cgp_fn]
    pub fn describe_mut(&mut self, #[implicit] name: &str, #[implicit] bytes: &[u8]) -> usize {
        name.len() + bytes.len()
    }

    #[derive(HasField)]
    pub struct Record {
        pub width: f64,
        pub height: f64,
        pub r#type: String,
        pub pair: (f64, f64),
        pub tags: Vec<String>,
        pub name: String,
        pub bytes: Vec<u8>,
        pub limit: Option<u32>,
        pub nickname: Option<String>,
        pub counter: u64,
    }

    #[test]
    fn every_access_form_reads_its_field() {
        let mut record = Record {
            width: 2.0,
            height: 3.0,
            r#type: "box".to_owned(),
            pair: (1.0, 2.0),
            tags: vec!["a".to_owned()],
            name: "ada".to_owned(),
            bytes: vec![1, 2],
            limit: Some(5),
            nickname: Some("lin".to_owned()),
            counter: 0,
        };

        assert_eq!(record.area(), 6.0);
        assert_eq!(record.kind(), "box");
        assert_eq!(record.read_owned(), 3.0);
        assert_eq!(record.read_ref(), 1);
        assert_eq!(record.read_str(), 3);
        assert_eq!(record.read_slice(), 2);
        assert_eq!(record.read_option_ref(), 5);
        assert_eq!(record.read_option_str(), 3);
        assert_eq!(record.read_mref(), 3);
        assert_eq!(record.describe_mut(), 5);

        record.bump();
        record.shout_name();
        record.zero_bytes();
        record.raise_limit();
        record.shout_nickname();

        assert_eq!(record.counter, 1);
        assert_eq!(record.name, "ADA");
        assert_eq!(record.bytes, vec![0, 0]);
        assert_eq!(record.limit, Some(6));
        assert_eq!(record.nickname.as_deref(), Some("LIN"));
    }
}

/// ## Examples
///
/// The complete trait, the mutable example, and the `#[cgp_impl]` fragment. The page assumes the
/// `EmailSender` component and the `App` context without declaring them.
pub mod examples {
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

    pub fn print_area(rect: &Rectangle) {
        println!("area = {}", rect.rectangle_area());
    }

    #[cgp_fn]
    pub fn shout(&mut self, #[implicit] name: &mut str) {
        name.make_ascii_uppercase();
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    #[cgp_component(EmailSender)]
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    #[cgp_impl(new SendViaSmtp)]
    impl EmailSender {
        fn send_email(&self, #[implicit] smtp_server: &str, to: &str, body: &str) {
            // connect to `smtp_server` and send the message
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
        }
    }

    check_components! {
        App {
            EmailSenderComponent,
        }
    }

    #[test]
    fn the_examples_run() {
        print_area(&Rectangle {
            width: 2.0,
            height: 3.0,
        });

        let mut person = Person {
            name: "ada".to_owned(),
        };
        person.shout();
        assert_eq!(person.name, "ADA");

        let app = App {
            smtp_server: "smtp.example.com".to_owned(),
        };
        app.send_email("to@example.com", "hi");
    }
}

/// ## Under the hood
///
/// The hand-written expansion of the page's listing, under another trait name so it can sit
/// beside the macro's own output, plus the slice and mutable bounds the section names.
pub mod under_the_hood {
    use cgp::prelude::*;

    pub trait RectangleArea {
        fn rectangle_area(&self) -> f64;
    }

    impl<__Context__> RectangleArea for __Context__
    where
        Self: HasField<Symbol!("width"), Value = f64> + HasField<Symbol!("height"), Value = f64>,
    {
        fn rectangle_area(&self) -> f64 {
            let width: f64 = self.get_field(PhantomData::<Symbol!("width")>).clone();
            let height: f64 = self.get_field(PhantomData::<Symbol!("height")>).clone();

            width * height
        }
    }

    #[cgp_fn]
    pub fn slice_len(&self, #[implicit] slice: &[u8]) -> usize {
        slice.len()
    }

    #[cgp_fn]
    pub fn bump_counter(&mut self, #[implicit] counter: &mut u64) {
        *counter += 1;
    }

    /// Two methods reading one field at one type contribute one bound.
    #[cgp_component(Namer)]
    pub trait CanName {
        fn first(&self) -> String;
        fn second(&self) -> String;
    }

    #[cgp_impl(new ReadName)]
    impl Namer {
        fn first(&self, #[implicit] name: &str) -> String {
            name.to_owned()
        }

        fn second(&self, #[implicit] name: &str) -> String {
            name.to_uppercase()
        }
    }

    #[derive(HasField)]
    pub struct Sample {
        pub width: f64,
        pub height: f64,
        pub slice: [u8; 3],
        pub counter: u64,
        pub name: String,
    }

    delegate_components! {
        Sample {
            NamerComponent: ReadName,
        }
    }

    check_components! {
        Sample {
            NamerComponent,
        }
    }

    #[test]
    fn the_expansion_behaves_like_the_macro() {
        let mut sample = Sample {
            width: 2.0,
            height: 3.0,
            slice: [1, 2, 3],
            counter: 0,
            name: "ada".to_owned(),
        };
        assert_eq!(RectangleArea::rectangle_area(&sample), 6.0);
        // An array field qualifies for a `&[u8]` argument through `AsRef<[u8]>`.
        assert_eq!(sample.slice_len(), 3);
        sample.bump_counter();
        assert_eq!(sample.counter, 1);
        assert_eq!((sample.first(), sample.second()), ("ada".to_owned(), "ADA".to_owned()));
    }
}

/// ## Common Mistakes
///
/// Reads that differ only in how they borrow agree on one field type. The rejected snippets are
/// trybuild fixtures.
pub mod common_mistakes {
    use cgp::prelude::*;

    #[cgp_component(Namer)]
    pub trait CanName {
        fn borrowed(&self) -> String;
        fn owned(&self) -> String;
    }

    #[cgp_impl(new ReadName)]
    impl Namer {
        fn borrowed(&self, #[implicit] name: &str) -> String {
            name.to_owned()
        }

        fn owned(&self, #[implicit] name: String) -> String {
            name
        }
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    delegate_components! {
        Person {
            NamerComponent: ReadName,
        }
    }

    check_components! {
        Person {
            NamerComponent,
        }
    }
}
