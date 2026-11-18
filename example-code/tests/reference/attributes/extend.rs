//! Code from `docs/reference/attributes/extend.md` — *`#[extend]`*.
//!
//! The snippet the page rejects lives under `tests/compile_fail/reference/attributes/`.

/// ## Overview and Usage
///
/// A supertrait added to a component, a list in one attribute, and a list split across two.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_auto_getter]
    pub trait HasTitle {
        fn title(&self) -> &str;
    }

    #[cgp_fn]
    #[extend(HasName, HasTitle)]
    pub fn listed(&self) -> String {
        format!("{} {}", self.title(), self.name())
    }

    #[cgp_fn]
    #[extend(HasName)]
    #[extend(HasTitle)]
    pub fn split(&self) -> String {
        format!("{} {}", self.title(), self.name())
    }

    /// `#[extend]` and `#[uses]` together share one `Self:` predicate.
    #[cgp_fn]
    #[extend(HasName)]
    #[uses(HasTitle)]
    pub fn mixed(&self) -> String {
        format!("{} {}", self.title(), self.name())
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
        pub title: String,
    }

    #[test]
    fn the_lists_accumulate() {
        let person = Person {
            name: "Lovelace".to_owned(),
            title: "Countess".to_owned(),
        };
        assert_eq!(person.listed(), "Countess Lovelace");
        assert_eq!(person.split(), person.listed());
        assert_eq!(person.mixed(), person.listed());
    }
}

/// ## When the supertrait supplies a type
pub mod when_the_supertrait_supplies_a_type {
    use cgp::prelude::*;

    #[cgp_component(Loader)]
    #[use_type(HasErrorType.Error)]
    pub trait CanLoad {
        fn load(&self, path: &str) -> Result<String, Error>;
    }
}

/// ## Examples
///
/// The `#[cgp_fn]` example with a context that has both fields, and the component example with a
/// provider and wiring. The page names the `Greeter` provider trait without writing a provider.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_auto_getter]
    pub trait HasTitle {
        fn title(&self) -> &str;
    }

    #[cgp_fn]
    #[extend(HasName, HasTitle)]
    pub fn full_label(&self) -> String {
        format!("{} {}", self.title(), self.name())
    }

    /// A caller holding `T: FullLabel` can call the supertrait's getter without asking for it.
    pub fn name_of<T: FullLabel>(value: &T) -> &str {
        value.name()
    }

    #[cgp_component(Greeter)]
    #[extend(HasName)]
    pub trait CanGreet {
        fn greet(&self);
    }

    /// The provider trait carries `HasName` as a `where` bound, which an impl must prove rather than
    /// inherit, so the provider imports it again.
    #[cgp_impl(new GreetHello)]
    #[uses(HasName)]
    impl Greeter {
        fn greet(&self) {
            println!("Hello, {}!", self.name());
        }
    }

    /// A caller of `CanGreet` may rely on the supertrait without asking for it.
    pub fn greet_by_name<T: CanGreet>(value: &T) -> &str {
        value.greet();
        value.name()
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
        pub title: String,
    }

    delegate_components! {
        Person {
            GreeterComponent: GreetHello,
        }
    }

    check_components! {
        Person {
            GreeterComponent,
        }
    }

    #[test]
    fn callers_rely_on_the_supertrait() {
        let person = Person {
            name: "Ada".to_owned(),
            title: "Countess".to_owned(),
        };
        assert_eq!(person.full_label(), "Countess Ada");
        assert_eq!(name_of(&person), "Ada");
        assert_eq!(greet_by_name(&person), "Ada");
    }
}

/// ## Under the hood
///
/// The two listings' inputs. The page's `CanGreet` returns nothing; so does this one.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[cgp_auto_getter]
    pub trait HasTitle {
        fn title(&self) -> &str;
    }

    #[cgp_fn]
    #[extend(HasName, HasTitle)]
    pub fn full_label(&self) -> String {
        format!("{} {}", self.title(), self.name())
    }

    #[cgp_component(Greeter)]
    #[extend(HasName)]
    pub trait CanGreet {
        fn greet(&self);
    }

    /// The provider from Examples, proving the provider trait's `where` bound with `#[uses]`.
    #[cgp_impl(new GreetHello)]
    #[uses(HasName)]
    impl Greeter {
        fn greet(&self) {
            println!("Hello, {}!", self.name());
        }
    }
}
