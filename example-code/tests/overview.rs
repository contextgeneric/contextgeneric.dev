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
