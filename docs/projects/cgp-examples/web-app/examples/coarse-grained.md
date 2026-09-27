---
sidebar_label: 'coarse_grained'
sidebar_position: 1
description: 'A CGP application wired with one manager trait per domain, and the cost it shows: a dependency one method needs is carried by every method of the manager.'
---

# Wire an application with one trait per domain

This stage wires a small social-media backend with one trait per domain, a user manager and a post
manager, and shows what that costs as soon as one method needs something the others do not. It is
the first stage of [`web-app`](../index.md), a wiring study from the [cgp-examples](../../index.md)
repository, built with [CGP](/docs/). Nothing in the crate runs; every provider body is `todo!()`,
and the compiler's check of the wiring is what this page runs.

:::tip

### New to CGP?

The [Hello World tutorial](/docs/tutorials/hello) introduces components, providers, and wiring with
a smaller program. [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits)
explains what a component is, which is what this stage sizes. This page can be read without either.

:::

## The problem

The task is the backend of a small social-media service: create, read, and update users and posts,
store them in Postgres, and reject a username or a post that a content filter flags. This stage
writes it the way most applications start, with one trait per domain, and the problem it shows is
the one that starting point hides: a requirement of one method becomes a requirement of all of them.

### Without CGP

In plain Rust, the same design is a `UserManager` trait implemented by a Postgres type that holds
its database and its censor, or takes the censor as a generic parameter. It has the same cost. Only
`create_user` uses the censor, yet every value of the Postgres manager has to be built with one, so
code that only reads users, such as a report generator, still has to supply a censor it never calls.
Splitting the trait fixes it in plain Rust too, which is what the [next stage](./fine-grained.md)
does with CGP. This stage exists to make the cost visible first.

## Check it

From the root of the [cgp-examples repository](https://github.com/contextgeneric/cgp-examples):

```sh
cargo check -p cgp-example-web-app
```

The check passes. For this stage, that means `ProductionApp` has a working provider for each of its
four components, including everything those providers need.

## One component per domain

The module,
[`coarse_grained.rs`](https://github.com/contextgeneric/cgp-examples/blob/main/web-app/src/coarse_grained.rs),
defines a manager trait for each domain, and a small trait for each content filter:

```rust
#[cgp_component(UserManager)]
pub trait CanManageUser {
    fn create_user(&self, username: &str, email: &Email) -> Result<User, Error>;

    fn get_user(&self, user_id: &UserId) -> Result<User, Error>;

    fn update_user_data(&self, user_id: &UserId, user_data: &UserData) -> Result<(), Error>;
}

#[cgp_component(UsernameCensor)]
pub trait CanCensorUsername {
    fn username_is_censored(&self, username: &str) -> Probability;
}
```

[`#[cgp_component]`](/docs/reference/macros/cgp_component) turns each trait into a component: the
trait callers use, a trait providers implement, and a key that wiring names. `CanManagePost` has
`create_post`, `get_post`, `update_post`, and `delete_post`, and `CanDetectSpamMessage` has
`message_is_spam`. This is the shape most applications start from, since it follows the way the
domain is described.

## The manager's provider needs the censor

The user manager's provider stores users in Postgres, and checks new usernames with the censor
before creating them:

```rust
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

        todo!()
    }

    fn get_user(&self, #[implicit] database: &PostgresDb, user_id: &UserId) -> Result<User, Error> {
        todo!()
    }

    fn update_user_data(
        &self,
        #[implicit] database: &PostgresDb,
        user_id: &UserId,
        user_data: &UserData,
    ) -> Result<(), Error> {
        todo!()
    }
}
```

[`#[uses]`](/docs/reference/attributes/uses) declares what the provider needs from the context, here
the censor. [`#[implicit]`](/docs/reference/attributes/implicit) marks the `database` argument as
filled from the context's field of that name, so callers never pass it. Only `create_user` calls the
censor, but `#[uses]` is on the whole impl, so the whole provider depends on it.
`PostgresPostManager` has the same shape, with the spam detector.

## The context wires and checks in one table

`ProductionApp` is the context, a type that stands for the application and holds its choices. It
holds the database handle, and wires its four components:

```rust
#[derive(HasField)]
pub struct ProductionApp {
    pub database: PostgresDb,
}

delegate_and_check_components! {
    ProductionApp {
        UserManagerComponent: PostgresUserManager,
        PostManagerComponent: PostgresPostManager,
        UsernameCensorComponent: DummyUserCensor,
        SpamMessageDetectorComponent: DummySpamMessageDetector,
    }
}
```

[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components) wires each
entry and checks it in the same step, which suits a table of plain entries like this one.
[`#[derive(HasField)]`](/docs/reference/derives/derive_has_field) exposes the `database` field to
the providers that read it. The two filters are wired to placeholder providers here; the next stage
adds real ones.

## Try a change

Remove the censor's entry, the line `UsernameCensorComponent: DummyUserCensor,`, and run
[`cargo cgp check`](/docs/cargo-cgp/check). The whole user manager fails:

```text
error[E0277]: [CGP-E001] the consumer trait `CanManageUser` is not implemented for context `ProductionApp`
    = note: root cause: [CGP-E107] context `ProductionApp` does not contain any delegate entry for `UsernameCensorComponent`
```

The context has lost `get_user` and `update_user_data` as well as `create_user`, though neither
calls the censor, because the three methods share one provider and one set of requirements. Compare
the same change in the [next stage](./fine-grained.md#try-a-change). `cargo cgp check` leads with
the root cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet
reshape every class.

## The pattern

This stage shows **coarse-grained components**: one component per domain, each implemented by one
provider. The shape is easy to read and short to wire, four entries for the whole application. Its
cost is that a component carries the union of its methods' requirements. A context that wants to
read users, such as a report generator with no censor, cannot use `PostgresUserManager` at all, and
swapping how one method works means writing a whole new manager.

Coarse components are the right size when a domain's methods are always provided together. When
they start to need different things, the next stage's split is the fix. [Consumer and provider
traits](/docs/concepts/consumer-and-provider-traits) explains what a component is, and
[Impl-side dependencies](/docs/concepts/impl-side-dependencies) explains how `#[uses]` attaches a
requirement to a provider.

## Where to go next

- [`fine_grained`](./fine-grained.md): the next stage, one component per operation.
- [web-app overview](../index.md): the four stages side by side.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): how a provider's requirements are
  declared and resolved.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
