---
title: 'Swap a Storage Backend per Application'
sidebar_label: 'Swap a Storage Backend'
sidebar_position: 3
description: 'A Rust tutorial: a profile-picture lookup that reads from a folder in the app and from memory in tests, each choice wired and checked at compile time.'
---

# Swap a Storage Backend per Application

Most applications reach a point where one operation needs a different backend in different
places. A deployment reads its files from disk, for example, while its tests would rather not touch
the disk at all. [Context-Generic Programming](/docs/) (CGP) is a Rust library for that kind of
choice: it enables pluggable trait implementations in Rust, and each application type chooses the
implementation it uses at compile time.

In this tutorial, we will build a working profile-picture lookup and link out to the details as we
go. By the end, one lookup function will read pictures from a folder in the application and from
memory in a test, and the compiler will check that each choice is complete. The tutorial assumes
you know Rust, including async functions, and can add a dependency. It assumes no CGP, though the
[Hello World tutorial](/docs/tutorials/hello) is a shorter first look at the same starting point.

## Set up the project

Start a new binary project with `cargo new profile-pictures`, then add three dependencies to its
`Cargo.toml`: `cgp` itself, `anyhow` for errors, and `tokio`, because the lookup is async and reads
files through Tokio:

```toml title="Cargo.toml"
[dependencies]
cgp = "0.8.0"
anyhow = "1"
tokio = { version = "1", features = ["fs", "macros", "rt-multi-thread"] }
```

Every module that uses CGP imports its prelude. The program fits in one file, so these imports go
at the top of `src/main.rs`:

```rust title="src/main.rs"
use std::collections::HashMap;
use std::path::Path;

use anyhow::anyhow;
use cgp::prelude::*;
```

## The application context

The lookup needs two pieces of data: a table of users, and somewhere to keep their pictures. Each
user may have a picture, stored under an object ID:

```rust
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub profile_picture_object_id: Option<String>,
}
```

The application itself is a struct called `App`. In CGP, a type like this is called a **context**:
a type that stands for this application, which is where its choices live. This one has fields, the
values its operations need. Its `database` is a `HashMap` standing in for a real database, which
keeps the tutorial runnable without a server, and its `local_dir` names the folder that holds the
pictures:

```rust
#[derive(HasField)]
pub struct App {
    pub database: HashMap<UserId, User>,
    pub local_dir: String,
}
```

The `#[derive(HasField)]` line lets CGP code read those fields by name, which is what the next
step relies on.

## Read the application's data with functions

The lookup has two steps: find the user, then load the picture. We write each one as a function
marked `#[cgp_fn]`, and mark the arguments that should come from the application's fields with
`#[implicit]`:

```rust
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
```

Notice that callers never pass `database` or `local_dir`. `#[cgp_fn]` turns each function into a
method on any type whose fields match the implicit arguments by name and type, so `App` gains
`get_user` and `fetch_storage_object` without an `impl` block of its own. The `local_dir` argument
reads the `String` field as a `&str`. Both functions are async, so each also carries
`#[async_trait]`; without it, Rust warns about the `async fn` in the public trait that `#[cgp_fn]`
generates.

To try them, `main` writes a picture into a `pictures` folder, builds an `App` with one user, and
calls both methods:

```rust
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
```

Running `cargo run` prints `Alice has a 16-byte picture`. The bytes are not a real image, but the
program never decodes them, so any bytes will do.

## Combine the steps

The two steps belong together, so the next function calls both. Because `get_user` and
`fetch_storage_object` are now methods, the new function calls them on `self`, and declares that it
needs them with `#[uses]`:

```rust
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
```

The names in `#[uses(GetUser, FetchStorageObject)]` are the traits `#[cgp_fn]` generated from the
two functions: a function called `get_user` becomes a trait called `GetUser`. Declaring them there,
rather than as bounds in the signature, keeps the function's interface to a user ID alone. It also
means that `get_user_profile_picture` exists only on a type that has every field both steps read. A
context with a `database` but no `local_dir` would still have `get_user`, but a call to
`get_user_profile_picture` on it would fail to compile.

Replace the last lines of `main`, from `let user` onwards, with a call to the combined function:

```rust
    match app.get_user_profile_picture(&UserId(1)).await? {
        Some(picture) => println!("Found a {}-byte picture", picture.len()),
        None => println!("No picture"),
    }

    Ok(())
}
```

`cargo run` now prints `Found a 16-byte picture`.

## Add a backend for tests

So far there is one way to fetch a picture, and it reads the disk. A test of
`get_user_profile_picture` would rather keep its pictures in memory, so it needs a second way to
fetch them, chosen by the test's own context. A `#[cgp_fn]` function has exactly one
implementation, so the storage step needs something that can have several.

That something is a CGP **component**. We replace the `fetch_storage_object` function with a
trait marked `#[cgp_component]`:

```rust
#[async_trait]
#[cgp_component(StorageObjectFetcher)]
pub trait CanFetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}
```

Code calls `CanFetchStorageObject` as an ordinary trait, while `StorageObjectFetcher` names the
trait each implementation targets. An implementation is called a **provider**, and we write one
with `#[cgp_impl]`, giving it a name. The disk-reading provider is the old function's body, under
the name `FetchLocalObject`, and it still reads `local_dir` as an implicit argument:

```rust
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
```

The combined function now depends on the component rather than on the removed function, so its
`#[uses]` names the trait that callers use:

```rust
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
```

A provider does nothing until a context chooses it. `App` makes its choice with
`delegate_components!`, which works like a lookup table from each component to the provider that
implements it for this context:

```rust
delegate_components! {
    App {
        StorageObjectFetcherComponent: FetchLocalObject,
    }
}
```

The key, `StorageObjectFetcherComponent`, is a name `#[cgp_component]` generates for the
component. Rust resolves the table at compile time, so `app.fetch_storage_object(...)` becomes a
direct call to `FetchLocalObject`, without a runtime lookup. `cargo run` prints the same line as
before.

Now for the test. Its context, `TestApp`, has the same `database` field and keeps its pictures in an
`objects` map instead of a folder. A second provider, `FetchMemoryObject`, reads that map, and
`TestApp` chooses it in its own table. All of this goes in a test module at the end of
`src/main.rs`:

```rust
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
```

Run `cargo test`, and the test passes without touching the disk. Neither
`get_user_profile_picture` nor `get_user` changed to make that possible. Each context chose its
storage backend in one line of its table, and the same lookup ran against a folder in the
application and against memory in the test. Adding a third backend, such as a cloud bucket, would
be one more provider and one more table entry. The [Overview](/docs/overview) shows providers for
Amazon S3 and Google Cloud Storage written against this same component.

## Check the wiring

A wiring table is lazy: adding an entry does not check that the context has what the provider
needs. The check happens when the component is used, and if something is missing, the error points
at the place it was used, which may be far from the table that caused it. CGP's
`check_components!` moves that check next to the table, so add one for each context, `App`'s after
its table and `TestApp`'s inside the test module:

```rust
check_components! {
    App {
        StorageObjectFetcherComponent,
    }
}
```

```rust
    check_components! {
        TestApp {
            StorageObjectFetcherComponent,
        }
    }
```

To see what the check catches, delete the `local_dir` field from `App`, along with the line in
`main` that sets it. `FetchLocalObject` still needs the field, so `cargo check` now fails at the
check. The compiler's message is long, and it spells the missing field's name as a nested type.
Shortened, it begins like this:

```text
error[E0277]: the trait bound `App: CanUseComponent<StorageObjectFetcherComponent>` is not satisfied
...
help: the trait `HasField<Symbol<9, Chars<'l', Chars<'o', Chars<'c', Chars<'a', Chars<'l', Chars<'_', Chars<'d', Chars<'i', cgp::macro_prelude::Chars<'r', Nil>>>>>>>>>>>` is not implemented for `App`
```

[cargo-cgp](/docs/tooling/cargo-cgp), CGP's error toolchain, explains the same failure more
directly. After [installing it](/docs/tooling/cargo-cgp/installation), run `cargo cgp check` in
place of `cargo check`. It names the missing field first, then the chain of requirements that led to
it. Here is its report, without the source excerpt that points at the check:

```text
error[E0277]: [CGP-E001] missing field `local_dir` on `App`
   = note: root cause: [CGP-E110] missing field `local_dir` on `App`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanFetchStorageObject` for context `App`
             └─ [CGP-E102] provider trait impl `StorageObjectFetcher` with context `App` for provider `FetchLocalObject`
               └─ [CGP-E110] missing field `local_dir` on `App`
   = help: add the field `local_dir` to `App`
```

`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not
yet reshape every class. Put the field back, and both checks pass.

## Where to go next

You now have a lookup whose storage backend each application chooses, checked at compile time. To
go further:

- **See how each construct is built:** the [area-calculation tutorials](/docs/tutorials/area-calculation)
  derive `#[cgp_fn]`, components, providers, and wiring step by step from plain Rust, and end with
  a longer look at checking.
- **Understand the mechanism:** [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits)
  explains how a call reaches the chosen provider, and
  [Implicit arguments](/docs/concepts/implicit-arguments) how fields become arguments.
- **Let each context choose its error type too:** the Overview's
  [Abstract Over Every Dependency](/docs/overview#abstract-over-every-dependency) replaces
  `anyhow::Result` in this same component.
- **Write generic code over structs and enums:** for framework and library authors,
  [Extensible records](/docs/concepts/extensible-records) and
  [Extensible variants](/docs/concepts/extensible-variants) show code that works over any type's
  fields or variants.

---

*An AI agent wrote this page using the CGP knowledge base, and its examples were verified by
compiling them. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
