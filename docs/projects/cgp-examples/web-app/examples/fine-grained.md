---
sidebar_label: 'fine_grained'
sidebar_position: 2
description: 'A CGP application split into one component per operation, with content checks as wrapper providers and the providers grouped into bundles.'
---

# Split a manager into one component per operation

This stage splits the backend's two managers into one component per operation, moves the content
checks into providers that wrap another provider, and groups the result into three bundles. It is
the second stage of [`web-app`](../index.md), a wiring study from the [cgp-examples](../../index.md)
repository, built with [CGP](/docs/). Nothing in the crate runs; every provider body is `todo!()`,
and the compiler's check of the wiring is what this page runs.

:::tip

### New to CGP?

[`coarse_grained`](./coarse-grained.md) is the stage this one splits. For CGP itself, the [Hello
World tutorial](/docs/tutorials/hello) introduces providers and wiring, [Higher-order
providers](/docs/concepts/higher-order-providers) explains the wrappers, and [Aggregate
providers](/docs/concepts/aggregate-providers) explains the bundles.

:::

## The problem

The task is the same backend as the [coarse stage](./coarse-grained.md), rewired so that each
operation carries only the requirements it uses. That asks for three things at once: an operation
per component, so reading users does not need a censor; the content checks separated from the
storage code, so a check can wrap any way of creating users; and a way to group the result, so an
application does not wire nine components one by one.

### Without CGP

Plain Rust can do each of these. A trait per operation splits the requirements, and a decorator, a
struct generic over an inner creator that runs the check and then calls it, separates the check from
the storage. The cost is in putting them together. Each decorator holds its inner value and its
dependencies as fields, so assembling the application means constructor code that builds every layer
by hand in the right order, and a second configuration of the application is a second constructor
that repeats the layers it shares with the first. This stage expresses the same layering as wiring,
where the compiler checks each piece.

## Check it

From the root of the [cgp-examples repository](https://github.com/contextgeneric/cgp-examples):

```sh
cargo check -p cgp-example-web-app
```

The check passes. For this stage, that means `ProductionApp` satisfies all nine of its components.

## One component per operation

The module,
[`fine_grained.rs`](https://github.com/contextgeneric/cgp-examples/blob/main/web-app/src/fine_grained.rs),
gives each operation a component of its own, with the one method the manager had:

```rust
#[cgp_component(UserCreator)]
pub trait CanCreateUser {
    fn create_user(&self, username: &str, email: &Email) -> Result<User, Error>;
}

#[cgp_component(UserGetter)]
pub trait CanGetUser {
    fn get_user(&self, user_id: &UserId) -> Result<User, Error>;
}
```

The users get `UserCreator`, `UserGetter`, and `UserUpdater`, and the posts get `PostCreator`,
`PostGetter`, `PostUpdater`, and `PostDeleter`, with the same two filters as before: nine components
in all. Each can now have its own provider, with its own requirements.

## The content check becomes a wrapper

The username check no longer lives inside the Postgres code. It is a provider of its own, which
rejects a censored name and otherwise passes the call to another creator, named as a type parameter:

```rust
#[cgp_impl(new FilterCensoredUsername<InnerCreator>)]
#[uses(CanCensorUsername)]
#[use_provider(InnerCreator: UserCreator)]
impl<InnerCreator> UserCreator {
    fn create_user(&self, username: &str, email: &Email) -> Result<User, Error> {
        if self.username_is_censored(username) > Probability::new(0.8) {
            return Err(Error::InvalidUsername);
        }

        InnerCreator::create_user(self, username, email)
    }
}
```

[`#[use_provider]`](/docs/reference/attributes/use_provider) states that `InnerCreator` must itself
be a `UserCreator` provider, and the wrapper calls it directly. So the Postgres creator,
`CreateUserWithPostgres`, checks nothing, and the check can wrap any creator, a Postgres one or any
other. `FilterSpamMessage<InnerCreator>` does the same for posts. The other Postgres providers,
`GetUserWithPostgres` and the rest, each implement one operation and read only the database.

## Bundles group the providers

Three bundles, each a named wiring table, group the providers by domain:

```rust
delegate_components! {
    new PostgresUserComponents {
        UserCreatorComponent:
            FilterCensoredUsername<CreateUserWithPostgres>,
        UserGetterComponent:
            GetUserWithPostgres,
        UserUpdaterComponent:
            UpdateUserWithPostgres,
    }
}
```

The `new` keyword declares `PostgresUserComponents` as a type, and the entries are its wiring. The
creator is wired as the wrapper around the Postgres creator, which is where the two meet.
`PostgresPostComponents` does the same for the four post operations, and
`AiContentFilterComponents` wires the two filters to AI-backed providers.

## The context forwards to the bundles

`ProductionApp` is the **context**, a type that stands for the application and holds its choices,
and it forwards each component to the bundle that holds it:

```rust
delegate_components! {
    ProductionApp {
        [
            UserCreatorComponent,
            UserGetterComponent,
            UserUpdaterComponent,
        ]:
            PostgresUserComponents,
        [
            PostCreatorComponent,
            PostGetterComponent,
            PostUpdaterComponent,
            PostDeleterComponent,
        ]:
            PostgresPostComponents,
        [
            UsernameCensorComponent,
            SpamMessageDetectorComponent,
        ]:
            AiContentFilterComponents,
    }
}
```

A bracketed list gives several keys one destination, and a bundle is a valid destination because it
is itself a wiring table: a lookup for `UserGetterComponent` goes to `PostgresUserComponents`, which
answers with `GetUserWithPostgres`. CGP calls a table used this way an [aggregate
provider](/docs/concepts/aggregate-providers). A separate
[`check_components!`](/docs/reference/macros/check_components) block lists all nine components on
`ProductionApp`, and a check of a component covers everything its provider needs, down through the
bundle.

## Try a change

Remove `UsernameCensorComponent` from the last list, so the censor has no entry, and run
[`cargo cgp check`](/docs/cargo-cgp/check). This time only the operation that uses the censor fails:

```text
error[E0277]: [CGP-E001] the consumer traits `CanCreateUser` and `CanCensorUsername` are not implemented for context `ProductionApp`
    = note: root cause: [CGP-E107] context `ProductionApp` does not contain any delegate entry for `UsernameCensorComponent`
```

Getting and updating users still check, because their providers need only the database. In the
[coarse stage](./coarse-grained.md#try-a-change) the same change took the whole user manager with
it. `cargo cgp check` leads with the root cause for the classes it recognizes, and the tool is a
v0.1.0-alpha that does not yet reshape every class.

## The pattern

This stage shows three patterns at once. **Fine-grained components** give each operation its own
provider and its own requirements, so a context can use one operation without paying for the
others. **Higher-order providers** move a check out of a provider into a wrapper that takes the
provider as a parameter, so the check composes with any implementation; see [Higher-order
providers](/docs/concepts/higher-order-providers). And **aggregate providers** group the wiring into
bundles that other contexts can reuse whole; see [Aggregate
providers](/docs/concepts/aggregate-providers).

The cost is that the context still names every component. The bundles shortened the right-hand
side of the table, but the nine keys are all still listed, and every new operation is a new key in
every context. The [next stage](./namespaces.md) removes that.

## Where to go next

- [`namespace`](./namespaces.md): the next stage, the same components grouped under paths.
- [web-app overview](../index.md): the four stages side by side.
- [Higher-order providers](/docs/concepts/higher-order-providers): the CGP idea behind the filter
  wrappers.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
