//! Code from `docs/comparisons/ml-modules.md` — *ML modules and modular implicits*.

/// ## Signatures and structures
pub mod signatures_and_structures {
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

    /// ## Higher-order providers are functors
    #[cgp_impl(new ScaledArea<InnerCalculator>)]
    #[use_provider(InnerCalculator: AreaCalculator)]
    impl<InnerCalculator> AreaCalculator {
        fn area(&self, #[implicit] scale_factor: f64) -> f64 {
            InnerCalculator::area(self) * scale_factor * scale_factor
        }
    }

    #[derive(HasField)]
    pub struct ScaledRectangle {
        pub width: f64,
        pub height: f64,
        pub scale_factor: f64,
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
    fn the_functor_is_applied_in_the_wiring() {
        let rectangle = ScaledRectangle {
            width: 2.0,
            height: 5.0,
            scale_factor: 3.0,
        };
        assert_eq!(rectangle.area(), 90.0);
    }
}

/// ## Abstract types
pub mod abstract_types {
    use core::fmt::Debug;

    use cgp::prelude::*;

    // The page shows CGP's own `HasErrorType`; declaring it locally shows the shape the library's
    // definition has, and the local name shadows the prelude's within this module.
    #[cgp_type]
    #[prefix(@cgp.core.error in DefaultNamespace)]
    pub trait HasErrorType {
        type Error: Debug;
    }
}

/// ## Assembling with `delegate_components!`
pub mod assembling_with_delegate_components {
    use cgp::core::error::ErrorTypeProviderComponent;
    use cgp::prelude::*;

    /// Stand-in for the AWS SDK's S3 client, which the page names as `aws_sdk_s3::Client`.
    pub mod aws_sdk_s3 {
        pub struct Client;
    }

    // The page wires this component and provider without declaring them; the declarations
    // follow the site's object-storage example.
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
            // The page elides the body; the stand-in returns the object's address instead.
            let _ = storage_client;
            Ok(format!("s3://{bucket_id}/{object_id}").into_bytes())
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub storage_client: aws_sdk_s3::Client,
        pub bucket_id: String,
    }

    delegate_components! {
        App {
            StorageObjectFetcherComponent: FetchS3Object,
            ErrorTypeProviderComponent: UseType<anyhow::Error>,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { StorageObjectFetcherComponent, ErrorTypeProviderComponent } }
    }

    #[test]
    fn the_table_is_resolved_in_any_order() {
        let app = App {
            storage_client: aws_sdk_s3::Client,
            bucket_id: "pictures".to_owned(),
        };
        assert_eq!(
            app.fetch_storage_object("alice.png").unwrap(),
            b"s3://pictures/alice.png"
        );
    }
}
