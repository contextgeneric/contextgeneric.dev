//! `docs/overview.md`, *Type-Safe Wiring*: `App` lacks the `bucket_id` field `FetchS3Object` reads,
//! so the check rejects the wiring. The S3 client is a stand-in, and the provider's body, which the
//! page elides, is filled in.

use cgp::prelude::*;

pub mod aws_sdk_s3 {
    pub struct Client;

    impl Client {
        pub fn get_object(
            &self,
            bucket_id: &str,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            Ok(format!("s3://{bucket_id}/{object_id}").into_bytes())
        }
    }
}

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
        storage_client.get_object(bucket_id, object_id)
    }
}

#[derive(HasField)]
pub struct App {
    pub storage_client: aws_sdk_s3::Client,
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

fn main() {}
