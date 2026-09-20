---
sidebar_label: 'Dependency injection'
sidebar_position: 8
description: 'CGP read against Spring, Guice, and Dagger: injection without a container, reflection, or runtime graph.'
---

# Dependency injection

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who knows
dependency injection (DI) from Spring, Guice, Dagger, or their kin. CGP solves the same decoupling
problem, at compile time and without a container, and the vocabulary maps almost directly. The page
covers that mapping, what changes when resolution moves from runtime to types, where a DI framework
remains the better tool, and what to expect that differs.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. In DI terms it is the assembled object graph, made a type you define to stand for an
application.

| In a DI framework | In CGP |
| --- | --- |
| A bean or a binding | A **provider**: an interchangeable implementation of a trait |
| The `@Configuration` class or the Guice module | **Wiring**, written in a `delegate_components!` table |
| A constructor parameter | An **impl-side dependency**, declared with `#[uses]` or `#[implicit]` |
| The interface a bean is bound to | A **component**: one trait with many possible implementations |
| Graph validation at startup or build time | `check_components!`, at compile time |
| The container | The type system; nothing exists at runtime |

## The idea, briefly

Dependency injection gives an object its collaborators from the outside instead of letting it
construct them. A class that names only the *interfaces* it needs can be handed fakes in a test and
different implementations in a different deployment, and never changes. The frameworks automate the
supplying, which in a large object graph is elaborate enough that they exist for it.

### The IoC container and beans

Spring's core is an *inversion-of-control container* that instantiates, configures, and connects the
application's objects, its *beans*. A class becomes a bean with an annotation, and a `@Configuration`
class spells out which implementation stands in for an interface:

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

The container injects each bean's dependencies by one of three mechanisms. *Constructor injection*
passes them as constructor arguments, so they are required and the fields can be `final`:

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

*Setter injection* supplies a dependency after construction, and *field injection* writes it into a
private field by reflection, marked `@Autowired`. The Spring reference and the wider community
recommend constructor injection for required dependencies, because it makes a class's dependencies
explicit in its signature and lets the object be built without a container in a test
([Spring Framework reference](https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html)).

### Modules and bindings

Guice and Dagger express the same wiring by explicit *bindings* in a *module*:

```java
public class StorageModule extends AbstractModule {
    @Override
    protected void configure() {
        bind(StorageClient.class).to(S3StorageClient.class);
    }
}
```

Spring and Guice resolve bindings at runtime through reflection. Dagger resolves them at *compile
time*: its annotation processor generates the wiring code during the build, so a missing binding is a
compile error and there is no reflection at runtime. That split is the sharpest axis of variation
among DI frameworks, and CGP sits at the compile-time end of it.

### Dependency injection without a framework

Rust practitioners generally hold that the language needs no DI framework, because traits and
generics already decouple a component from its collaborators:

```rust
trait StorageClient {
    fn fetch(&self, object_id: &str) -> Vec<u8>;
}

struct ProfilePictureService<S: StorageClient> {
    storage: S,
}
```

This is dependency injection in the original sense, with the compiler doing the checking
([jmmv.dev, *Rust traits and dependency injection*](https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html)).
Its limitation is the one CGP lifts. A bound like `S: StorageClient` leaks into every caller's
signature, and coherence permits one `impl StorageClient` per type, so offering several
interchangeable implementations of one interface runs into the rules the
[Bypassing coherence](/docs/concepts/coherence) page explains. CGP is the next step along the line
Rust already accepts, not an import of the container model.

## How CGP expresses it

CGP performs dependency injection through two mechanisms working together. A provider declares what
it needs as [impl-side dependencies](/docs/concepts/impl-side-dependencies), and a context supplies
them by wiring each component to a provider. Both are resolved by the compiler, so the "container"
has no runtime existence. Every context below is an **environmental context**: a type standing for an
application, carrying the application's choices and dependencies rather than being the data operated
on.

### Impl-side dependencies are the injected constructor parameters

Where a Spring service lists `StorageClient` and `UserRepository` as constructor parameters, a CGP
provider lists its trait dependencies with `#[uses]` and its value dependencies as `#[implicit]`
arguments. A user-creation provider that needs a database connection and a censorship service declares
both:

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

`#[uses(CanCensorUsername)]` injects a *trait dependency*, the role a `UserRepository` collaborator
plays in the constructor. The `#[implicit] database` argument injects a *value* pulled from the
context's `database` field, the role a configuration bean plays. Neither appears in the
`CanManageUser` consumer trait a caller invokes, so, unlike a leaked generic bound, they do not
cascade to callers. That is the decoupling a DI framework promises, delivered by declaring the
requirements one level down in the provider rather than in a container. `Error` here is the
application's own enum, and the insert body is elided.

### Wiring is the container configuration

A context selects which provider satisfies each component in a
[`delegate_components!`](/docs/reference/macros/delegate_components) table, the direct analogue of a
`@Configuration` class or a Guice module. Two contexts map the same component to different providers
with no conflict, because the choice is keyed on the context type:

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

`FetchS3Object` and `FetchGCloudObject` are interchangeable providers of the same trait, the
equivalent of two beans bound to one interface, and the wiring picks one per context. Because the
selection is resolved during type checking and monomorphized to a direct call, the `App` binary
contains only the S3 code path and the `GCloudApp` binary only the GCloud one. A container makes the
same substitution by holding both implementations and choosing at startup.

### Checking replaces the container's startup validation

A container discovers a missing or ambiguous binding when it assembles the graph: at startup for
Spring and Guice, at build time for Dagger. CGP's counterpart is
[`check_components!`](/docs/reference/macros/check_components), which asserts at compile time that a
context's wiring is complete and every provider's transitive dependencies are satisfied:

```rust
check_components! {
    App {
        StorageObjectFetcherComponent,
    }
}
```

If `FetchS3Object` needs a field or trait the `App` context does not supply, this fails to compile
with the missing dependency named, rather than surfacing as a startup exception or a null reference
deep in a request. It is the guarantee Dagger gives, reached through the trait system instead of an
annotation processor. CGP wiring is [lazy](/docs/concepts/check-traits), so this check turns a latent gap into an
early error.

## What each approach costs

DI frameworks are valued for decoupling components from their collaborators, which makes code testable
and swappable, for centralizing wiring in one readable place, and, in Spring's case, for the ecosystem
keyed off the same bean model. Their costs, as their users state them, cluster around the runtime,
reflective nature of the popular frameworks. Dependencies can become hidden: with field injection a
class's signature says nothing about what it needs, and a missing binding is a runtime failure rather
than a compile error, which is why the community steers toward constructor injection
([Nuri, *Field injection is not recommended*](https://blog.marcnuri.com/field-injection-is-not-recommended)).
Classpath scanning and reflective graph construction cost startup time, which is why Dagger's
compile-time generation exists. And the frameworks do so much automatically that when something goes
wrong the developer has little visibility into why
([Shore, *The Problem With Dependency Injection Frameworks*](https://www.jamesshore.com/v2/blog/2023/the-problem-with-dependency-injection-frameworks)).

CGP resolves everything at compile time, and that is its central cost as well as its benefit. It
cannot reconfigure an application without recompiling, load plugins chosen at startup from a config
file, or build a graph whose shape is not known until the program runs. It is confined to Rust. Its
machinery, the consumer and provider split, the wiring, and the type-level tables, has a learning
curve of its own. And its raw diagnostics are trait-solver output over generated types:
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives, including plain traits and generics.

## Where a DI framework is the better choice

A DI framework remains the better choice when the application needs runtime reconfiguration, when it
lives in a JVM or .NET ecosystem whose libraries assume the container, or when the team's familiarity
with the framework outweighs the benefits of static wiring. Within Rust, plain traits and generics are
the better choice when one implementation per type suffices and the bound does not spread far. CGP is
the better tool where the graph is known at build time and the guarantees and zero runtime cost
matter: systems programming, latency-sensitive services, and libraries that must not impose a runtime.

## What to expect that differs

**There is no container object.** A DI reader will look for something holding the graph and resolving
dependencies by reflection at startup. CGP's container is the type system, the graph is a set of
trait impls, and the resolution happens during compilation and compiles to direct calls. There is no
runtime cost and no runtime failure mode to look for.

**There is no scanning or auto-registration.** CGP asks for explicit wiring. There is no classpath
scan, no `@Component` discovery, and no runtime rebinding, and the graph must be known at compile
time. This is the price of the guarantees, the same trade Dagger made, carried to its conclusion.

**The same interface can resolve differently per context.** A single global binding graph cannot
express two applications binding one interface two ways. In CGP that is one wiring line each.

**Dependencies are never hidden.** They are stated in `#[uses]` and `#[implicit]` and enforced by the
compiler, which gives the explicitness the community prizes in constructor injection by default.

## Where to go next

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): the construct this page maps onto
  constructor parameters.
- [Checking your wiring](/docs/concepts/check-traits): why wiring is lazy and what the check catches.
- [Implicit arguments](/docs/concepts/implicit-arguments): value injection from context fields.
- [ML modules](./ml-modules.md) and [Policy-based design](./policy-based-design.md): the same
  centralized wiring seen from two other traditions.

## Sources

The CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!` assertion per
wired context. The Java snippets follow the framework documentation cited below.

- [Spring Framework reference, *Dependency Injection*](https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html): the IoC container, beans, and the constructor and setter injection mechanisms.
- [Baeldung, *Inversion of Control and Dependency Injection in Spring*](https://www.baeldung.com/inversion-control-and-dependency-injection-in-spring): the distinction between IoC and DI and `@Autowired` resolution by type.
- [Comparing Dependency Injection Frameworks](https://medium.com/@AlexanderObregon/comparing-dependency-injection-frameworks-spring-guice-and-dagger-a614dccd5859) and [Dagger vs Guice](https://www.hackingnote.com/en/versus/dagger-vs-guice/): the runtime-versus-compile-time split across frameworks.
- [Nuri, *Field injection is not recommended*](https://blog.marcnuri.com/field-injection-is-not-recommended) and [Shore, *The Problem With Dependency Injection Frameworks*](https://www.jamesshore.com/v2/blog/2023/the-problem-with-dependency-injection-frameworks): the hidden-dependency and runtime-failure concerns, and the case for constructor injection.
- [jmmv.dev, *Rust traits and dependency injection*](https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html): the position that Rust performs dependency injection through traits and generics without a framework.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
