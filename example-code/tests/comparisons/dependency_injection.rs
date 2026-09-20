//! Code from `docs/comparisons/dependency-injection.md` — *Dependency injection*.

/// ## Dependency injection without a framework (Rust)
pub mod without_a_framework {
    pub trait StorageClient {
        fn fetch(&self, object_id: &str) -> Vec<u8>;
    }

    pub struct ProfilePictureService<S: StorageClient> {
        pub storage: S,
    }
}

/// ## Impl-side dependencies are the injected constructor parameters
pub mod impl_side_dependencies_are_constructor_parameters {
    use cgp::prelude::*;

    // Types the page names without declaring.
    pub struct PostgresDb;
    pub struct Email(pub String);
    #[derive(Debug, PartialEq)]
    pub struct User {
        pub username: String,
    }
    #[derive(Debug, PartialEq)]
    pub enum Error {
        InvalidUsername,
    }
    #[derive(Debug, PartialEq, PartialOrd)]
    pub struct Probability(f64);
    impl Probability {
        pub fn new(value: f64) -> Self {
            Probability(value)
        }
    }

    #[cgp_component(UsernameCensor)]
    pub trait CanCensorUsername {
        fn username_is_censored(&self, username: &str) -> Probability;
    }

    #[cgp_impl(new CensorByWordList)]
    impl UsernameCensor {
        fn username_is_censored(&self, username: &str) -> Probability {
            if username.contains("spam") {
                Probability::new(0.99)
            } else {
                Probability::new(0.0)
            }
        }
    }

    #[cgp_component(UserManager)]
    pub trait CanManageUser {
        fn create_user(&self, username: &str, email: &Email) -> Result<User, Error>;
    }

    #[cgp_impl(new PostgresUserManager)]
    #[uses(CanCensorUsername)]
    impl UserManager {
        fn create_user(
            &self,
            #[implicit] database: &PostgresDb,
            username: &str,
            email: &Email,
        ) -> Result<User, Error> {
            if self.username_is_censored(username) > Probability::new(0.8) {
                return Err(Error::InvalidUsername);
            }
            // The page elides the insert through `database`.
            let _ = (database, email);
            Ok(User {
                username: username.to_owned(),
            })
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub database: PostgresDb,
    }

    delegate_components! {
        App {
            UserManagerComponent: PostgresUserManager,
            UsernameCensorComponent: CensorByWordList,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { UserManagerComponent } }
    }

    #[test]
    fn the_dependencies_are_supplied_by_the_context() {
        let app = App {
            database: PostgresDb,
        };
        let email = Email("ada@example.com".to_owned());
        assert!(app.create_user("ada", &email).is_ok());
        assert_eq!(
            app.create_user("spam-bot", &email),
            Err(Error::InvalidUsername)
        );
    }
}

/// ## Wiring is the container configuration
/// ## Checking replaces the container's startup validation
pub mod wiring_is_the_container_configuration {
    use cgp::prelude::*;

    #[cgp_component(StorageObjectFetcher)]
    pub trait CanFetchStorageObject {
        fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
    }

    #[cgp_impl(new FetchS3Object)]
    impl StorageObjectFetcher {
        fn fetch_storage_object(
            &self,
            #[implicit] s3_bucket: &str,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            // The page elides the request to S3.
            Ok(format!("s3://{s3_bucket}/{object_id}").into_bytes())
        }
    }

    #[cgp_impl(new FetchGCloudObject)]
    impl StorageObjectFetcher {
        fn fetch_storage_object(
            &self,
            #[implicit] gcloud_bucket: &str,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            // The page elides the request to Google Cloud Storage.
            Ok(format!("gs://{gcloud_bucket}/{object_id}").into_bytes())
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub s3_bucket: String,
    }

    #[derive(HasField)]
    pub struct GCloudApp {
        pub gcloud_bucket: String,
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

    mod check_app {
        use super::*;
        check_components! {
            App {
                StorageObjectFetcherComponent,
            }
        }
    }

    mod check_gcloud_app {
        use super::*;
        check_components! { GCloudApp { StorageObjectFetcherComponent } }
    }

    #[test]
    fn two_deployments_two_bindings() {
        let app = App {
            s3_bucket: "pictures".to_owned(),
        };
        let gcloud = GCloudApp {
            gcloud_bucket: "pictures".to_owned(),
        };
        assert_eq!(
            app.fetch_storage_object("ada.png").unwrap(),
            b"s3://pictures/ada.png".to_vec()
        );
        assert_eq!(
            gcloud.fetch_storage_object("ada.png").unwrap(),
            b"gs://pictures/ada.png".to_vec()
        );
    }
}
