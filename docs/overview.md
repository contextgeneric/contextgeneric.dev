---
title: 'Overview of the features, with examples'
sidebar_label: 'Overview'
sidebar_position: 3
description: 'A tour of what Context-Generic Programming adds to Rust: each feature with a compiled example, the condition it holds under, and the page that explains it.'
---

# Overview

[Context-Generic Programming](/docs/) (CGP) is a language extension for Rust, with pluggable trait
implementations at compile-time. This page walks through what that gives you, one feature at a
time: each section names the problem, shows a short example, states where the feature stops, and
links the page that explains it. The examples are fragments of programs that compile against
`cgp` 0.8.0, leaving out the imports and the definitions they do not need to show. To run a program
first, start with the [Quickstart](/docs/quickstart).

The five headline features follow one application that fetches users' profile pictures from object
storage, the scenario the [`cgp` README](https://github.com/contextgeneric/cgp/blob/main/README.md)
also uses. The examples elide the requests to each storage service, because the features are about
how the code is arranged rather than what it does.

## The headline features

### One Interface, Many Implementations

Several implementations of one interface can coexist, and each application chooses the one it uses.
In plain Rust, each application type can implement the trait itself, but then the S3 code belongs to
that one type. To reuse it, you would write a blanket implementation for every type with an S3
client, and another for every type with a Google Cloud client, and Rust rejects the pair because one
type could have both. CGP gives each implementation its own **provider** type, so the
implementations never overlap, and each **context**, the type that stands for one application's
choices, selects a provider for the **component**:

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

`FetchGCloudObject` reads a Google Cloud `Storage` client from the same `storage_client` field, and
`FetchLocalObject` reads files from a `local_dir` field for tests. Three contexts then choose one
provider each, in their **wiring**:

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

Code that calls `fetch_storage_object` stays the same in all three. Rust's coherence rules still
apply to the generated implementations; CGP changes how the implementations are arranged rather than
disabling the check. [Bypassing coherence](/docs/concepts/coherence) explains the rules, and
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) shows how a call reaches
the chosen provider.

### Zero-Cost Abstraction

Choosing a provider costs nothing at runtime, because the compiler resolves the wiring into a direct
call. `App`'s wiring above amounts to this implementation, which you could write by hand:

```rust
impl CanFetchStorageObject for App {
    async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
        <FetchS3Object as StorageObjectFetcher<App>>::fetch_storage_object(self, object_id)
            .await
    }
}
```

There is no runtime registry, reflection, or virtual method table between the call and the
provider, and a provider is a type that is never instantiated. The provider's own work still costs
what it costs, and generic code can add compile time and code size.
[`cargo cgp expand`](/docs/cargo-cgp/expand) prints what the macros actually generate, and
[Dynamic dispatch](/docs/comparisons/dynamic-dispatch) compares this with `dyn Trait`.

### Type-Safe Wiring

A context that cannot satisfy its provider fails to build. `FetchS3Object` reads a `bucket_id`
field, so wiring it on a context without one, and checking that wiring with `check_components!`, is
a compile error:

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

[cargo-cgp](/docs/cargo-cgp), CGP's error toolchain, reports the missing field first and then the
chain that needed it:

```text
error[E0277]: [CGP-E001] missing field `bucket_id` on `App`
  --> src/lib.rs:48:9
   |
48 |         StorageObjectFetcherComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E110] missing field `bucket_id` on `App`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanFetchStorageObject` for context `App`
             └─ [CGP-E102] provider trait impl `StorageObjectFetcher` with context `App` for provider `FetchS3Object`
               └─ [CGP-E110] missing field `bucket_id` on `App`
   = help: add the field `bucket_id` to `App`
```

Wiring alone checks nothing: an entry compiles until the component is used or checked, so check
each context where you assemble it. The checks establish that the components fit together, not that
the application is correct. The raw compiler error for the same mistake spells the field's name as
a nested type, `Symbol<9, Chars<'b', …>>`. `cargo cgp check` leads with the root cause for the
classes it recognizes, and the tool does not yet reshape every class.
[Check traits](/docs/concepts/check-traits) explains the checks, the
[checking tutorial](/docs/tutorials/area-calculation/checking) walks through a mis-wiring, and
[Dependency injection](/docs/comparisons/dependency-injection) compares the wiring with a container.

### Abstract Over Every Dependency

Core logic can name the dependencies it needs, such as an error type, and leave each context to
supply them. Here the component uses the context's error type instead of `anyhow::Error`, and a
provider states the conversion it needs with `#[uses]`, without adding it to the interface:

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

A context then chooses the error type in one wiring entry, such as
`ErrorTypeProviderComponent: UseType<anyhow::Error>`, beside the provider that converts each source
error. Another context can choose its own error enum without touching this code, as long as it also
supplies the conversions. The same arrangement works for a runtime or a storage backend, and it lets
a core library stay `no_std` when the providers chosen for it are. A provider that needs an
operating system still needs one, and async code may need attention to
[`Send` bounds](/docs/concepts/send-bounds).
[Modular error handling](/docs/concepts/modular-error-handling) and
[Impl-side dependencies](/docs/concepts/impl-side-dependencies) explain both halves.

### Still Ordinary Rust

CGP is a library on stable Rust, and its code reads like ordinary Rust. The simplest form is a
function whose `#[implicit]` arguments come from the context's fields:

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
`PgPool` field named `database` gains `get_user`, without wiring. Providers use familiar `impl`
syntax, and the trait that callers use stays an ordinary trait: the hand-written implementation
under *Zero-Cost Abstraction* is a valid way to implement it directly. So you can adopt CGP one
component at a time. The cost is the provider model you learn and the wiring you maintain.
[Implicit arguments](/docs/concepts/implicit-arguments) explains the field reads, and
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) shows how much of CGP a problem needs.

## Beyond choosing implementations

The same machinery lets a context choose types as well as implementations, lets generic code work
over data, and composes small steps into larger operations.

### Abstract types

A context can choose a type, such as its error type, that every trait and function using it then
shares. When the error type is a generic parameter, every function that forwards a fallible call
carries it as a parameter of its own:

```rust
pub fn load_profile<Context, Error>(context: &Context, user: &str) -> Result<String, Error>
where
    Context: Load<Error>,
{
    context.load(&format!("profiles/{user}"))
}
```

With CGP, the function imports the context's error type, and neither it nor its callers name one:

```rust
#[cgp_fn]
#[uses(CanLoad)]
#[use_type(HasErrorType.Error)]
pub fn load_profile(&self, user: &str) -> Result<String, Error> {
    self.load(&format!("profiles/{user}"))
}
```

A plain associated type already removes the parameter from one trait. CGP adds one type, chosen in
the context's wiring, that every trait importing `HasErrorType` agrees on, so a loader and a storage
fetcher on the same context return the same error. These are Rust associated types, so their
guarantees come from the bounds you declare, and a type
that appears in an interface, such as a returned error, still belongs in that interface. For a
function with one or two independent type parameters, ordinary generics may be clearer.
[Abstract types](/docs/concepts/abstract-types) explains the mechanism.

### Extensible records and variants

Generic code can build a struct field by field and handle an enum variant by variant. A derive
exposes a type's fields to the trait system, so separate parts of a program can each build their own
piece of a record:

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

`DatabaseConfig` and `HttpConfig` are structs with one of the fields each, deriving `CgpData` too.
The compiler checks that the pieces cover every field; nothing is discovered at runtime, and a type
takes part only through its derive. [Extensible records](/docs/concepts/extensible-records),
[Extensible variants](/docs/concepts/extensible-variants), and
[Dispatching](/docs/concepts/dispatching) cover the patterns and their limits.

### Composable handlers

CGP's handler components describe computations, synchronous or async, fallible or not, and providers
for them compose into pipelines whose input and output types must agree. A context chooses the steps
in its wiring:

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

Computing on `App` with the input 5 returns 11. Ordinary function calls are simpler when a sequence
never needs interchangeable steps. [Handlers](/docs/concepts/handlers) covers the family and its
combinators.

### Namespaces

A namespace is a wiring table that several contexts share, so each context lists only what differs.
Components register under a path in the namespace, a namespace binds the paths every context agrees
on, and a context joins it and fills the rest:

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

A `TestApp` joining the same namespace with a different greeter differs from `App` by one line. A
binding the namespace makes cannot be overridden by a context that joins it, so a namespace binds
only what every context shares. For a component with a type parameter, an `open` statement in the
wiring dispatches per type without defining a namespace.
[Namespaces](/docs/concepts/namespaces) explains both.

### Higher-order providers

A provider can take another provider as a parameter and build on it, so one wrapper serves many
implementations. `ScaledArea` scales whatever area its inner provider computes, and a context
chooses both in one entry:

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

Each layer adds a step to follow when you trace which code runs.
[Higher-order providers](/docs/concepts/higher-order-providers) shows how the bounds compose.

## The problems it solves

CGP helps most where a program already needs several implementations, or where dependencies spread
through its signatures. Each problem below links the section of this page and the Concepts page that
answer it:

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

For a trait with one implementation, a plain trait is the right tool. CGP adds declarations and
wiring that a plain trait does not need, compile-time work for its macros and type checking, and
long raw errors, though `cargo cgp check` leads with the root cause for the classes it recognizes,
and the tool does not yet reshape every class. Learning the patterns takes time; if you work with an
assistant, [CGP's agent skill](/docs/ai/skills/) helps it with the vocabulary, the diagnostics, and
the wiring, and you still review what it writes. The
[front page's costs](/#costs) state the project's maturity, and
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) explains when a plain trait is enough
and when providers help.

## Where to go next

Each of these pages continues from a different part of this tour:

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
