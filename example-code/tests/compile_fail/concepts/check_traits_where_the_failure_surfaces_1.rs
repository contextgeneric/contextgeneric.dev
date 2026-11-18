use cgp::prelude::*;

#[cgp_component(StorageObjectFetcher)]
pub trait CanFetchStorageObject {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
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

// The mistake: `BrokenApp` has no `local_dir` field, though `FetchLocalObject` reads one.
#[derive(HasField)]
pub struct BrokenApp {
    pub bucket_id: String,
}

delegate_components! { BrokenApp { StorageObjectFetcherComponent: FetchLocalObject } }

fn main() {
    let app = BrokenApp {
        bucket_id: "pictures".to_owned(),
    };
    // The hidden failure: `E0599` at the call site, naming nothing about the missing field.
    let _ = app.fetch_storage_object("alice.png");
}
