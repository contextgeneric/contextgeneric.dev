//! Code from `docs/overview.md`, the feature tour.
//!
//! The Overview is a standalone page rather than a section, so it gets its own test binary, as the
//! Quickstart and the front page do. One module per heading, in the page's order. The headline
//! features run the profile-picture scenario under the names the `cgp` README uses; the clients and
//! the pool are stand-ins defined below, because the page shows only their types.

/// Stand-in for the AWS SDK's S3 client, which the page names as `aws_sdk_s3::Client`.
pub mod aws_sdk_s3 {
    pub struct Client;

    impl Client {
        pub async fn get_object(
            &self,
            bucket_id: &str,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            Ok(format!("s3://{bucket_id}/{object_id}").into_bytes())
        }
    }
}

/// Stand-in for the Google Cloud Storage client, which the page names as
/// `google_cloud_storage::client::Storage`.
pub mod google_cloud_storage {
    pub mod client {
        pub struct Storage;

        impl Storage {
            pub async fn read_object(
                &self,
                bucket_id: &str,
                object_id: &str,
            ) -> anyhow::Result<Vec<u8>> {
                Ok(format!("gs://{bucket_id}/{object_id}").into_bytes())
            }
        }
    }
}

/// ## One Interface, Many Implementations
pub mod one_interface_many_implementations {
    use cgp::prelude::*;

    use super::{aws_sdk_s3, google_cloud_storage};

    #[async_trait]
    #[cgp_component(StorageObjectFetcher)]
    pub trait CanFetchStorageObject {
        async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    #[cgp_impl(new FetchS3Object)]
    impl StorageObjectFetcher {
        async fn fetch_storage_object(
            &self,
            #[implicit] storage_client: &aws_sdk_s3::Client,
            #[implicit] bucket_id: &str,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            // The page elides the request to Amazon S3.
            storage_client.get_object(bucket_id, object_id).await
        }
    }

    // The page names this provider in prose.
    #[cgp_impl(new FetchGCloudObject)]
    impl StorageObjectFetcher {
        async fn fetch_storage_object(
            &self,
            #[implicit] storage_client: &google_cloud_storage::client::Storage,
            #[implicit] bucket_id: &str,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            storage_client.read_object(bucket_id, object_id).await
        }
    }

    // The page names this provider in prose.
    #[cgp_impl(new FetchLocalObject)]
    impl StorageObjectFetcher {
        async fn fetch_storage_object(
            &self,
            #[implicit] local_dir: &str,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            Ok(std::fs::read(format!("{local_dir}/{object_id}"))?)
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub storage_client: aws_sdk_s3::Client,
        pub bucket_id: String,
    }

    #[derive(HasField)]
    pub struct GCloudApp {
        pub storage_client: google_cloud_storage::client::Storage,
        pub bucket_id: String,
    }

    #[derive(HasField)]
    pub struct TestApp {
        pub local_dir: String,
    }

    delegate_components! {
        App {
            StorageObjectFetcherComponent: FetchS3Object,
        }
    }

    delegate_components! {
        GCloudApp {
            StorageObjectFetcherComponent: FetchGCloudObject,
        }
    }

    delegate_components! {
        TestApp {
            StorageObjectFetcherComponent: FetchLocalObject,
        }
    }

    mod check_contexts {
        use super::*;
        check_components! { App { StorageObjectFetcherComponent } }
        check_components! { #[check_trait(CheckGCloudApp)] GCloudApp { StorageObjectFetcherComponent } }
        check_components! { #[check_trait(CheckTestApp)] TestApp { StorageObjectFetcherComponent } }
    }

    #[test]
    fn each_context_fetches_through_its_own_provider() {
        use futures::executor::block_on;

        let app = App {
            storage_client: aws_sdk_s3::Client,
            bucket_id: "pictures".to_owned(),
        };
        let gcloud = GCloudApp {
            storage_client: google_cloud_storage::client::Storage,
            bucket_id: "pictures".to_owned(),
        };
        let test = TestApp {
            local_dir: env!("CARGO_MANIFEST_DIR").to_owned(),
        };

        assert_eq!(
            block_on(app.fetch_storage_object("ada.png")).unwrap(),
            b"s3://pictures/ada.png".to_vec()
        );
        assert_eq!(
            block_on(gcloud.fetch_storage_object("ada.png")).unwrap(),
            b"gs://pictures/ada.png".to_vec()
        );
        assert!(block_on(test.fetch_storage_object("Cargo.toml")).is_ok());
        assert!(block_on(test.fetch_storage_object("no-such-object")).is_err());
    }
}

/// ## Zero-Cost Abstraction
///
/// The page shows the impl that `App`'s wiring amounts to. It cannot sit beside the wiring, which
/// already implements the trait for `App`, so this module defines an unwired `App` of its own.
pub mod zero_cost_abstraction {
    use cgp::prelude::*;

    use super::aws_sdk_s3;
    use super::one_interface_many_implementations::{
        CanFetchStorageObject, FetchS3Object, StorageObjectFetcher,
    };

    #[derive(HasField)]
    pub struct App {
        pub storage_client: aws_sdk_s3::Client,
        pub bucket_id: String,
    }

    impl CanFetchStorageObject for App {
        async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
            <FetchS3Object as StorageObjectFetcher<App>>::fetch_storage_object(self, object_id)
                .await
        }
    }

    #[test]
    fn the_hand_written_impl_calls_the_provider() {
        use futures::executor::block_on;

        let app = App {
            storage_client: aws_sdk_s3::Client,
            bucket_id: "pictures".to_owned(),
        };
        assert_eq!(
            block_on(app.fetch_storage_object("ada.png")).unwrap(),
            b"s3://pictures/ada.png".to_vec()
        );
    }
}

/// ## Type-Safe Wiring
///
/// The page shows `App` without its `bucket_id` field, which the check rejects: the trybuild
/// fixture `tests/compile_fail/overview/type_safe_wiring_missing_field.rs`. Here the field is
/// present, so the same check passes.
pub mod type_safe_wiring {
    use cgp::prelude::*;

    use super::aws_sdk_s3;
    use super::one_interface_many_implementations::{FetchS3Object, StorageObjectFetcherComponent};

    #[derive(HasField)]
    pub struct App {
        pub storage_client: aws_sdk_s3::Client,
        pub bucket_id: String,
    }

    delegate_components! {
        App {
            StorageObjectFetcherComponent: FetchS3Object,
        }
    }

    check_components! {
        App {
            StorageObjectFetcherComponent,
        }
    }
}

/// ## Abstract Over Every Dependency
pub mod abstract_over_every_dependency {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    #[async_trait]
    #[cgp_component(StorageObjectFetcher)]
    #[use_type(HasErrorType.Error)]
    pub trait CanFetchStorageObject {
        async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, Error>;
    }

    #[cgp_impl(new FetchLocalObject)]
    #[uses(CanRaiseError<std::io::Error>)]
    #[use_type(HasErrorType.Error)]
    impl StorageObjectFetcher {
        async fn fetch_storage_object(
            &self,
            #[implicit] local_dir: &str,
            object_id: &str,
        ) -> Result<Vec<u8>, Error> {
            std::fs::read(format!("{local_dir}/{object_id}")).map_err(Self::raise_error)
        }
    }

    #[derive(HasField)]
    pub struct TestApp {
        pub local_dir: String,
    }

    delegate_components! {
        TestApp {
            open ErrorRaiserComponent;

            ErrorTypeProviderComponent: UseType<anyhow::Error>,
            @ErrorRaiserComponent.std::io::Error: RaiseFrom,
            StorageObjectFetcherComponent: FetchLocalObject,
        }
    }

    mod check_test_app {
        use super::*;
        check_components! { TestApp { StorageObjectFetcherComponent } }
    }

    #[test]
    fn the_context_chooses_the_error_type() {
        use futures::executor::block_on;

        let test = TestApp {
            local_dir: env!("CARGO_MANIFEST_DIR").to_owned(),
        };
        assert!(block_on(test.fetch_storage_object("Cargo.toml")).is_ok());
        let error: anyhow::Error =
            block_on(test.fetch_storage_object("no-such-object")).unwrap_err();
        assert!(error.downcast_ref::<std::io::Error>().is_some());
    }
}

/// ## Still Ordinary Rust
pub mod still_ordinary_rust {
    use cgp::prelude::*;

    /// Stand-in for the `sqlx` pool the page names.
    pub struct PgPool;

    pub struct UserId(pub u64);

    pub struct User {
        pub name: String,
        pub profile_picture_object_id: Option<String>,
    }

    #[cgp_fn]
    #[async_trait]
    pub async fn get_user(
        &self,
        #[implicit] database: &PgPool,
        user_id: &UserId,
    ) -> anyhow::Result<User> {
        // The page elides the query on `database`.
        let _ = database;
        Ok(User {
            name: format!("user {}", user_id.0),
            profile_picture_object_id: None,
        })
    }

    #[derive(HasField)]
    pub struct App {
        pub database: PgPool,
    }

    #[test]
    fn a_context_with_the_field_gains_the_method() {
        use futures::executor::block_on;

        let app = App { database: PgPool };
        let user = block_on(app.get_user(&UserId(7))).unwrap();
        assert_eq!(user.name, "user 7");
        assert!(user.profile_picture_object_id.is_none());
    }
}

/// ## Abstract types
pub mod abstract_types {
    /// The plain-Rust version the page shows first: the error type is a parameter of every layer.
    pub mod before {
        pub trait Load<Error> {
            fn load(&self, path: &str) -> Result<String, Error>;
        }

        pub fn load_profile<Context, Error>(context: &Context, user: &str) -> Result<String, Error>
        where
            Context: Load<Error>,
        {
            context.load(&format!("profiles/{user}"))
        }
    }

    /// The CGP version: the context chooses the error type, and the function names neither.
    pub mod after {
        use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
        use cgp::extra::error::RaiseFrom;
        use cgp::prelude::*;

        #[cgp_component(Loader)]
        #[use_type(HasErrorType.Error)]
        pub trait CanLoad {
            fn load(&self, path: &str) -> Result<String, Error>;
        }

        #[cgp_fn]
        #[uses(CanLoad)]
        #[use_type(HasErrorType.Error)]
        pub fn load_profile(&self, user: &str) -> Result<String, Error> {
            self.load(&format!("profiles/{user}"))
        }

        // The page leaves the provider and the context to the Concepts page it links.
        #[cgp_impl(new LoadOrFail)]
        #[uses(CanRaiseError<String>)]
        #[use_type(HasErrorType.Error)]
        impl Loader {
            fn load(&self, path: &str) -> Result<String, Error> {
                if path.ends_with('/') {
                    return Err(Self::raise_error("empty name".to_owned()));
                }
                Ok(format!("contents of {path}"))
            }
        }

        pub struct App;

        delegate_components! {
            App {
                open ErrorRaiserComponent;

                ErrorTypeProviderComponent: UseType<String>,
                LoaderComponent: LoadOrFail,
                @ErrorRaiserComponent.String: RaiseFrom,
            }
        }

        mod check_app {
            use super::*;
            check_components! { App { LoaderComponent } }
        }

        #[test]
        fn the_function_never_names_the_error_type() {
            assert_eq!(App.load_profile("ada").unwrap(), "contents of profiles/ada");
            assert_eq!(App.load_profile("").unwrap_err(), "empty name");
        }
    }
}

/// ## Extensible records and variants
pub mod extensible_records_and_variants {
    use cgp::core::field::impls::CanBuildFrom;
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub struct AppConfig {
        pub database_url: String,
        pub http_timeout: u32,
    }

    #[derive(CgpData)]
    pub struct DatabaseConfig {
        pub database_url: String,
    }

    #[derive(CgpData)]
    pub struct HttpConfig {
        pub http_timeout: u32,
    }

    #[test]
    fn independent_pieces_build_the_whole() {
        let config = AppConfig::builder()
            .build_from(DatabaseConfig {
                database_url: "postgres://localhost".to_owned(),
            })
            .build_from(HttpConfig { http_timeout: 30 })
            .finalize_build();

        assert_eq!(config.database_url, "postgres://localhost");
        assert_eq!(config.http_timeout, 30);
    }
}

/// ## Composable handlers
pub mod composable_handlers {
    use core::marker::PhantomData;

    use cgp::extra::handler::{CanCompute, ComputerComponent, PipeHandlers};
    use cgp::prelude::*;

    #[cgp_computer]
    fn double(x: u64) -> u64 {
        x * 2
    }

    #[cgp_computer]
    fn increment(x: u64) -> u64 {
        x + 1
    }

    pub struct App;

    delegate_components! {
        App {
            ComputerComponent: PipeHandlers<Product![Double, Increment]>,
        }
    }

    #[test]
    fn the_steps_run_in_order() {
        assert_eq!(App.compute(PhantomData::<()>, 5u64), 11);
    }
}

/// ## Namespaces
pub mod namespaces {
    use cgp::prelude::*;

    cgp_namespace! { new AppNamespace {} }

    #[cgp_component(Greeter)]
    #[prefix(@app.GreeterComponent in AppNamespace)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_component(Farewell)]
    #[prefix(@app.FarewellComponent in AppNamespace)]
    pub trait CanSayGoodbye {
        fn goodbye(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello!".to_owned()
        }
    }

    #[cgp_impl(new GreetQuietly)]
    impl Greeter {
        fn greet(&self) -> String {
            "hello".to_owned()
        }
    }

    #[cgp_impl(new SayGoodbye)]
    impl Farewell {
        fn goodbye(&self) -> String {
            "Goodbye!".to_owned()
        }
    }

    cgp_namespace! {
        new AppDefaults: AppNamespace {
            @app.FarewellComponent: SayGoodbye,
        }
    }

    pub struct App;
    pub struct TestApp;

    delegate_components! {
        App {
            namespace AppDefaults;
            @app.GreeterComponent: GreetHello,
        }
    }

    delegate_components! {
        TestApp {
            namespace AppDefaults;
            @app.GreeterComponent: GreetQuietly,
        }
    }

    mod check_contexts {
        use super::*;
        check_components! { App { GreeterComponent, FarewellComponent } }
        check_components! { #[check_trait(CheckTestApp)] TestApp { GreeterComponent, FarewellComponent } }
    }

    #[test]
    fn the_contexts_share_the_namespace_and_differ_by_one_line() {
        assert_eq!(App.greet(), "Hello!");
        assert_eq!(TestApp.greet(), "hello");
        assert_eq!(App.goodbye(), TestApp.goodbye());
    }
}

/// ## Higher-order providers
pub mod higher_order_providers {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanCalculateArea {
        fn area(&self) -> f64;
    }

    #[cgp_impl(new RectangleArea)]
    impl AreaCalculator {
        fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
            width * height
        }
    }

    #[cgp_impl(new ScaledArea<Inner>)]
    #[use_provider(Inner: AreaCalculator)]
    impl<Inner> AreaCalculator {
        fn area(&self, #[implicit] scale: f64) -> f64 {
            Inner::area(self) * scale * scale
        }
    }

    #[derive(HasField)]
    pub struct ScaledRectangle {
        pub width: f64,
        pub height: f64,
        pub scale: f64,
    }

    delegate_components! {
        ScaledRectangle {
            AreaCalculatorComponent: ScaledArea<RectangleArea>,
        }
    }

    mod check_scaled_rectangle {
        use super::*;
        check_components! { ScaledRectangle { AreaCalculatorComponent } }
    }

    #[test]
    fn the_wrapper_scales_the_inner_area() {
        let shape = ScaledRectangle {
            width: 3.0,
            height: 4.0,
            scale: 2.0,
        };
        assert_eq!(shape.area(), 48.0);
    }
}
