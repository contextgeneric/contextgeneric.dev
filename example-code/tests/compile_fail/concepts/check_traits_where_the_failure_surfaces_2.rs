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

// The check forces the failure to the wiring site and names the missing field.
check_components! {
    BrokenApp {
        StorageObjectFetcherComponent,
    }
}

fn main() {}
