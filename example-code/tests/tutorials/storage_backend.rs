//! `docs/tutorials/storage-backend.md`
//!
//! The page builds one `src/main.rs` in steps, so each module below is that file as it stands at the
//! end of the heading it names, assembled from the page's blocks in order and a later module repeating
//! what an earlier one showed. Each module's `main` is the page's, compiled but not run, since it writes
//! a `pictures` folder; a test beside it runs the same calls in a temporary directory. The deliberate
//! mistake in *Check the wiring* is a compile-fail fixture holding the whole finished file with the
//! field removed, at `tests/compile_fail/tutorials/storage_backend_missing_field.rs`.

/// *Read the application's data with functions*.
mod read_with_functions {
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
        pub local_dir: String,
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

    #[cgp_fn]
    #[async_trait]
    pub async fn fetch_storage_object(
        &self,
        #[implicit] local_dir: &str,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let data = tokio::fs::read(Path::new(local_dir).join(object_id)).await?;
        Ok(data)
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
            local_dir: "pictures".to_owned(),
        };

        let user = app.get_user(&UserId(1)).await?;
        let picture = app.fetch_storage_object("alice.png").await?;
        println!("{} has a {}-byte picture", user.name, picture.len());

        Ok(())
    }

    #[tokio::test]
    async fn main_prints_the_user_and_the_picture_size() -> anyhow::Result<()> {
        let local_dir = std::env::temp_dir().join("cgp-tutorial-storage-read");
        std::fs::create_dir_all(&local_dir)?;
        tokio::fs::write(local_dir.join("alice.png"), b"not really a PNG").await?;

        let app = App {
            database: HashMap::from([(
                UserId(1),
                User {
                    name: "Alice".to_owned(),
                    profile_picture_object_id: Some("alice.png".to_owned()),
                },
            )]),
            local_dir: local_dir.to_str().unwrap().to_owned(),
        };

        let user = app.get_user(&UserId(1)).await?;
        let picture = app.fetch_storage_object("alice.png").await?;
        assert_eq!(format!("{} has a {}-byte picture", user.name, picture.len()), "Alice has a 16-byte picture");
        Ok(())
    }
}

/// *Combine the steps*.
mod combine_the_steps {
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
        pub local_dir: String,
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

    #[cgp_fn]
    #[async_trait]
    pub async fn fetch_storage_object(
        &self,
        #[implicit] local_dir: &str,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let data = tokio::fs::read(Path::new(local_dir).join(object_id)).await?;
        Ok(data)
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
            local_dir: "pictures".to_owned(),
        };

        match app.get_user_profile_picture(&UserId(1)).await? {
            Some(picture) => println!("Found a {}-byte picture", picture.len()),
            None => println!("No picture"),
        }

        Ok(())
    }

    #[cgp_fn]
    #[async_trait]
    #[uses(GetUser, FetchStorageObject)]
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

    /// The page's claim that a context with a `database` but no `local_dir` has `get_user`; the call
    /// to `get_user_profile_picture` it says fails to compile is not made here.
    #[derive(HasField)]
    pub struct UsersOnlyApp {
        pub database: HashMap<UserId, User>,
    }

    #[tokio::test]
    async fn main_prints_the_picture_size() -> anyhow::Result<()> {
        let local_dir = std::env::temp_dir().join("cgp-tutorial-storage-combine");
        std::fs::create_dir_all(&local_dir)?;
        tokio::fs::write(local_dir.join("alice.png"), b"not really a PNG").await?;

        let app = App {
            database: HashMap::from([(
                UserId(1),
                User {
                    name: "Alice".to_owned(),
                    profile_picture_object_id: Some("alice.png".to_owned()),
                },
            )]),
            local_dir: local_dir.to_str().unwrap().to_owned(),
        };

        let picture = app.get_user_profile_picture(&UserId(1)).await?;
        assert_eq!(picture.map(|data| data.len()), Some(16));

        let users_only = UsersOnlyApp { database: HashMap::new() };
        assert!(users_only.get_user(&UserId(1)).await.is_err());
        Ok(())
    }
}

/// *Add a backend for tests* and *Check the wiring*: the finished file.
mod a_backend_for_tests {
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
        pub local_dir: String,
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
            local_dir: "pictures".to_owned(),
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

    #[tokio::test]
    async fn main_reads_from_the_folder() -> anyhow::Result<()> {
        let local_dir = std::env::temp_dir().join("cgp-tutorial-storage-backend");
        std::fs::create_dir_all(&local_dir)?;
        tokio::fs::write(local_dir.join("alice.png"), b"not really a PNG").await?;

        let app = App {
            database: HashMap::from([(
                UserId(1),
                User {
                    name: "Alice".to_owned(),
                    profile_picture_object_id: Some("alice.png".to_owned()),
                },
            )]),
            local_dir: local_dir.to_str().unwrap().to_owned(),
        };

        let picture = app.get_user_profile_picture(&UserId(1)).await?;
        assert_eq!(picture.map(|data| data.len()), Some(16));
        Ok(())
    }
}
