//! Code from `docs/reference/macros/async_trait.md` — *`#[async_trait]`*.
//!
//! The snippet the page rejects lives under `tests/compile_fail/reference/macros/`.

/// ## Overview and Usage
///
/// The `CanFetch` trait with the attribute, implemented directly on a context the page does not
/// show, and an ignored argument, which the page says is accepted.
pub mod overview_and_usage {
    use cgp::prelude::*;

    #[async_trait]
    pub trait CanFetch {
        async fn fetch(&self, id: &str) -> Result<Vec<u8>, String>;
    }

    #[async_trait(ignored)]
    pub trait CanFetchQuietly {
        async fn fetch_quietly(&self) -> u8;
    }

    pub struct App;

    impl CanFetch for App {
        async fn fetch(&self, id: &str) -> Result<Vec<u8>, String> {
            Ok(id.as_bytes().to_vec())
        }
    }

    #[test]
    fn the_rewritten_method_is_implemented_with_async_fn() {
        assert_eq!(
            futures::executor::block_on(App.fetch("x")),
            Ok(b"x".to_vec())
        );
    }
}

/// ### Ordering with a host macro
///
/// `#[async_trait]` above and below `#[cgp_component]`, and below `#[cgp_fn]`. The page elides the
/// `#[cgp_fn]` body and names `Client` without declaring it; it is a small struct here.
pub mod ordering_with_a_host_macro {
    pub mod outermost {
        use cgp::prelude::*;

        #[async_trait]
        #[cgp_component(StorageObjectFetcher)]
        pub trait CanFetchStorageObject {
            async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, String>;
        }
    }

    pub mod below_the_component {
        use cgp::prelude::*;

        #[cgp_component(StorageObjectFetcher)]
        #[async_trait]
        pub trait CanFetchStorageObject {
            async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, String>;
        }
    }

    pub mod under_cgp_fn {
        use cgp::prelude::*;

        pub struct Client {
            pub prefix: String,
        }

        #[cgp_fn]
        #[async_trait]
        pub async fn fetch_storage_object(
            &self,
            #[implicit] storage_client: &Client,
            object_id: &str,
        ) -> Result<Vec<u8>, String> {
            // The page elides the body.
            Ok(format!("{}{object_id}", storage_client.prefix).into_bytes())
        }

        #[derive(HasField)]
        pub struct App {
            pub storage_client: Client,
        }

        #[test]
        fn the_generated_trait_runs() {
            let app = App {
                storage_client: Client {
                    prefix: "p/".to_owned(),
                },
            };
            assert_eq!(
                futures::executor::block_on(app.fetch_storage_object("o")),
                Ok(b"p/o".to_vec())
            );
        }
    }
}

/// ## Examples
///
/// The page's async component, provider, and wiring, with the check the page shows.
pub mod examples {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    #[async_trait]
    #[cgp_component(StorageObjectFetcher)]
    #[use_type(HasErrorType.Error)]
    pub trait CanFetchStorageObject {
        async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, Error>;
    }

    #[cgp_impl(new FetchFromBucket)]
    #[use_type(HasErrorType.Error)]
    impl StorageObjectFetcher {
        async fn fetch_storage_object(
            &self,
            #[implicit] bucket_id: &str,
            object_id: &str,
        ) -> Result<Vec<u8>, Error> {
            Ok(format!("{bucket_id}/{object_id}").into_bytes())
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub bucket_id: String,
    }

    delegate_components! {
        App {
            ErrorTypeProviderComponent: UseType<String>,
            StorageObjectFetcherComponent: FetchFromBucket,
        }
    }

    check_components! {
        App {
            StorageObjectFetcherComponent,
        }
    }

    #[test]
    fn the_provider_is_awaited_through_the_trait() {
        let app = App {
            bucket_id: "b".to_owned(),
        };
        assert_eq!(
            futures::executor::block_on(app.fetch_storage_object("o")),
            Ok(b"b/o".to_vec())
        );
    }
}

/// ## Under the hood
///
/// The three-method trait whose expansion the page lists, checked with
/// `cargo cgp expand --item reference::macros::async_trait::under_the_hood`.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[async_trait]
    pub trait CanFetch {
        async fn fetch(&self, id: &str) -> Result<Vec<u8>, String>;
        async fn run(&self);
        fn sync_method(&self) -> u8;
    }
}
