//! Code from `docs/concepts/impl-side-dependencies.md` — *Impl-side dependencies*.

/// ## The cost of a `where` clause
pub mod the_cost_of_a_where_clause {
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

/// ## Providers with different requirements
pub mod providers_with_different_requirements {
    use cgp::prelude::*;

    /// Stand-in for the AWS SDK's S3 client, which the page names as `aws_sdk_s3::Client`.
    pub mod aws_sdk_s3 {
        pub struct Client;
    }

    #[cgp_component(StorageObjectFetcher)]
    pub trait CanFetchStorageObject {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    // The page names these getters and their methods in prose without declaring them.
    #[cgp_auto_getter]
    pub trait HasS3Bucket {
        fn storage_client(&self) -> &aws_sdk_s3::Client;

        fn bucket_id(&self) -> &str;
    }

    #[cgp_auto_getter]
    pub trait HasLocalDir {
        fn local_dir(&self) -> &str;
    }

    #[cgp_impl(new FetchS3Object)]
    #[uses(HasS3Bucket)]
    impl StorageObjectFetcher {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
            // The page elides the request; the stand-in returns the object's address instead.
            let _ = self.storage_client();
            Ok(format!("s3://{}/{object_id}", self.bucket_id()).into_bytes())
        }
    }

    #[cgp_impl(new FetchLocalObject)]
    #[uses(HasLocalDir)]
    impl StorageObjectFetcher {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(std::fs::read(format!("{}/{object_id}", self.local_dir()))?)
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub storage_client: aws_sdk_s3::Client,
        pub bucket_id: String,
    }

    #[derive(HasField)]
    pub struct TestApp {
        pub local_dir: String,
    }

    delegate_components! { App     { StorageObjectFetcherComponent: FetchS3Object } }
    delegate_components! { TestApp { StorageObjectFetcherComponent: FetchLocalObject } }

    mod check_app {
        use super::*;
        check_components! { App { StorageObjectFetcherComponent } }
    }

    mod check_test_app {
        use super::*;
        check_components! { TestApp { StorageObjectFetcherComponent } }
    }

    /// Neither requirement reaches a caller: this function bounds on the trait alone, and it
    /// accepts both contexts even though they satisfy it for entirely different reasons.
    pub fn load_picture<Context>(context: &Context) -> anyhow::Result<Vec<u8>>
    where
        Context: CanFetchStorageObject,
    {
        context.fetch_storage_object("alice.png")
    }

    #[test]
    fn one_bound_accepts_both_contexts() {
        let dir = std::env::temp_dir().join("cgp-impl-side-dependencies");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("alice.png"), b"picture").unwrap();

        let test_app = TestApp {
            local_dir: dir.to_str().unwrap().to_owned(),
        };
        assert_eq!(load_picture(&test_app).unwrap(), b"picture");

        let app = App {
            storage_client: aws_sdk_s3::Client,
            bucket_id: "pictures".to_owned(),
        };
        assert_eq!(load_picture(&app).unwrap(), b"s3://pictures/alice.png");
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
