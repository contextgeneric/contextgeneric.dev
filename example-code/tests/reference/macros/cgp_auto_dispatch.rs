//! Code from `docs/reference/macros/cgp_auto_dispatch.md` — *`#[cgp_auto_dispatch]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`. This file builds
//! against the `cgp` pinned in `Cargo.lock`; the *Under the hood* listing was checked with
//! `cargo cgp expand` against the local `cgp` checkout, whose matcher call is qualified.

/// ## Examples
///
/// The page's `Shape` enum with `HasArea` and `CanScale`, and the argument-taking `contains` the
/// page shows under the hood.
pub mod examples {
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    pub struct Circle {
        pub radius: f64,
    }
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    #[cgp_auto_dispatch]
    pub trait HasArea {
        fn area(&self) -> f64;
    }

    impl HasArea for Circle {
        fn area(&self) -> f64 {
            core::f64::consts::PI * self.radius * self.radius
        }
    }

    impl HasArea for Rectangle {
        fn area(&self) -> f64 {
            self.width * self.height
        }
    }

    #[cgp_auto_dispatch]
    pub trait CanScale {
        fn scale(&mut self, factor: f64);
    }

    impl CanScale for Circle {
        fn scale(&mut self, factor: f64) {
            self.radius *= factor;
        }
    }

    impl CanScale for Rectangle {
        fn scale(&mut self, factor: f64) {
            self.width *= factor;
            self.height *= factor;
        }
    }

    #[cgp_auto_dispatch]
    pub trait CanContain {
        fn contains(&self, x: f64, y: f64) -> bool;
    }

    impl CanContain for Circle {
        fn contains(&self, x: f64, y: f64) -> bool {
            x * x + y * y <= self.radius * self.radius
        }
    }

    impl CanContain for Rectangle {
        fn contains(&self, x: f64, y: f64) -> bool {
            x.abs() <= self.width / 2.0 && y.abs() <= self.height / 2.0
        }
    }

    #[test]
    fn the_enum_dispatches_each_method() {
        let shape = Shape::Rectangle(Rectangle {
            width: 2.0,
            height: 2.0,
        });
        assert_eq!(shape.area(), 4.0);

        let mut shape = Shape::Circle(Circle { radius: 1.0 });
        shape.scale(2.0);
        assert!(shape.contains(1.5, 0.0));
        assert!(!shape.contains(2.5, 0.0));
    }
}

/// ## Usage
///
/// The shapes the page lists in prose: a by-value `self`, an `async` method, a trait generic
/// parameter, a supertrait, named and elided lifetimes, and a default body, each dispatched over one
/// enum.
pub mod usage {
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub enum Token {
        Word(Word),
        Number(Number),
    }

    pub struct Word(pub String);
    pub struct Number(pub u32);

    #[cgp_auto_dispatch]
    pub trait IntoText {
        fn into_text(self) -> String;
    }

    impl IntoText for Word {
        fn into_text(self) -> String {
            self.0
        }
    }

    impl IntoText for Number {
        fn into_text(self) -> String {
            self.0.to_string()
        }
    }

    #[cgp_auto_dispatch]
    pub trait CanLength {
        async fn length(&self) -> usize;
    }

    impl CanLength for Word {
        async fn length(&self) -> usize {
            self.0.len()
        }
    }

    impl CanLength for Number {
        async fn length(&self) -> usize {
            self.0.to_string().len()
        }
    }

    #[cgp_auto_dispatch]
    pub trait CanRender<Prefix> {
        fn render(&self, prefix: Prefix) -> String;
    }

    impl CanRender<char> for Word {
        fn render(&self, prefix: char) -> String {
            format!("{prefix}{}", self.0)
        }
    }

    impl CanRender<char> for Number {
        fn render(&self, prefix: char) -> String {
            format!("{prefix}{}", self.0)
        }
    }

    /// A supertrait, which the enum provides by dispatching it too.
    #[cgp_auto_dispatch]
    pub trait HasName {
        fn name(&self) -> String;
    }

    #[cgp_auto_dispatch]
    pub trait CanGreet: HasName {
        fn greet(&self) -> String;
    }

    impl HasName for Word {
        fn name(&self) -> String {
            self.0.clone()
        }
    }

    impl HasName for Number {
        fn name(&self) -> String {
            self.0.to_string()
        }
    }

    impl CanGreet for Word {
        fn greet(&self) -> String {
            format!("hello, {}", self.name())
        }
    }

    impl CanGreet for Number {
        fn greet(&self) -> String {
            format!("number {}", self.name())
        }
    }

    /// Named and elided lifetimes, and a default body.
    #[cgp_auto_dispatch]
    pub trait CanPick {
        fn pick<'a>(&'a self, fallback: &'a str) -> &'a str;

        fn first_char(&self, text: Option<&str>) -> Option<char> {
            text.and_then(|text| text.chars().next())
        }
    }

    impl CanPick for Word {
        fn pick<'a>(&'a self, _fallback: &'a str) -> &'a str {
            &self.0
        }
    }

    impl CanPick for Number {
        fn pick<'a>(&'a self, fallback: &'a str) -> &'a str {
            fallback
        }
    }

    #[test]
    fn every_listed_shape_dispatches() {
        assert_eq!(Token::Number(Number(12)).into_text(), "12");
        assert_eq!(
            futures::executor::block_on(Token::Word(Word("abc".to_owned())).length()),
            3
        );
        assert_eq!(Token::Word(Word("w".to_owned())).render('#'), "#w");
        assert_eq!(Token::Number(Number(7)).greet(), "number 7");
        assert_eq!(Token::Number(Number(7)).pick("none"), "none");
        assert_eq!(Token::Number(Number(7)).first_char(Some("xy")), Some('x'));
    }
}

/// ## Under the hood
///
/// The helper's reserved name: a free function called `area` beside the trait does not clash with
/// the `__compute_area__` helper the macro emits for the `area` method, a raw method name yields
/// valid generated names, and the `label` example's elided lifetimes keep their meaning.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    pub struct Circle {
        pub radius: f64,
    }

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    pub fn area(side: f64) -> f64 {
        side * side
    }

    #[cgp_auto_dispatch]
    pub trait HasArea {
        fn area(&self) -> f64;
    }

    impl HasArea for Circle {
        fn area(&self) -> f64 {
            core::f64::consts::PI * self.radius * self.radius
        }
    }

    impl HasArea for Rectangle {
        fn area(&self) -> f64 {
            self.width * self.height
        }
    }

    /// The raw-identifier case the page names: `r#type` dispatches through `ComputeType`.
    #[cgp_auto_dispatch]
    pub trait HasKind {
        fn r#type(&self) -> &'static str;
    }

    impl HasKind for Circle {
        fn r#type(&self) -> &'static str {
            "circle"
        }
    }

    impl HasKind for Rectangle {
        fn r#type(&self) -> &'static str {
            "rectangle"
        }
    }

    /// The elided-lifetime example: the returned borrow takes the receiver's lifetime, so it
    /// outlives a shorter-lived `suffix`.
    #[cgp_auto_dispatch]
    pub trait CanLabel {
        fn label(&self, suffix: &str) -> &str;
    }

    impl CanLabel for Circle {
        fn label(&self, _suffix: &str) -> &str {
            "circle"
        }
    }

    impl CanLabel for Rectangle {
        fn label(&self, _suffix: &str) -> &str {
            "rectangle"
        }
    }

    #[test]
    fn the_returned_borrow_outlives_the_argument() {
        let shape = Shape::Circle(Circle { radius: 1.0 });
        let label = {
            let suffix = String::from("!");
            shape.label(&suffix)
        };
        assert_eq!(label, "circle");
    }

    #[test]
    fn a_raw_method_name_dispatches() {
        assert_eq!(Shape::Circle(Circle { radius: 1.0 }).r#type(), "circle");
    }

    #[test]
    fn the_free_function_and_the_method_coexist() {
        let shape = Shape::Rectangle(Rectangle {
            width: 2.0,
            height: 3.0,
        });
        assert_eq!(shape.area(), 6.0);
        assert_eq!(area(2.0), 4.0);
    }
}
