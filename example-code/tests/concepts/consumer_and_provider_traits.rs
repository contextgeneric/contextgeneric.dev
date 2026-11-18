//! Code from `docs/concepts/consumer-and-provider-traits.md` — *Consumer and provider traits*.
//!
//! Module names follow the page's headings. Rejected code lives in trybuild fixtures. The two
//! generated-code listings are assembled by hand in [`how_a_call_finds_its_provider`]; that checks
//! their structure, while macro expansion establishes what the library generates. The page's S3
//! client is a stand-in defined below, because the page shows only its type.

/// Stand-in for the AWS SDK's S3 client, which the page names as `aws_sdk_s3::Client`.
pub mod aws_sdk_s3 {
    pub struct Client;
}

/// ## What an ordinary Rust trait does, and where it stops
///
/// The arrangement the page opens on: two application types, two behaviours, no CGP. It compiles,
/// which is what the paragraph after it says.
///
/// The section's second block does not, and is the reason the first one does not scale. The two
/// marker traits it bounds on are undeclared on the page, so they are declared there; without them
/// the failure would be an unresolved name rather than the overlap it is meant to show:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/consumer_and_provider_traits_what_an_ordinary_rust_trait_does.rs`.
pub mod what_an_ordinary_rust_trait_does {
    use super::aws_sdk_s3;

    pub trait CanFetchStorageObject {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    pub struct App {
        pub storage_client: aws_sdk_s3::Client,
        pub bucket_id: String,
    }

    pub struct TestApp {
        pub local_dir: String,
    }

    impl CanFetchStorageObject for App {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
            // fetch `object_id` from `self.bucket_id` through `self.storage_client`
            // The page elides the request; the stand-in client has none to make.
            let _ = (&self.storage_client, &self.bucket_id, object_id);
            Ok(Vec::new())
        }
    }

    impl CanFetchStorageObject for TestApp {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(std::fs::read(format!("{}/{object_id}", self.local_dir))?)
        }
    }
}

/// ## Splitting one trait in two
///
/// The trait pair written out by hand, and two implementations targeting named types. The page
/// elides both bodies as `/* ... */`, which would leave the impls empty and the required method
/// missing, so they are filled in here.
pub mod splitting_one_trait_in_two {
    // what callers use
    pub trait CanFetchStorageObject {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    // what implementations target
    pub trait StorageObjectFetcher<Context> {
        fn fetch_storage_object(
            context: &Context,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>>;
    }

    pub trait HasS3Client {}
    pub trait HasLocalDir {}

    pub struct FetchS3Object;
    pub struct FetchLocalObject;

    impl<Context: HasS3Client> StorageObjectFetcher<Context> for FetchS3Object {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    impl<Context: HasLocalDir> StorageObjectFetcher<Context> for FetchLocalObject {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(Vec::new())
        }
    }
}

/// ## How a call finds its provider
///
/// The section's three listings — the wiring table, the consumer blanket impl, and the provider
/// blanket impl — assembled into one program. Because the last two are what `#[cgp_component]`
/// *generates*, they are written by hand here rather than derived, which is the point: the page
/// claims a component is these pieces, and a hand-rolled component built from exactly them resolves
/// a call and passes a check.
///
/// **This proves the listings are structurally sound, not that they are textually current.** The
/// authority on what the macro emits is `cargo cgp expand --lib --item …`, which is also what the
/// page's own note about reserved names and the spelled-out delegate refers to. Check the text
/// against `expand`; check the shape against this module.
pub mod how_a_call_finds_its_provider {
    use cgp::core::component::DelegateComponent;
    use cgp::core::component::IsProviderFor;
    use cgp::prelude::*;

    pub trait CanFetchStorageObject {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    pub trait StorageObjectFetcher<Context>:
        IsProviderFor<StorageObjectFetcherComponent, Context, ()>
    {
        fn fetch_storage_object(
            context: &Context,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>>;
    }

    // The key. The page calls it "a marker generated alongside the two traits".
    pub struct StorageObjectFetcherComponent;

    pub struct FetchS3Object;
    pub struct FetchLocalObject;

    impl<Context> StorageObjectFetcher<Context> for FetchS3Object {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(b"s3".to_vec())
        }
    }

    impl<Context> StorageObjectFetcher<Context> for FetchLocalObject {
        fn fetch_storage_object(context: &Context, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Ok(b"local".to_vec())
        }
    }

    // Provider macros generate these checking implementations with the provider's bounds.
    impl<Context> IsProviderFor<StorageObjectFetcherComponent, Context, ()> for FetchS3Object {}
    impl<Context> IsProviderFor<StorageObjectFetcherComponent, Context, ()> for FetchLocalObject {}

    pub struct App;
    pub struct TestApp;

    delegate_components! { App     { StorageObjectFetcherComponent: FetchS3Object } }
    delegate_components! { TestApp { StorageObjectFetcherComponent: FetchLocalObject } }

    // The first generated implementation: a context that implements the provider trait for itself
    // gets the consumer trait.
    impl<Context> CanFetchStorageObject for Context
    where
        Context: StorageObjectFetcher<Context>,
    {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
            Context::fetch_storage_object(self, object_id)
        }
    }

    // The second: anything with a table entry inherits the provider trait from what the entry names.
    impl<Context, Provider> StorageObjectFetcher<Context> for Provider
    where
        Provider: DelegateComponent<StorageObjectFetcherComponent>
            + IsProviderFor<StorageObjectFetcherComponent, Context, ()>,
        Provider::Delegate: StorageObjectFetcher<Context>,
    {
        fn fetch_storage_object(
            context: &Context,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            Provider::Delegate::fetch_storage_object(context, object_id)
        }
    }

    /// The three-step resolution the page traces, asserted as a bound rather than described.
    #[test]
    fn a_call_resolves_through_the_table() {
        fn resolves<Context: CanFetchStorageObject>() {}

        resolves::<App>();
        resolves::<TestApp>();

        assert_eq!(App.fetch_storage_object("alice.png").unwrap(), b"s3");
        assert_eq!(TestApp.fetch_storage_object("alice.png").unwrap(), b"local");
    }
}

/// ## Writing it
///
/// The same component as [`how_a_call_finds_its_provider`], written the way it is actually written:
/// the two traits, the marker, and both blanket impls come from `#[cgp_component]`, and each
/// provider is a `#[cgp_impl]` block in consumer-trait shape.
pub mod writing_it {
    use cgp::prelude::*;

    use super::aws_sdk_s3;

    #[cgp_component(StorageObjectFetcher)]
    pub trait CanFetchStorageObject {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    #[cgp_impl(new FetchS3Object)]
    impl StorageObjectFetcher {
        fn fetch_storage_object(
            &self,
            #[implicit] storage_client: &aws_sdk_s3::Client,
            #[implicit] bucket_id: &str,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            // fetch `object_id` from `bucket_id` through `storage_client`
            // The page elides the request; the stand-in returns the object's address instead.
            let _ = storage_client;
            Ok(format!("s3://{bucket_id}/{object_id}").into_bytes())
        }
    }

    #[cgp_impl(new FetchLocalObject)]
    impl StorageObjectFetcher {
        fn fetch_storage_object(
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

    pub fn fetch_profile_picture(
        app: &impl CanFetchStorageObject,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        app.fetch_storage_object(object_id)
    }

    /// A generic caller needs only the consumer trait, while each context's selected provider
    /// reads its own fields.
    #[test]
    fn generic_caller_uses_each_selected_provider() {
        let dir = std::env::temp_dir().join("cgp-consumer-and-provider-traits");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("alice.png"), b"picture").unwrap();

        let test_app = TestApp {
            local_dir: dir.to_str().unwrap().to_owned(),
        };
        assert_eq!(fetch_profile_picture(&test_app, "alice.png").unwrap(), b"picture");

        let app = App {
            storage_client: aws_sdk_s3::Client,
            bucket_id: "pictures".to_owned(),
        };
        assert_eq!(
            fetch_profile_picture(&app, "alice.png").unwrap(),
            b"s3://pictures/alice.png"
        );
    }
}
