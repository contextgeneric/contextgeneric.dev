pub trait CanFetchStorageObject {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}

pub trait HasS3Client {}
pub trait HasLocalDir {}

impl<T: HasS3Client> CanFetchStorageObject for T { fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> { Ok(Vec::new()) } }
impl<T: HasLocalDir> CanFetchStorageObject for T { fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> { Ok(Vec::new()) } }   // error[E0119]

fn main() {}
