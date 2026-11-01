//! Code from `docs/concepts/impl-side-dependencies.md` — *Impl-side dependencies*.

/// ## What a `where` clause costs the callers above it
pub mod what_a_where_clause_costs {
    pub trait HasName {
        fn name(&self) -> &str;
    }

    pub fn greet<Context>(context: &Context) -> String
    where
        Context: HasName,
    {
        format!("Hello, {}!", context.name())
    }

    // The bound propagates. This function never touches a name, and declares the requirement anyway.
    pub fn greet_twice<Context>(context: &Context) -> String
    where
        Context: HasName,
    {
        format!("{} {}", greet(context), greet(context))
    }
}

/// ## Moving the requirement onto the implementation
pub mod moving_the_requirement {
    pub trait HasName {
        fn name(&self) -> &str;
    }

    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    impl<Context> CanGreet for Context
    where
        Context: HasName,
    {
        fn greet(&self) -> String {
            format!("Hello, {}!", self.name())
        }
    }

    // Generic callers can require `CanGreet` without repeating `HasName`.
    pub fn greet_twice<Context>(context: &Context) -> String
    where
        Context: CanGreet,
    {
        format!("{} {}", context.greet(), context.greet())
    }
}

/// ## Two providers, two sets of requirements, one interface
pub mod two_providers_two_requirements {
    use core::cell::RefCell;

    use cgp::prelude::*;

    #[cgp_component(EmailSender)]
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    #[cgp_auto_getter]
    pub trait HasSmtpServer {
        fn smtp_server(&self) -> &str;
    }

    #[cgp_auto_getter]
    pub trait HasSentEmails {
        fn sent_emails(&self) -> &RefCell<Vec<String>>;
    }

    #[cgp_impl(new SendViaSmtp)]
    #[uses(HasSmtpServer)]
    impl EmailSender {
        fn send_email(&self, to: &str, body: &str) {
            let _ = (self.smtp_server(), to, body);
        }
    }

    #[cgp_impl(new RecordEmails)]
    #[uses(HasSentEmails)]
    impl EmailSender {
        fn send_email(&self, to: &str, body: &str) {
            self.sent_emails()
                .borrow_mut()
                .push(format!("{to}: {body}"));
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

    /// Neither requirement reaches a caller: this function bounds on the trait alone, and it
    /// accepts both contexts even though they satisfy it for entirely different reasons.
    pub fn notify<Context>(context: &Context)
    where
        Context: CanSendEmail,
    {
        context.send_email("a@b.c", "hi");
    }

    #[test]
    fn one_bound_accepts_both_contexts() {
        let test_app = TestApp {
            sent_emails: RefCell::new(Vec::new()),
        };
        notify(&test_app);
        notify(&App {
            smtp_server: "localhost".to_owned(),
        });

        assert_eq!(test_app.sent_emails.borrow().as_slice(), ["a@b.c: hi"]);
    }
}

/// ## Traits, values, and types
///
/// The value leg. The field requirement is generated from the argument name and lands on the impl.
pub mod the_value_leg {
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
}

/// ## Keeping type choices on the context
///
/// Generic forwarding functions can carry type parameters and bounds needed by the job runner.
pub mod the_type_leg_as_a_parameter {
    pub trait Store {
        type Error;
    }

    pub trait Runtime {
        type Error;
    }

    pub fn run_job<E, R, S>(store: &S, runtime: &R) -> Result<(), E>
    where
        E: From<S::Error> + From<R::Error>,
        R: Runtime,
        S: Store,
    {
        // The page omits the job execution body.
        Ok(())
    }

    // This generic forwarding layer repeats the job runner's parameters and bounds.
    pub fn run_jobs<E, R, S>(store: &S, runtime: &R) -> Result<(), E>
    where
        E: From<S::Error> + From<R::Error>,
        R: Runtime,
        S: Store,
    {
        run_job(store, runtime)
    }
}

/// ## Keeping type choices on the context
///
/// The context selects the associated error type, which remains part of the public interface.
pub mod the_type_leg_as_an_abstract_type {
    // The error component's wiring key is deliberately not in the prelude.
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    #[cgp_component(JobRunner)]
    #[use_type(HasErrorType.Error)]
    pub trait CanRunJob {
        fn run_job(&self) -> Result<(), Error>;
    }

    #[cgp_impl(new RunJobOnce)]
    #[use_type(HasErrorType.Error)]
    impl JobRunner {
        fn run_job(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    // The forwarding operation shares the context's error and requires `CanRunJob` on its impl.
    #[cgp_fn]
    #[uses(CanRunJob)]
    #[use_type(HasErrorType.Error)]
    pub fn run_jobs(&self) -> Result<(), Error> {
        self.run_job()?;
        self.run_job()
    }

    pub fn run_batch<Context>(context: &Context) -> Result<(), Context::Error>
    where
        Context: RunJobs,
    {
        context.run_jobs()
    }

    pub struct App;

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            JobRunnerComponent: RunJobOnce,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { JobRunnerComponent } }
    }

    #[test]
    fn the_forwarder_uses_the_contexts_shared_error_type() {
        let result: Result<(), String> = run_batch(&App);
        assert!(result.is_ok());
    }
}
