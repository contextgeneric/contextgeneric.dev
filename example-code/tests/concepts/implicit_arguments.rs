//! Code from `docs/concepts/implicit-arguments.md` — *Implicit arguments*.

/// ## Reading a field, spelled out
///
/// What an implicit argument stands in for: a field requirement declared by hand and a read that
/// carries a type-level tag.
pub mod reading_a_field_spelled_out {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetByName)]
    impl Greeter
    where
        Self: HasField<Symbol!("name"), Value = String>,
    {
        fn greet(&self) -> String {
            let name = self.get_field(PhantomData::<Symbol!("name")>);
            format!("Hello, {name}!")
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    delegate_components! { App { GreeterComponent: GreetByName } }

    mod check_app {
        use super::*;
        check_components! { App { GreeterComponent } }
    }
}

/// ## The same provider as a function taking arguments
pub mod the_same_provider_as_a_function {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetByName)]
    impl Greeter {
        fn greet(&self, #[implicit] name: &str) -> String {
            format!("Hello, {name}!")
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    delegate_components! { App { GreeterComponent: GreetByName } }

    mod check_app {
        use super::*;
        check_components! { App { GreeterComponent } }
    }

    /// The argument is gone from the public signature: `greet()` still takes nothing.
    #[test]
    fn the_implicit_argument_does_not_reach_the_caller() {
        let app = App {
            name: "World".to_owned(),
        };
        assert_eq!(app.greet(), "Hello, World!");
    }
}

/// ## A capability from a function alone
///
/// `#[cgp_fn]` needs no trait, no provider, and no wiring — the blanket impl applies to any context
/// carrying the fields.
pub mod a_capability_from_a_function_alone {
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

    #[test]
    fn any_context_with_the_fields_has_the_capability() {
        let rectangle = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert_eq!(rectangle.rectangle_area(), 12.0);
    }
}

/// ## The declared type decides how the field is read
pub mod the_declared_type_decides_the_read {
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn describe(
        &self,
        // Owned: read by reference and cloned, so the context keeps its field.
        #[implicit] name: String,
        // `&str`: backed by a `String` field, borrowed rather than cloned.
        #[implicit] title: &str,
        // Any other borrow: taken as it stands.
        #[implicit] tags: &Vec<String>,
    ) -> String {
        format!("{title} {name} {tags:?}")
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
        pub title: String,
        pub tags: Vec<String>,
    }

    #[test]
    fn the_context_keeps_its_fields() {
        let app = App {
            name: "Ada".to_owned(),
            title: "Dr".to_owned(),
            tags: vec!["admin".to_owned()],
        };

        assert_eq!(app.describe(), r#"Dr Ada ["admin"]"#);
        // Still there — the owned argument was cloned out.
        assert_eq!(app.name, "Ada");
    }
}

/// ## When a getter trait is still the right thing
///
/// The case an implicit argument cannot reach: the value lives on a type other than the provider's
/// own context, so there is no `self` field to read and the requirement is a bound on that other
/// type.
pub mod when_a_getter_is_still_right {
    use cgp::prelude::*;

    #[cgp_auto_getter]
    pub trait HasAuthHeader {
        fn auth_header(&self) -> &str;
    }

    #[cgp_component(RequestAuthenticator)]
    pub trait CanAuthenticate<Request> {
        fn authenticate(&self, request: &Request) -> bool;
    }

    #[cgp_impl(new AuthenticateByHeader)]
    impl<Request> RequestAuthenticator<Request>
    where
        Request: HasAuthHeader,
    {
        fn authenticate(&self, request: &Request) -> bool {
            request.auth_header().starts_with("Bearer ")
        }
    }

    #[derive(HasField)]
    pub struct Request {
        pub auth_header: String,
    }

    pub struct App;

    delegate_components! {
        App {
            open RequestAuthenticatorComponent;
            @RequestAuthenticatorComponent.Request: AuthenticateByHeader,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { RequestAuthenticatorComponent: Request } }
    }

    #[test]
    fn the_getter_is_a_bound_on_the_request_not_on_the_context() {
        let request = Request {
            auth_header: "Bearer t".to_owned(),
        };
        assert!(App.authenticate(&request));
    }
}
