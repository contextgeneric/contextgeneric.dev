---
sidebar_label: 'Dependency injection'
sidebar_position: 8
description: 'Compare CGP with Spring, Guice, Dagger, and plain Rust injection: dependency declarations, wiring, validation, and runtime values.'
---

# Dependency injection

CGP supports dependency injection through reusable providers selected by each application. It is a language extension for Rust, with pluggable trait implementations at
compile-time, implemented as a library on stable Rust whose consumer traits are ordinary Rust
traits; the [Introduction](/docs/) explains the basics. For readers familiar with Spring, Guice,
Dagger, or constructor injection, this page compares dependency declarations, wiring, validation,
and the cases that still call for a container.

## In your terms

A **context** is the type a CGP method runs on, supplying its data through fields and its
implementations through wiring. In the examples here, it represents an application and holds the
runtime values its providers need. The compiler resolves provider selection, while application code
constructs and manages those values.

| In a DI framework | In CGP |
| --- | --- |
| An implementation bound to an interface | A **provider**: a named, interchangeable implementation |
| A configuration class or module | **Wiring**, written in a `delegate_components!` table |
| A constructor dependency | An **[impl-side dependency](/docs/reference/glossary#impl-side-dependency)**, declared with `#[uses]` or `#[implicit]` |
| The interface a bean implements | A consumer trait, grouped with its provider trait into a **component** |
| Graph validation at startup or build time | `check_components!`, at compile time |
| Container-managed instances and lifetimes | Context fields and ordinary Rust construction, ownership, and borrowing |

## The idea, briefly

Dependency injection supplies an object's collaborators from outside the object. A class that
requires a storage interface can receive a production client or a test fake without changing its
business logic. DI frameworks automate the construction and connection of these objects, which
becomes useful as the dependency graph grows.

### The IoC container and beans

Spring's inversion-of-control container constructs, configures, and connects application objects
called beans. An annotation can register a class for discovery, while a configuration class can
provide a factory for a particular interface. These fragments illustrate both forms:

```java
@Service
public class UserService {
    // business logic lives here
}

@Configuration
public class AppConfig {
    @Bean
    public StorageClient storageClient() {
        return new S3StorageClient(/* ... */);
    }
}
```

### Constructor, setter, and field injection

Constructor injection makes required dependencies visible in the constructor's signature. The
container passes them as arguments, and the object can keep them in `final` fields:

```java
@Service
public class ProfilePictureService {
    private final StorageClient storage;
    private final UserRepository users;

    public ProfilePictureService(StorageClient storage, UserRepository users) {
        this.storage = storage;
        this.users = users;
    }
}
```

Setter and field injection supply dependencies after construction. A setter exposes an assignment
method; field injection lets the framework populate an annotated field. Spring recommends
constructor injection for required dependencies because it supports immutable objects and ensures
that an object starts with those dependencies supplied. The same constructor can be called directly
in a test. See the
[Spring reference](https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html).

### Modules and bindings

Guice modules explicitly bind interfaces to implementations. This module selects a storage client:

```java
public class StorageModule extends AbstractModule {
    @Override
    protected void configure() {
        bind(StorageClient.class).to(S3StorageClient.class);
    }
}
```

DI frameworks differ in when and how they resolve bindings. Spring and Guice commonly assemble
objects at runtime, using reflective mechanisms. Dagger generates construction and wiring code at
compile time and validates the bindings needed by a component. CGP also resolves implementation
choices at compile time, but uses Rust's trait system rather than an annotation processor. See the
[Dagger developer guide](https://dagger.dev/dev-guide/).

### Dependency injection without a framework

Rust traits and generics already support dependency injection. A service can receive any storage
value implementing its required interface:

```rust
trait StorageClient {
    fn fetch(&self, object_id: &str) -> Vec<u8>;
}

struct ProfilePictureService<S: StorageClient> {
    storage: S,
}
```

This design is often sufficient. Different storage types can implement `StorageClient`, so replacing
one collaborator does not itself require CGP. The additional work appears when generic parameters
spread through enclosing types, or when reusable implementations overlap for the same target type.
CGP addresses those cases with a shared context and separately named providers. The
[coherence explanation](/docs/concepts/coherence) develops the latter problem.

## How CGP expresses it

CGP separates a provider's dependency requirements from an application's implementation choices.
The provider declares [impl-side dependencies](/docs/concepts/impl-side-dependencies), and the
context supplies the required traits and fields. Every context below is an [environmental context](/docs/reference/glossary#environmental-context):
a type representing an application, with [self-targeted](/docs/reference/glossary#self-targeted-component) components that operate through it.
The snippets omit supporting application types and some implementation bodies.

### Impl-side dependencies declare what an implementation needs

A CGP provider declares trait dependencies with `#[uses]` and field dependencies with `#[implicit]`.
This user-creation implementation needs a username-censorship operation and a database value:

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
        // ... insert the user with `database`
    }
}
```

`#[uses(CanCensorUsername)]` requires the context to implement the censorship trait. The
`#[implicit] database` parameter borrows the context's `database` field. Neither requirement appears
in the `CanManageUser` consumer trait, so a generic caller can require that trait without repeating
the database and censorship bounds. The dependencies remain explicit in the implementation and are
checked when the concrete context's wiring is checked or used. `Error` is the application's enum;
the database insertion is omitted here.

### Wiring selects implementations for an application

A [`delegate_components!`](/docs/reference/macros/delegate_components) table selects a provider
for each component. These contexts choose different storage implementations for the same interface:

```rust
#[cgp_component(StorageObjectFetcher)]
pub trait CanFetchStorageObject {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}

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
```

`App` routes storage calls to `FetchS3Object`, while `GCloudApp` routes them to `FetchGCloudObject`.
The compiler resolves both routes statically. A program may use either context or both; the wiring
does not require separate binaries or guarantee that one implementation is absent from a binary.
The providers identify behavior, while the contexts hold values such as client connections and
bucket names.

### Checking validates the declared dependencies

[`check_components!`](/docs/reference/macros/check_components) verifies that the selected provider's
transitive requirements are satisfied for a context:

```rust
check_components! {
    App {
        StorageObjectFetcherComponent,
    }
}
```

This check fails to compile if `FetchS3Object` needs a field or trait that `App` does not supply.
Like Dagger's graph validation, it catches missing declared dependencies during the build. It does
not validate credentials, network availability, or other runtime conditions. CGP wiring is
[lazy](/docs/concepts/check-traits), so an explicit check catches a missing dependency even before
application code calls the component.

## What each approach costs

DI frameworks centralize object construction and wiring, but their automation takes work to trace.
Field injection can hide dependencies from a class's constructor, and runtime graph assembly can
report missing bindings only when that graph is built. Constructor injection makes dependencies
visible; compile-time generation, as in Dagger, moves binding validation into the build. These costs
therefore depend on the framework and injection style. The
[Spring reference](https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html)
explains the injection trade-offs, and
[Shore's critique](https://www.jamesshore.com/v2/blog/2023/the-problem-with-dependency-injection-frameworks)
argues that framework automation can make a dependency graph harder to follow.

CGP requires declarations, wiring, and compile-time trait resolution. Developers must learn the
consumer/provider split and trace dependencies through the context. Its wiring selects providers
statically; runtime reconfiguration requires ordinary Rust mechanisms, such as enums or trait
objects, inside or alongside that wiring. CGP also leaves object construction and lifecycle
management to the application.

CGP's raw errors can be difficult to read because they include generated traits and types.
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) weighs the additional machinery against
plain traits, generics, and other alternatives.

## Where a DI framework is the better choice

A DI framework fits applications that rely on its object lifecycle support, runtime configuration,
or surrounding ecosystem. A JVM application built around Spring's bean model already has reasons
to use that model beyond selecting implementations. Dagger is an option when that application wants
compile-time graph validation.

Plain Rust traits and generics fit dependencies that can be expressed without extensive parameter
propagation or overlapping implementations. CGP becomes useful when reusable providers need
independent implementation choices per context and static wiring justifies the extra declarations.
A build-time dependency graph alone does not make CGP necessary.

## What to expect that differs

CGP resolves provider selection without a container object. The wiring consists of trait impls;
context fields still contain runtime values that application code must construct and manage.

CGP uses declared wiring rather than classpath scanning or bean discovery. Changing a wiring entry
requires compilation. Runtime choices can be represented by values or dispatch mechanisms that a
selected provider uses.

Each context type can choose a different provider for the same component. DI frameworks can also
support separate configurations or graphs; CGP records this distinction in Rust types.

Provider declarations expose their context dependencies. These declarations do not describe every
possible influence on the body: a provider can still access globals or perform I/O through ordinary
Rust APIs.

## Where to go next

These pages explain the mechanisms behind the comparison:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): requirements declared by an
  implementation and hidden behind its consumer trait.
- [Checking your wiring](/docs/concepts/check-traits): why wiring is lazy and what a check verifies.
- [Implicit arguments](/docs/concepts/implicit-arguments): reading values from context fields.
- [ML modules](./ml-modules.md) and [Policy-based design](./policy-based-design.md): other approaches
  to choosing implementations and assembling dependencies.

## Sources

The CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!` assertion per
wired context. The Java snippets follow the framework documentation cited below.

- [Dagger developer guide](https://dagger.dev/dev-guide/): generated wiring, component graphs, and compile-time validation.
- [Spring Framework reference, *Dependency Injection*](https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html): the IoC container, beans, and the constructor and setter injection mechanisms.
- [Baeldung, *Inversion of Control and Dependency Injection in Spring*](https://www.baeldung.com/inversion-control-and-dependency-injection-in-spring): the distinction between IoC and DI and `@Autowired` resolution by type.
- [Comparing Dependency Injection Frameworks](https://medium.com/@AlexanderObregon/comparing-dependency-injection-frameworks-spring-guice-and-dagger-a614dccd5859) and [Dagger vs Guice](https://www.hackingnote.com/en/versus/dagger-vs-guice/): the runtime-versus-compile-time split across frameworks.
- [Nuri, *Field injection is not recommended*](https://blog.marcnuri.com/field-injection-is-not-recommended) and [Shore, *The Problem With Dependency Injection Frameworks*](https://www.jamesshore.com/v2/blog/2023/the-problem-with-dependency-injection-frameworks): the hidden-dependency and runtime-failure concerns, and the case for constructor injection.
- [jmmv.dev, *Rust traits and dependency injection*](https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html): the position that Rust performs dependency injection through traits and generics without a framework.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
