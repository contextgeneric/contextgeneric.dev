//! Code from `docs/comparisons/policy-based-design.md` — *C++ policy-based design*.

use cgp::prelude::*;

#[cgp_component(MessageProvider)]
pub trait HasMessage {
    fn message(&self) -> String;
}

#[cgp_component(Writer)]
pub trait CanWrite {
    fn write(&self, message: String);
}

#[cgp_impl(new EnglishMessage)]
impl MessageProvider {
    fn message(&self) -> String {
        "Hello, World!".to_owned()
    }
}

#[cgp_impl(new GermanMessage)]
impl MessageProvider {
    fn message(&self) -> String {
        "Hallo Welt!".to_owned()
    }
}

#[cgp_impl(new WriteToStdout)]
impl Writer {
    fn write(&self, message: String) {
        println!("{message}");
    }
}

/// ## Providers are policies; the context is the host
pub mod providers_are_policies {
    use super::*;

    #[cgp_fn]
    #[uses(HasMessage, CanWrite)]
    pub fn run(&self) {
        self.write(self.message());
    }

    pub struct EnglishApp;
    pub struct GermanApp;

    delegate_components! {
        EnglishApp {
            MessageProviderComponent: EnglishMessage,
            WriterComponent: WriteToStdout,
        }
    }

    delegate_components! {
        GermanApp {
            MessageProviderComponent: GermanMessage,
            WriterComponent: WriteToStdout,
        }
    }

    mod check_english {
        use super::*;
        check_components! { EnglishApp { MessageProviderComponent, WriterComponent } }
    }

    mod check_german {
        use super::*;
        check_components! { GermanApp { MessageProviderComponent, WriterComponent } }
    }

    #[test]
    fn two_hosts_two_policy_sets() {
        assert_eq!(EnglishApp.message(), "Hello, World!");
        assert_eq!(GermanApp.message(), "Hallo Welt!");
        EnglishApp.run();
        GermanApp.run();
    }
}

/// ## Policies as type parameters are higher-order providers
pub mod policies_as_type_parameters {
    use super::*;

    #[cgp_component(Runner)]
    pub trait CanRun {
        fn run(&self);
    }

    #[cgp_impl(new HelloWorld<W, M>)]
    #[use_provider(W: Writer)]
    #[use_provider(M: MessageProvider)]
    impl<W, M> Runner {
        fn run(&self) {
            W::write(self, M::message(self))
        }
    }

    pub struct App;

    delegate_components! {
        App {
            RunnerComponent: HelloWorld<WriteToStdout, GermanMessage>,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { RunnerComponent } }
    }

    /// The same shape on a function: the generic parameters move onto the generated trait, so the
    /// caller instantiates it the way C++ instantiates a template.
    #[cgp_fn]
    #[use_provider(W: Writer)]
    #[use_provider(M: MessageProvider)]
    pub fn hello<W, M>(&self) {
        W::write(self, M::message(self))
    }

    #[test]
    fn the_wiring_entry_names_the_instantiation() {
        App.run();
        <App as Hello<WriteToStdout, EnglishMessage>>::hello(&App);
    }
}

/// ## `#[cgp_impl]` is CRTP with the cast done for you
pub mod cgp_impl_is_crtp {
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
    pub struct Person {
        pub name: String,
    }

    delegate_components! { Person { GreeterComponent: GreetHello } }

    mod check_person {
        use super::*;
        check_components! { Person { GreeterComponent } }
    }

    #[test]
    fn the_provider_reads_the_hosts_data() {
        let person = Person {
            name: "Ada".to_owned(),
        };
        assert_eq!(person.greet(), "Hello, Ada!");
    }
}
