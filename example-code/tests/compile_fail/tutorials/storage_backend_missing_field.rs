use std::collections::HashMap;
use std::path::Path;

use anyhow::anyhow;
use cgp::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub profile_picture_object_id: Option<String>,
}

#[derive(HasField)]
pub struct App {
    pub database: HashMap<UserId, User>,
}

#[cgp_fn]
#[async_trait]
pub async fn get_user(
    &self,
    #[implicit] database: &HashMap<UserId, User>,
    user_id: &UserId,
) -> anyhow::Result<User> {
    database
        .get(user_id)
        .cloned()
        .ok_or_else(|| anyhow!("no user with ID {}", user_id.0))
}

#[async_trait]
#[cgp_component(StorageObjectFetcher)]
pub trait CanFetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}

#[cgp_impl(new FetchLocalObject)]
impl StorageObjectFetcher {
    async fn fetch_storage_object(
        &self,
        #[implicit] local_dir: &str,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let data = tokio::fs::read(Path::new(local_dir).join(object_id)).await?;
        Ok(data)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio::fs::create_dir_all("pictures").await?;
    tokio::fs::write("pictures/alice.png", b"not really a PNG").await?;

    let app = App {
        database: HashMap::from([(
            UserId(1),
            User {
                name: "Alice".to_owned(),
                profile_picture_object_id: Some("alice.png".to_owned()),
            },
        )]),
    };

    match app.get_user_profile_picture(&UserId(1)).await? {
        Some(picture) => println!("Found a {}-byte picture", picture.len()),
        None => println!("No picture"),
    }

    Ok(())
}

#[cgp_fn]
#[async_trait]
#[uses(GetUser, CanFetchStorageObject)]
pub async fn get_user_profile_picture(
    &self,
    user_id: &UserId,
) -> anyhow::Result<Option<Vec<u8>>> {
    let user = self.get_user(user_id).await?;

    match user.profile_picture_object_id {
        Some(object_id) => Ok(Some(self.fetch_storage_object(&object_id).await?)),
        None => Ok(None),
    }
}

delegate_components! {
    App {
        StorageObjectFetcherComponent: FetchLocalObject,
    }
}

check_components! {
    App {
        StorageObjectFetcherComponent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(HasField)]
    pub struct TestApp {
        pub database: HashMap<UserId, User>,
        pub objects: HashMap<String, Vec<u8>>,
    }

    #[cgp_impl(new FetchMemoryObject)]
    impl StorageObjectFetcher {
        async fn fetch_storage_object(
            &self,
            #[implicit] objects: &HashMap<String, Vec<u8>>,
            object_id: &str,
        ) -> anyhow::Result<Vec<u8>> {
            objects
                .get(object_id)
                .cloned()
                .ok_or_else(|| anyhow!("no object named {object_id}"))
        }
    }

    delegate_components! {
        TestApp {
            StorageObjectFetcherComponent: FetchMemoryObject,
        }
    }

    check_components! {
        TestApp {
            StorageObjectFetcherComponent,
        }
    }

    #[tokio::test]
    async fn fetches_the_picture_from_memory() -> anyhow::Result<()> {
        let app = TestApp {
            database: HashMap::from([(
                UserId(1),
                User {
                    name: "Alice".to_owned(),
                    profile_picture_object_id: Some("alice.png".to_owned()),
                },
            )]),
            objects: HashMap::from([("alice.png".to_owned(), b"fake image".to_vec())]),
        };

        let picture = app.get_user_profile_picture(&UserId(1)).await?;
        assert_eq!(picture, Some(b"fake image".to_vec()));

        Ok(())
    }
}

// `docs/tutorials/storage-backend.md`, *Check the wiring*: the page's finished `src/main.rs` with the
// `local_dir` field deleted from `App` and from `main`, as the page asks. `FetchLocalObject` still
// reads the field, so the check rejects `App`'s wiring. The comment sits at the end so the file's
// line numbers match the reader's.
