---
title: 'Overview of the features, with examples'
sidebar_label: 'Overview'
sidebar_position: 3
description: 'A tour of what Context-Generic Programming adds to Rust: each feature with a compiled example, the condition it holds under, and the page that explains it.'
---

# Overview

[Context-Generic Programming](/docs/) (CGP) is a language extension for Rust, with pluggable trait
implementations at compile-time. This page is a tour of what that gives you. It takes the features
one at a time, and for each one it starts from a problem you may already have, shows a short
example, says where the feature stops helping, and links the page that explains it in full. The
examples are fragments of programs that compile against `cgp` 0.8.0, with the imports and the
definitions they do not need to show left out. If you would rather run a program first, start with
the [Quickstart](/docs/quickstart).

The first five features follow one scenario, the one the
[`cgp` README](https://github.com/contextgeneric/cgp/blob/main/README.md) also uses: an application
that fetches users' profile pictures from object storage. The examples leave out the actual requests
to each storage service, because the features are about how the code is arranged rather than what
it does.

## The headline features

### One Interface, Many Implementations

Suppose the application stores its pictures in Amazon S3, a second deployment of it uses Google
Cloud Storage, and its tests read from a local folder. In plain Rust, each application type can
implement a fetching trait itself, but then the S3 code belongs to that one type. Reusing it means
writing a blanket implementation for every type with an S3 client, and another for every type with a
Google Cloud client, and Rust rejects that pair because one type could have both clients.

CGP avoids the conflict by giving each implementation a type of its own, called a **provider**.
Because the providers are different types, their implementations never overlap. Here is the
interface, defined as a CGP **component**, and the provider that fetches from S3:

```rust
#[async_trait]
#[cgp_component(StorageObjectFetcher)]
pub trait CanFetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}

#[cgp_impl(new FetchS3Object)]
impl StorageObjectFetcher {
    async fn fetch_storage_object(
        &self,
        #[implicit] storage_client: &aws_sdk_s3::Client,
        #[implicit] bucket_id: &str,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        // Fetch the object from Amazon S3.
    }
}
```

Two more providers follow the same pattern: `FetchGCloudObject` reads a Google Cloud `Storage`
client from the same `storage_client` field, and `FetchLocalObject` reads files from a `local_dir`
field for tests. Each application type, which CGP calls a **context**, then chooses one provider in
its **wiring**:

```rust
delegate_components! {
    App {
        StorageObjectFetcherComponent: FetchS3Object,
    }
}

delegate_components! {
    GCloudApp {
        StorageObjectFetcherComponent: FetchGCloudObject,
    }
}

delegate_components! {
    TestApp {
        StorageObjectFetcherComponent: FetchLocalObject,
    }
}
```

Code that calls `fetch_storage_object` stays the same in all three contexts. That does not mean
Rust's coherence rules have been disabled. They still apply to everything the macros generate, and
CGP only changes how the implementations are arranged, so that they no longer conflict.
[Bypassing coherence](/docs/concepts/coherence) explains the rules, and
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) shows how a call reaches
the chosen provider.

### Zero-Cost Abstraction

A natural question about choosing implementations is what the choice costs at runtime. The answer is
nothing, because the compiler resolves the wiring into a direct call. `App`'s wiring above amounts to
the following implementation, which you could also write by hand:

```rust
impl CanFetchStorageObject for App {
    async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
        <FetchS3Object as StorageObjectFetcher<App>>::fetch_storage_object(self, object_id)
            .await
    }
}
```

Nothing sits between the call and the provider: no runtime registry, no reflection, and no virtual
method table, and the provider itself is a type that is never instantiated. The provider's own work,
such as the request to S3, still runs, and generic code can add compile time and code size. If you
want to see what the macros actually generate, [`cargo cgp expand`](/docs/cargo-cgp/expand) prints
it, and [Dynamic dispatch](/docs/comparisons/dynamic-dispatch) compares this approach with
`dyn Trait`.

### Type-Safe Wiring

Choosing providers is only useful if a wrong choice is caught early, and with CGP it is caught at
compile time: a context that cannot give its provider what it needs fails to build once its wiring
is checked. For example, `FetchS3Object` reads a `bucket_id` field, so wiring it on a context that
lacks one, and checking that wiring with `check_components!`, is a compile error:

```rust
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
```

[cargo-cgp](/docs/cargo-cgp), CGP's error toolchain, reports the missing field first, followed by
the chain of requirements that led to it. Here is its report, without the source excerpt that points
at the check:

```text
error[E0277]: [CGP-E001] missing field `bucket_id` on `App`
   = note: root cause: [CGP-E110] missing field `bucket_id` on `App`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanFetchStorageObject` for context `App`
             └─ [CGP-E102] provider trait impl `StorageObjectFetcher` with context `App` for provider `FetchS3Object`
               └─ [CGP-E110] missing field `bucket_id` on `App`
   = help: add the field `bucket_id` to `App`
```

The check matters, because wiring on its own checks nothing. An entry compiles until the component
is used or checked, so check each context where you assemble it. Even then, the check establishes
that the components fit together, not that the application is correct. The diagnostics have a cost
as well: the raw compiler error for this mistake spells the field's name as a nested type,
`Symbol<9, Chars<'b', …>>`. `cargo cgp check` leads with the root cause for the classes it
recognizes, and the tool does not yet reshape every class.
[Check traits](/docs/concepts/check-traits) explains the checks, the
[checking tutorial](/docs/tutorials/area-calculation/checking) walks through a mis-wiring, and
[Dependency injection](/docs/comparisons/dependency-injection) compares wiring with a container.

### Abstract Over Every Dependency

So far the component has returned `anyhow::Result`, which ties every implementation to one error
type. Core logic does not have to commit like that. It can name the dependencies it needs, such as
an error type, and leave each context to supply them. Here the same component uses the context's
error type instead of `anyhow::Error`, and a provider declares the conversion it needs with
`#[uses]`, without adding it to the interface:

```rust
#[async_trait]
#[cgp_component(StorageObjectFetcher)]
#[use_type(HasErrorType.Error)]
pub trait CanFetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, Error>;
}

#[cgp_impl(new FetchLocalObject)]
#[uses(CanRaiseError<std::io::Error>)]
#[use_type(HasErrorType.Error)]
impl StorageObjectFetcher {
    async fn fetch_storage_object(
        &self,
        #[implicit] local_dir: &str,
        object_id: &str,
    ) -> Result<Vec<u8>, Error> {
        std::fs::read(format!("{local_dir}/{object_id}")).map_err(Self::raise_error)
    }
}
```

The S3 and Google Cloud providers change in the same way: each returns `Error` and declares, with
`#[uses]`, the conversion it needs from its client's error. A context then chooses its error type in
a single wiring entry, such as
`ErrorTypeProviderComponent: UseType<anyhow::Error>`, next to the providers that convert each source
error into it. Another context can choose an error enum of its own without touching this code, as
long as it supplies those conversions as well. The same arrangement works for other dependencies,
such as an async runtime, and it lets a core library stay `no_std` as long as the providers chosen
for it are `no_std` too. A provider that reads files, like this one, still needs `std`, and its
blocking read suits a test context, where an application would use an async file API. Async code may
also need attention to [`Send` bounds](/docs/concepts/send-bounds).
[Modular error handling](/docs/concepts/modular-error-handling) and
[Impl-side dependencies](/docs/concepts/impl-side-dependencies) explain the two halves of this
arrangement.

### Still Ordinary Rust

None of this asks you to leave ordinary Rust behind. CGP is a library on stable Rust, and its
simplest form is a function whose `#[implicit]` arguments come from the context's fields:

```rust
#[cgp_fn]
#[async_trait]
pub async fn get_user(
    &self,
    #[implicit] database: &PgPool,
    user_id: &UserId,
) -> anyhow::Result<User> {
    // Query `database` for the user row.
}
```

`#[cgp_fn]` turns the function into a trait with a
[blanket implementation](/docs/reference/glossary#blanket-implementation), so any context with a
`PgPool` field named `database` gains `get_user`, with no wiring at all. The rest of CGP stays close
to Rust in the same way. Providers use familiar `impl` syntax, and the trait that callers use remains
an ordinary trait, so the hand-written implementation under *Zero-Cost Abstraction* is a valid way
to implement it directly. This is why you can adopt CGP one component at a time. The cost is the
provider model you have to learn and the wiring you have to maintain.
[Implicit arguments](/docs/concepts/implicit-arguments) explains the field reads, and
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) helps you judge how much of CGP a
problem needs.

## Beyond choosing implementations

Choosing implementations is the core of CGP, but the same machinery reaches further. A context can
choose types as well as implementations, generic code can work over the structure of data, and small
steps can be composed into larger operations.

### Abstract types

The error type from earlier is one case of a broader problem. In plain Rust, each trait that can fail
usually declares its own associated error type. A function that uses two such traits must then
either convert between their errors or state that they are the same type, and that bound travels
with it to every generic caller:

```rust
pub trait Load {
    type Error;
    fn load(&self, path: &str) -> Result<String, Self::Error>;
}

pub trait Save {
    type Error;
    fn save(&self, path: &str, contents: &str) -> Result<(), Self::Error>;
}

pub fn back_up_profile<Context>(
    context: &Context,
    user: &str,
) -> Result<(), <Context as Load>::Error>
where
    Context: Load + Save<Error = <Context as Load>::Error>,
{
    let profile = context.load(&format!("profiles/{user}"))?;
    context.save(&format!("backups/{user}"), &profile)
}
```

With CGP, both traits import the context's one error type, so the function has nothing to equate
and names no error type at all:

```rust
#[cgp_fn]
#[uses(CanLoad, CanSave)]
#[use_type(HasErrorType.Error)]
pub fn back_up_profile(&self, user: &str) -> Result<(), Error> {
    let profile = self.load(&format!("profiles/{user}"))?;
    self.save(&format!("backups/{user}"), &profile)
}
```

The context chooses that error type once, in its wiring, and every trait that imports
`HasErrorType` agrees on it, so a loader and a saver on the same context always return the same
error. These are still Rust associated types, which means their guarantees come only from the bounds
you declare, and a type that appears in an interface, such as a returned error, still belongs in that
interface. For a function with one or two independent type parameters,
ordinary generics may be clearer. [Abstract types](/docs/concepts/abstract-types) explains the
mechanism.

### Extensible records and variants

Some generic code needs to work over the shape of data, building a struct field by field or handling
an enum variant by variant. CGP supports this through a derive that exposes a type's fields to the
trait system, so separate parts of a program can each build their own piece of a record:

```rust
#[derive(CgpData)]
pub struct AppConfig {
    pub database_url: String,
    pub http_timeout: u32,
}

let config = AppConfig::builder()
    .build_from(DatabaseConfig {
        database_url: "postgres://localhost".to_owned(),
    })
    .build_from(HttpConfig { http_timeout: 30 })
    .finalize_build();
```

Here `DatabaseConfig` and `HttpConfig` are structs with one of the fields each, and they derive
`CgpData` too. The compiler checks that the pieces cover every field, so nothing is discovered at
runtime.

Variants work the same way from the other side. With `#[cgp_auto_dispatch]` on a trait, an enum
implements that trait from its variants:

```rust
#[cgp_auto_dispatch]
pub trait CanDescribe {
    fn describe(&self) -> String;
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

`Circle` and `Rectangle` each implement `CanDescribe` as ordinary types, and `Shape` gains
`describe` without a `match`, calling the implementation of whichever variant it holds. In both
directions a type takes part only if it has the derive, and an enum dispatches only when every
variant's payload implements the trait.
[Extensible records](/docs/concepts/extensible-records),
[Extensible variants](/docs/concepts/extensible-variants), and
[Dispatching](/docs/concepts/dispatching) cover the patterns and their limits.

### Composable handlers

CGP also has a family of handler components for computations, synchronous or async, fallible or
not. Providers for them compose into pipelines whose input and output types must agree, and a
context chooses the steps in its wiring:

```rust
#[cgp_computer]
fn double(x: u64) -> u64 {
    x * 2
}

#[cgp_computer]
fn increment(x: u64) -> u64 {
    x + 1
}

delegate_components! {
    App {
        ComputerComponent: PipeHandlers<Product![Double, Increment]>,
    }
}
```

`#[cgp_computer]` turns each function into a provider named after it, `Double` and `Increment`, so
computing on `App` with the input 5 doubles it and then adds one, returning 11. When a sequence never
needs interchangeable steps, ordinary function calls are simpler. [Handlers](/docs/concepts/handlers)
covers the family and its combinators.

### Namespaces

As an application grows, so does its wiring, and several contexts often repeat most of the same
entries. A namespace gives them a wiring table they share, so each context lists only what differs.
Each component registers under a path, such as `@app.GreeterComponent` in `AppNamespace`, with a
`#[prefix]` attribute. `AppDefaults` then extends that namespace with the bindings every context
shares, and a context joins it and fills in the rest:

```rust
cgp_namespace! {
    new AppDefaults: AppNamespace {
        @app.FarewellComponent: SayGoodbye,
    }
}

delegate_components! {
    App {
        namespace AppDefaults;
        @app.GreeterComponent: GreetHello,
    }
}
```

A `TestApp` that joins the same namespace with a different greeter differs from `App` by one line.
The sharing has a limit, though: a context that joins a namespace cannot override a binding the
namespace makes, so a namespace should bind only what every context shares. For a component with a
type parameter there is also a lighter option, an `open` statement in the wiring, which dispatches
per type without defining a namespace at all. [Namespaces](/docs/concepts/namespaces) explains both.

### Higher-order providers

Providers can also build on one another. A provider can take another provider as a parameter, so a
single wrapper can serve many implementations. `ScaledArea` scales whatever area its inner provider
computes, and a context chooses both in a single entry:

```rust
#[cgp_impl(new ScaledArea<Inner>)]
#[use_provider(Inner: AreaCalculator)]
impl<Inner> AreaCalculator {
    fn area(&self, #[implicit] scale: f64) -> f64 {
        Inner::area(self) * scale * scale
    }
}

delegate_components! {
    ScaledRectangle {
        AreaCalculatorComponent: ScaledArea<RectangleArea>,
    }
}
```

The price of this flexibility is indirection, because each layer adds a step to follow when you
trace which code runs. [Higher-order providers](/docs/concepts/higher-order-providers) shows how the
bounds compose.

## The problems it solves

The features above are easier to judge against the problems you actually have. CGP helps most where
a program already needs several implementations, or where dependencies spread through its
signatures. The table below starts from the problem and links the section of this page that shows
it, where there is one, along with the Concepts page that explains it:

| Problem | On this page | Explained in |
| --- | --- | --- |
| Rust rejects two implementations of a trait for the same type | [One Interface, Many Implementations](#one-interface-many-implementations) | [Bypassing coherence](/docs/concepts/coherence) |
| A trait for a type you do not own needs a newtype | Not shown here | [Bypassing coherence](/docs/concepts/coherence) |
| Tests need a different implementation from production | [One Interface, Many Implementations](#one-interface-many-implementations) | [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) |
| Feature flags already select between backends | [One Interface, Many Implementations](#one-interface-many-implementations) | [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) |
| Changing the error type or runtime touches every layer | [Abstract Over Every Dependency](#abstract-over-every-dependency) | [Modular error handling](/docs/concepts/modular-error-handling) |
| A trait has grown until no implementation needs all of it | Not shown here | [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) |
| An implementation's dependencies leak into the public interface | [Abstract Over Every Dependency](#abstract-over-every-dependency) | [Impl-side dependencies](/docs/concepts/impl-side-dependencies) |
| Generic parameters repeat through every layer | [Abstract types](#abstract-types) | [Abstract types](/docs/concepts/abstract-types) |
| Generic code needs to work over any struct or enum | [Extensible records and variants](#extensible-records-and-variants) | [Extensible records](/docs/concepts/extensible-records) |
| A wiring error buries its cause in generated types | [Type-Safe Wiring](#type-safe-wiring) | [Check traits](/docs/concepts/check-traits) |

## When a plain trait is enough

All of this comes at a cost, and for a trait with only one implementation, a plain trait is the
right tool. CGP adds declarations and wiring that a plain trait does not need, compile-time work for
its macros and type checking, and long raw errors. `cargo cgp check` leads with the root cause for
the classes it recognizes, and the tool does not yet reshape every class. Learning the patterns also
takes time. If you work with an assistant, [CGP's agent skill](/docs/ai/skills/) helps it with the
vocabulary, the diagnostics, and the wiring, although you still need to review what it writes. The
[front page's costs](/#costs) state the project's maturity, and
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) explains when a plain trait is enough
and when providers start to help.

## Where to go next

Where you go from here depends on which part of this tour caught your interest:

- **Run code:** the [Quickstart](/docs/quickstart), then the
  [Hello World tutorial](/docs/tutorials/hello).
- **Understand the mechanism:** the [Concepts](/docs/concepts), starting with
  [Bypassing coherence](/docs/concepts/coherence).
- **Compare with what you know:** the [Comparisons](/docs/comparisons).
- **Look up a construct:** the [Reference](/docs/reference).

---

*An AI agent wrote this page using the CGP knowledge base, and its examples were verified by
compiling them. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
