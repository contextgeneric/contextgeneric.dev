---
slug: extensible-datatypes-part-1
title: 'Programming Extensible Data Types in Rust with CGP - Part 1: Modular App Construction and Extensible Builders'
authors: [soares]
tags: [release, deepdive]
---

I’m excited to announce the release of [**CGP v0.4.2**](https://github.com/contextgeneric/cgp/releases/tag/v0.4.2), a major milestone that significantly expands the expressive power of generic programming in Rust. With this release, CGP introduces full support for **extensible records and variants**, unlocking a range of new capabilities for developers working with highly modular and reusable code.

Extensible records and variants allow developers to write code that operates on *any struct containing specific fields* or *any enum containing specific variants*, without needing their concrete definition. This makes it possible to write truly generic and flexible logic that is decoupled from rigid type definitions.

<!-- truncate -->

In earlier versions, CGP already offered a foundational feature through the `HasField` trait, which made it possible to *read* a field from any struct that included it. With version 0.4.2, this functionality is dramatically extended. Not only can you now read fields, but you can also *construct* values onto these fields in a type-safe manner. More importantly, the same level of extensibility is now available for enums, enabling operations over variants in a similarly generic fashion.

This advancement introduces two powerful programming patterns that are now possible with CGP:

1. **Extensible Builder Pattern**: This pattern allows for modular construction of structs from independent sub-structs, each contributing specific fields. It enables highly composable and decoupled design in data construction.

2. **Extensible Visitor Pattern**: This pattern enables the modular deconstruction of enums, allowing independent components to handle different variants without requiring full knowledge of the entire enum definition. This effectively enables a modularized version of the _visitor pattern_, by allowing new variants to be handled by extensible visitors.

For readers coming from more advanced programming languages, this development effectively brings the power of [**datatype-generic programming**](https://wiki.haskell.org/index.php?title=Generics), [**structural typing**](https://en.wikipedia.org/wiki/Structural_type_system), [**row polymorphism**](https://book.purescript.org/chapter4.html) and [**polymorphic variants**](https://ocaml.org/manual/5.1/polyvariant.html) to Rust. These are advanced type system features commonly found in languages like Haskell, PureScript and OCaml, and their availability in CGP represents a major leap in what is possible with the type system in Rust.

In addition, CGP v0.4.2 introduces support for safe **upcasting and downcasting between enums** that share a common subset of variants. This provides a foundation for writing extensible and evolvable APIs that remain compatible across different layers of abstraction or across independently maintained modules.

## Discussion

Discuss on [Reddit](https://www.reddit.com/r/rust/comments/1ltu9nl/programming_extensible_data_types_in_rust_with/), [Lobsters](https://lobste.rs/s/piugxk/programming_extensible_data_types_rust), [Hacker News](https://news.ycombinator.com/item?id=44497815), [GitHub](https://github.com/orgs/contextgeneric/discussions/12), or [Discord](https://discord.gg/Hgk3rCw6pQ).


# Series Overview

This article is the first in a five-part series exploring the examples and implementation of extensible data types in CGP. Below is an overview of what each part covers:

**Part 1: Modular App Construction and Extensible Builders** (this post) – In this introductory part, we present a high-level overview of the key features enabled by extensible data types. We then dive into a hands-on demonstration showing how extensible records can be used to build and compose modular builders for real-world applications.

[**Part 2: Modular Interpreters and Extensible Visitors**](/blog/extensible-datatypes-part-2) – This part continues the demonstration by introducing extensible variants. We use them to address the [**expression problem**](https://en.wikipedia.org/wiki/Expression_problem), implementing a set of reusable interpreter components for a small toy language.

[**Part 3: Implementing Extensible Records**](/blog/extensible-datatypes-part-3) – Here, we walk through the internal mechanics behind extensible records. We show how CGP supports the modular builder pattern demonstrated in Part 1 through its underlying type and trait machinery.

[**Part 4: Implementing Extensible Variants**](/blog/extensible-datatypes-part-4) – This part mirrors Part 3, but for extensible variants. We examine how extensible variants are implemented, and compare the differences and similarities between extensible records and variants.

# Feature Highlighs

## Safe Enum Upcasting

Let’s begin by looking at how CGP enables safe upcasting between enums. Imagine you have the following enum definition called `Shape`:

```rust
#[derive(HasFields, FromVariant, ExtractField)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
```

You may also have a different `ShapePlus` enum, defined elsewhere, that represents a *superset* of the variants in `Shape`:

```rust
#[derive(HasFields, FromVariant, ExtractField)]
pub enum ShapePlus {
    Triangle(Triangle),
    Rectangle(Rectangle),
    Circle(Circle),
}
```

With CGP v0.4.2, it is now possible to *upcast* a `Shape` value into a `ShapePlus` value in fully safe Rust:

```rust
let shape = Shape::Circle(Circle { radius: 5.0 });
let shape_plus = shape.upcast(PhantomData::<ShapePlus>);
assert_eq!(shape_plus, ShapePlus::Circle(Circle { radius: 5.0 }));
```

This operation works by leveraging the derived CGP traits `HasFields`, `ExtractField`, and `FromVariant`. As long as the source enum’s variants are a subset of the target enum’s, CGP can automatically generate the logic required to lift the smaller enum into the larger one.

A particularly powerful aspect of this design is that the two enums do not need to know about each other. They can be defined in entirely separate crates, and the trait derivations are completely general. You don’t need to define any enum-specific conversion traits. This makes it possible to build libraries of reusable variant groups and compose them freely in application code.

## Safe Enum Downcasting

In the reverse direction, CGP also supports *safe downcasting* from a larger enum to a smaller one that contains only a subset of its variants. Using the same `Shape` and `ShapePlus` enums, the following example demonstrates how this works:

```rust
let shape = ShapePlus::Circle(Circle { radius: 5.0 });

assert_eq!(
    shape.downcast(PhantomData::<Shape>).ok(),
    Some(Shape::Circle(Circle { radius: 5.0 }))
);
```

Like `upcast`, this `downcast` method relies on the same set of derived CGP traits and works for any pair of compatible enums. The operation returns a `Result`, where the `Ok` variant contains the downcasted value, and the `Err` variant carries the unhandled remainder of the original enum.

In the example above, we use `.ok()` to simplify the comparison, but in practice, the `Err` case contains useful remainder value that can be further examined or downcasted again.

### Safe Exhaustive Downcasting

One of the unique capabilities CGP provides is the ability to *exhaustively downcast* an enum, step by step, until all possible variants are handled. This pattern becomes especially useful when working with generic enums in extensible APIs, where the concrete enum definition is unknown or evolving.

To demonstrate this, suppose we define another enum to represent the remaining `Triangle` variant:

```rust
#[derive(HasFields, ExtractField, FromVariant)]
pub enum TriangleOnly {
    Triangle(Triangle),
}
```

Now, the combination of `Shape` and `TriangleOnly` covers the entire set of variants from `ShapePlus`. We can use this setup to exhaustively handle all possible cases, while staying entirely within the bounds of safe Rust:

```rust
let shape_plus = ShapePlus::Triangle(Triangle {
    base: 3.0,
    height: 4.0,
});

let area = match shape_plus.downcast(PhantomData::<Shape>) {
    Ok(shape) => match shape {
        Shape::Circle(circle) => PI * circle.radius * circle.radius,
        Shape::Rectangle(rectangle) => rectangle.width * rectangle.height,
    },
    Err(remainder) => match remainder.downcast_fields(PhantomData::<TriangleOnly>) {
        Ok(TriangleOnly::Triangle(triangle)) => triangle.base * triangle.height / 2.0,
    },
};
```

In this example, we first attempt to downcast into `Shape`. If that fails, the remainder is passed to `downcast_fields`, which attempts to further downcast to `TriangleOnly`. When all variants are properly handled, Rust automatically knows that there is no variant left to be handled, and we can safely omit the final `Err` case.

At first glance, this approach may appear more complex than simply matching against the original enum directly. However, its true strength lies in its **generality**. With CGP’s downcasting mechanism, you can pattern match over generic enum types without knowing their full structure in advance. This enables highly extensible and type-safe designs where variants can be added or removed modularly, without breaking existing logic.

## Safe Struct Building

Just as CGP enables safe, composable deconstruction of enums, it also brings **extensible construction** to structs. This is achieved through a form of structural merging, where smaller structs can be incrementally combined into larger ones. The result is a flexible and modular approach to building complex data types, well-suited for highly decoupled or plugin-style architectures.

To illustrate this, let’s take the example of a `Employee` struct:

```rust
#[derive(HasFields, BuildField)]
pub struct Employee {
    pub employee_id: u64,
    pub first_name: String,
    pub last_name: String,
}
```

Suppose we also define two smaller structs — `Person` and `EmployeeId` — each containing a subset of the fields in `Employee`:

```rust
#[derive(HasFields, BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

#[derive(HasFields, BuildField)]
pub struct EmployeeId {
    pub employee_id: u64,
}
```

With CGP, we can now construct a `Employee` value in a modular and extensible way, by composing these smaller building blocks:

```rust
let person = Person {
    first_name: "John".to_owned(),
    last_name: "Smith".to_owned(),
};

let employee_id = EmployeeId { employee_id: 1 };

let employee = Employee::builder()
    .build_from(person)
    .build_from(employee_id)
    .finalize_build();
```

Here’s what’s happening: The `builder()` method on `Employee` initiates a *partial record* builder, an intermediate structure that initially contains none of the target fields. Each call to `build_from` takes a struct that contributes one or more of the remaining fields and returns a new builder with those fields filled in. Once all required fields have been supplied, the `finalize_build()` method consumes the builder and produces a fully constructed `Employee` instance.

Just like enum upcasting and downcasting, the struct builder is implemented entirely in **safe**, **panic-free** Rust. There’s no runtime reflection or unsafe code involved. The only requirement is that the participating structs must have compatible fields and derive the CGP-provided traits `HasFields` and `BuildField`.

Moreover, this system is completely decoupled from specific struct definitions. The individual component structs — `Person`, `EmployeeId`, and `Employee` — can be defined in separate crates, with no awareness of each other. Once the CGP traits are derived, they become interoperable through structural field compatibility alone.

While this example may seem trivial — after all, constructing `Employee` directly is straightforward — it serves as a foundation for much more powerful generic abstractions. As you’ll see in the upcoming sections, the builder pattern opens the door to writing highly reusable, type-safe logic that can construct **generic types** without ever referencing their concrete types. This makes it possible to write libraries or plugins that contribute data to a shared structure without tight coupling or dependency on a central type definition.

# Motivation for Extensible Builders

To understand how extensible records enable modular builders, let’s explore a practical use case: constructing an application context from configuration inputs.

Imagine we’re building an API client for our application. The application context needs to include an SQLite database connection and an HTTP client. A typical way to model this in Rust would be to define a struct like the following:

```rust
#[cgp_context]
pub struct App {
    pub sqlite_pool: SqlitePool,
    pub http_client: Client,
}
```

This `App` struct holds a [`SqlitePool`](https://docs.rs/sqlx/latest/sqlx/sqlite/type.SqlitePool.html) from the `sqlx` crate, and an HTTP [`Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html) from `reqwest`. To construct this context, we might implement a `new` function as follows:

```rust
impl App {
    pub async fn new(db_path: &str) -> Result<Self, Error> {
        let http_client = Client::new();
        let sqlite_pool = SqlitePool::connect(db_path).await?;

        Ok(Self {
            http_client,
            sqlite_pool,
        })
    }
}
```

This constructor is asynchronous and returns a `Result<App, Error>`. It creates a default `Client` using `reqwest`, connects to the database using the provided path, and assembles both into an `App` struct.

## Adding AI Capabilities to `App`

At this point, the constructor looks simple. But in a real-world setting, it’s rarely that clean. Suppose the product team now wants to integrate AI capabilities into the application. To support this, we decide to use an LLM service like ChatGPT and extend the `App` struct accordingly:

```rust
#[cgp_context]
pub struct App {
    pub sqlite_pool: SqlitePool,
    pub http_client: Client,
    pub open_ai_client: openai::Client,
    pub open_ai_agent: Agent<openai::CompletionModel>,
}
```

In this updated version, we introduce two new fields: `open_ai_client`, which is used to communicate with the OpenAI API, and `open_ai_agent`, which encapsulates a configured agent that can perform conversational tasks using a model like GPT-4o using [`rig`](https://docs.rs/rig-core/latest/rig/index.html).

The `new` constructor must now also handle the initialization logic for these fields:

```rust
impl App {
    pub async fn new(db_path: &str) -> Result<Self, Error> {
        let http_client = Client::new();
        let sqlite_pool = SqlitePool::connect(db_path).await?;
        let open_ai_client = openai::Client::from_env();
        let open_ai_agent = open_ai_client.agent("gpt-4o").build();

        Ok(Self {
            http_client,
            sqlite_pool,
            open_ai_client,
            open_ai_agent,
        })
    }
}
```

Here, we initialize the OpenAI client using environment variables, and then build an agent configured for the `gpt-4o` model. These values are added alongside the existing HTTP and database clients.

## From Simple to Complex

Even with these additions, our constructor remains relatively manageable. However, as often happens in production, the requirements grow—and so does the configuration logic. Let’s imagine a more realistic version of this `new` function:

```rust
impl App {
    pub async fn new(
        db_options: &str,
        db_journal_mode: &str,
        http_user_agent: &str,
        open_ai_key: &str,
        open_ai_model: &str,
        llm_preamble: &str,
    ) -> Result<Self, Error> {
        let journal_mode = SqliteJournalMode::from_str(db_journal_mode)?;

        let db_options = SqliteConnectOptions::from_str(db_options)?.journal_mode(journal_mode);

        let sqlite_pool = SqlitePool::connect_with(db_options).await?;

        let http_client = Client::builder()
            .user_agent(http_user_agent)
            .connect_timeout(Duration::from_secs(5))
            .build()?;

        let open_ai_client = openai::Client::new(open_ai_key);
        let open_ai_agent = open_ai_client
            .agent(open_ai_model)
            .preamble(llm_preamble)
            .build();

        Ok(Self {
            open_ai_client,
            open_ai_agent,
            sqlite_pool,
            http_client,
        })
    }
}
```

This constructor now handles *five* separate input parameters, each contributing to the configuration of different parts of the application. It creates a `SqliteConnectOptions` object to configure the database with the specified journal mode. The HTTP client is set up with a custom user agent and a longer timeout. The AI client is initialized using an explicit API key, and the agent is constructed with a custom model and preamble.

While none of these steps are especially difficult on their own, the function is starting to grow in complexity. It’s also becoming more **fragile**, as all responsibilities are bundled into one place. Every change to a single subsystem — whether it’s database, HTTP, or AI — requires editing the same constructor.

## Why Modular Constructor Matters

As we've seen in the previous example, even modest configurability can cause a constructor's complexity to grow rapidly. With just a few additional fields or customization options, the function becomes harder to maintain, test, and reason about.

In many cases, there's no single “correct” way to construct an application context. For example, you might want to retain both versions of the `new` constructor from earlier: a minimal one for unit tests with default values, and a more elaborate, configurable one for production. In fact, it's common for different parts of an application to require different levels of configurability—some using defaults, others requiring fine-grained setup.

To manage this complexity, Rust developers often reach for the [*builder pattern*](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html). This involves creating a separate builder struct, typically with optional or defaultable fields and fluent setter methods. The builder is used to gradually assemble values before producing the final struct.

## Challenges for Modular Builders

The traditional builder pattern works, but it comes with serious limitations — especially when extensibility and modularity are important.

The first limitation is **tight coupling**. A builder is usually tied directly to a specific target struct. If you create a new context that’s only slightly different from an existing one, you often have to duplicate the entire builder implementation, even if most of the logic is the same.

Second, builders are typically **non-extensible**. If you want to extend the construction logic — say, by adding a new step to initialize an additional field — you usually have to modify the original builder struct. This makes it hard to share construction logic across crates or teams without exposing internal implementation details.

The root cause of these problems is that struct construction in Rust typically requires direct access to the **concrete type**. That means the builder must know the exact shape of the final struct and have access to all its field values up front. If you need intermediate values or want to plug in custom build steps, those values must be manually threaded through the builder and its state.

This rigidity makes it difficult to define reusable, composable building blocks—especially in large or evolving codebases.

## Modular Builders with CGP

Earlier versions of CGP also ran into these limitations. When writing *context-generic* code, we wanted to construct structs in a way that didn’t require knowing their concrete types ahead of time. But because Rust structs require all field values to be present simultaneously at construction time, we couldn’t easily implement flexible or reusable context-generic constructors.

With the latest release, that limitation is fully resolved.

CGP now supports **modular, extensible struct builders** that can be composed from smaller, independent parts. Each module can define how to build a piece of a context struct, and the builder automatically merges them — without needing to know the final shape of the struct ahead of time.

This opens the door to a new style of constructor logic: one that is **modular**, **composable**, and **context-generic**. You can define builders for individual subsystems (e.g., database, HTTP client, AI agent), and combine them to build any compatible application context.

# Extensible Builders

In this section, we’ll revisit the constructor examples we’ve already seen — and show how to rewrite them using CGP’s new builder pattern to achieve clean, modular, and reusable construction logic. A full version of the example code covered in this section is available on our [GitHub repository](https://github.com/contextgeneric/cgp-examples/tree/main/builder)

## Modular SQLite Builder

Let’s now explore how to implement modular construction of the `App` context using multiple CGP providers. We’ll start by defining a default SQLite builder provider using CGP's `Handler` component:

```rust
#[cgp_new_provider]
impl<Build, Code: Send, Input: Send> Handler<Build, Code, Input> for BuildDefaultSqliteClient
where
    Build: HasSqlitePath + CanRaiseAsyncError<sqlx::Error>,
{
    type Output = SqliteClient;

    async fn handle(
        build: &Build,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Self::Output, Build::Error> {
        let sqlite_pool = SqlitePool::connect(build.db_path())
            .await
            .map_err(Build::raise_error)?;

        Ok(SqliteClient { sqlite_pool })
    }
}
```

In this example, we define `BuildDefaultSqliteClient` as a CGP provider that implements the `Handler` component. This is the same `Handler` trait we introduced in [Hypershell](/blog/hypershell-release/#handler-component), where it was used to power shell-like pipelines. Here, we repurpose the same trait to construct modular context components. This demonstrates how general-purpose the `Handler` trait is — it can be used for pipelines, API handlers, visitors, and now, context builders.

The `Build` type parameter refers to a generic **builder context**, not the final `App` struct. This context includes the inputs required to construct a `SqliteClient`. In this case, the builder must be able to provide a database path, as well as a way to raise errors from `sqlx`. These requirements are expressed through the `HasSqlitePath` and `CanRaiseAsyncError` constraints.

The `HasSqlitePath` trait is defined as follows:

```rust
#[cgp_auto_getter]
pub trait HasSqlitePath {
    fn db_path(&self) -> &str;
}
```

By marking the trait with [`#[cgp_auto_getter]`](https://patterns.contextgeneric.dev/generic-accessor-providers.html#the-cgp_auto_getter-macro), CGP can automatically implement this trait for any builder context that contains a `db_path` field of type `String`. This automatic implementation reduces boilerplate and ensures that any context with the appropriate fields can satisfy the trait bounds.

Although our example does not make use of the `Code` or `Input` parameters, they remain part of the `Handler` signature. The `Code` parameter may be used for *compile-time options* that allow contexts to be constructed in multiple ways. Meanwhile, `Input` typically refers to the **partial** value of the final struct being built. These capabilities are useful in more advanced scenarios, but we will leave their explanation for a later section.

In this implementation, the `handle` method simply connects to the SQLite database using the provided path, wraps the resulting pool in a `SqliteClient` struct, and returns it. The `SqliteClient` is defined as:

```rust
#[derive(HasField, HasFields, BuildField)]
pub struct SqliteClient {
    pub sqlite_pool: SqlitePool,
}
```

This struct acts as a wrapper around `SqlitePool` and serves as the output of our modular builder. Although `BuildDefaultSqliteClient` does not build the full `App` context, we can merge its output into `App` using CGP’s `build_from` mechanism we covered earlier. Deriving `HasField`, `HasFields`, and `BuildField` on `SqliteClient` allows it to be safely and automatically merged into the final context during composition.

At this point, you might be wondering why so much infrastructure is needed just to call `SqlitePool::connect`. The answer is that, while this example is simple, real-world construction logic can be much more complex. By encapsulating each part of the logic into modular components, we gain flexibility, reusability, and testability.

To demonstrate this flexibility, consider a more complex version of the SQLite builder. This version uses connection options and journal mode configuration rather than a simple path string:

```rust
#[cgp_new_provider]
impl<Build, Code: Send, Input: Send> Handler<Build, Code, Input> for BuildSqliteClient
where
    Build: HasSqliteOptions + CanRaiseAsyncError<sqlx::Error>,
{
    type Output = SqliteClient;

    async fn handle(
        build: &Build,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Self::Output, Build::Error> {
        let journal_mode =
            SqliteJournalMode::from_str(build.db_journal_mode()).map_err(Build::raise_error)?;

        let db_options = SqliteConnectOptions::from_str(build.db_options())
            .map_err(Build::raise_error)?
            .journal_mode(journal_mode);

        let sqlite_pool = SqlitePool::connect_with(db_options)
            .await
            .map_err(Build::raise_error)?;

        Ok(SqliteClient { sqlite_pool })
    }
}

#[cgp_auto_getter]
pub trait HasSqliteOptions {
    fn db_options(&self) -> &str;

    fn db_journal_mode(&self) -> &str;
}
```

In this version, `BuildSqliteClient` constructs a `SqliteClient` using fully configurable connection options. The `Build` context must now implement `HasSqliteOptions`, a trait that provides both the connection URI and the desired journal mode.

This example illustrates the key advantage of modular builders: the builder logic is entirely **decoupled** from the context itself. If we want to use `BuildDefaultSqliteClient`, we can define a simple builder context with just a `db_path` field. If we switch to `BuildSqliteClient`, we only need to provide a different context that includes `db_options` and `db_journal_mode`. All other components of the builder can remain unchanged.

Thanks to this decoupling, we can easily swap in different builder providers depending on the needs of the environment — be it development, testing, or production — without rewriting the entire construction logic. This modularity makes CGP builders highly scalable and adaptable to real-world applications.

## HTTP Client Builder

Just as we modularized the construction of the SQLite client, we can also define a modular builder for an HTTP client using CGP. In this case, we will construct a custom `reqwest` client with specific configuration options. To keep the focus on advanced use cases, we will skip the simpler version and go directly to the more complex construction logic.

The HTTP client builder is implemented as follows:

```rust
#[cgp_new_provider]
impl<Build, Code: Send, Input: Send> Handler<Build, Code, Input> for BuildHttpClient
where
    Build: HasHttpClientConfig + CanRaiseAsyncError<reqwest::Error>,
{
    type Output = HttpClient;

    async fn handle(
        build: &Build,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Self::Output, Build::Error> {
        let http_client = Client::builder()
            .user_agent(build.http_user_agent())
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(Build::raise_error)?;

        Ok(HttpClient { http_client })
    }
}
```

This provider, `BuildHttpClient`, is structured very similarly to `BuildSqliteClient`. It implements the `Handler` trait and defines `HttpClient` as its output. The `Build` context is required to implement two traits: `HasHttpClientConfig`, which supplies the necessary configuration values, and `CanRaiseAsyncError<reqwest::Error>`, which allows the context to convert `reqwest` errors into its own error type.

The required configuration is minimal. In this case, we only need a user agent string, which is defined through the following trait:

```rust
#[cgp_auto_getter]
pub trait HasHttpClientConfig {
    fn http_user_agent(&self) -> &str;
}
```

As with the previous examples, the `#[cgp_auto_getter]` macro ensures that this trait is automatically implemented for any context that includes a `http_user_agent` field.

The output of this builder is a simple wrapper around `reqwest::Client`:

```rust
#[derive(HasField, HasFields, BuildField)]
pub struct HttpClient {
    pub http_client: Client,
}
```

Here again, we derive `HasField`, `HasFields`, and `BuildField` to support field merging into the final context later on. This makes the `HttpClient` output compatible with CGP’s `build_from` mechanism, allowing it to be composed with other builder outputs.

The `handle` method creates a new `reqwest::Client` using the client builder from `reqwest`. It sets the user agent using a value from the context, and specifies a connection timeout of five seconds. The constructed client is then wrapped in the `HttpClient` struct and returned.

Although this example remains relatively simple, it illustrates how each field or component in a context can be modularly constructed using dedicated builder logic. Each builder is independently defined, type-safe, and reusable. If the way we configure our HTTP client changes — for example, if we want to support proxies or TLS settings — we can define a new provider that implements a different construction strategy, without needing to change any of the other components in our application context.

## Combined SQLite and HTTP Client Builder

Before we move on, it is important to emphasize that CGP does **not** require you to break down the construction logic of every component in your application context into separate builders. While the modular approach can offer more flexibility and reuse, you are entirely free to combine multiple construction tasks into a single provider if that better suits your needs.

For example, here is how you might implement a single builder that constructs *both* the SQLite client and the HTTP client together:

```rust
#[cgp_new_provider]
impl<Build, Code: Send, Input: Send> Handler<Build, Code, Input> for BuildDefaultSqliteAndHttpClient
where
    Build: HasSqlitePath + CanRaiseAsyncError<sqlx::Error>,
{
    type Output = SqliteAndHttpClient;

    async fn handle(
        build: &Build,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Self::Output, Build::Error> {
        let sqlite_pool = SqlitePool::connect(build.db_path())
            .await
            .map_err(Build::raise_error)?;

        let http_client = Client::new();

        Ok(SqliteAndHttpClient { sqlite_pool, http_client })
    }
}

#[derive(HasField, HasFields, BuildField)]
pub struct SqliteAndHttpClient {
    pub sqlite_pool: SqlitePool,
    pub http_client: Client,
}
```

In this implementation, we define a single provider `BuildDefaultSqliteAndHttpClient` that returns a combined struct `SqliteAndHttpClient`, which contains both a `SqlitePool` and a `reqwest::Client`. The construction logic is written in one place, which can be convenient when these components are always used together or when their configuration is tightly integrated.

However, the tradeoff of this approach is that it reduces flexibility. This tight coupling can limit reuse and make future changes more difficult.

That said, the choice of whether to combine or separate builders is **entirely up to you**. CGP does **not** impose any rules on how you must structure your builder logic. It provides the tools to compose and reuse components where helpful, but it leaves design decisions to the developer.

For the remainder of this article, we will continue to use the fully modular approach, breaking construction logic down into smaller, independent units. Our goal is to illustrate the full extent of flexibility and reusability that CGP enables. However, if you prefer a different organizational structure, you are free to structure your builders in whatever way best suits your project.

## ChatGPT Client Builder

Regardless of whether you prefer to split or combine the construction of components such as the SQLite and HTTP clients, there are many situations where it makes sense to separate construction logic into smaller, more focused units. For instance, you might want to offer two versions of your application — one standard version and one "smart" version that includes AI capabilities. In such cases, it is useful to define a separate builder provider for the ChatGPT client, so that AI-related logic can be included only when necessary.

The implementation for the ChatGPT client builder follows the same general pattern as the previous builders. It is defined as follows:

```rust
#[cgp_new_provider]
impl<Build, Code: Send, Input: Send> Handler<Build, Code, Input> for BuildOpenAiClient
where
    Build: HasOpenAiConfig + HasAsyncErrorType,
{
    type Output = OpenAiClient;

    async fn handle(
        build: &Build,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Self::Output, Build::Error> {
        let open_ai_client = openai::Client::new(build.open_ai_key());
        let open_ai_agent = open_ai_client
            .agent(build.open_ai_model())
            .preamble(build.llm_preamble())
            .build();

        Ok(OpenAiClient {
            open_ai_client,
            open_ai_agent,
        })
    }
}
```

This builder requires the `Build` context to provide three string fields: the OpenAI API key, the model name, and a custom preamble string. These requirements are captured by the `HasOpenAiConfig` trait:

```rust
#[cgp_auto_getter]
pub trait HasOpenAiConfig {
    fn open_ai_key(&self) -> &str;

    fn open_ai_model(&self) -> &str;

    fn llm_preamble(&self) -> &str;
}
```

As with the other providers, we use the `#[cgp_auto_getter]` macro to automatically implement the trait, as long as the builder context contains the corresponding fields and derives `HasField`.

The `BuildOpenAiClient` provider returns an `OpenAiClient` struct that wraps two values: the low-level `openai::Client` and the higher-level `Agent` configured with the specified model and preamble.

```rust
#[derive(HasField, HasFields, BuildField)]
pub struct OpenAiClient {
    pub open_ai_client: openai::Client,
    pub open_ai_agent: Agent<openai::CompletionModel>,
}
```

By defining this logic in a standalone builder provider, we can easily opt in or out of ChatGPT support in our application context.

## Builder Context

Now that we have implemented builder providers for SQLite, HTTP, and ChatGPT clients, we can demonstrate how to combine them in a complete builder context that constructs the final `App` instance. Defining this context is surprisingly concise and requires only a few lines of code:

```rust
#[cgp_context]
#[derive(HasField, Deserialize)]
pub struct FullAppBuilder {
    pub db_options: String,
    pub db_journal_mode: String,
    pub http_user_agent: String,
    pub open_ai_key: String,
    pub open_ai_model: String,
    pub llm_preamble: String,
}
```

Here, we define a `FullAppBuilder` struct that includes all of the fields required by the three individual builder providers. The `#[cgp_context]` macro enables the CGP capabilities for the context struct, while the `HasField` derive macro enables automatic implementation of the necessary accessor traits using `#[cgp_auto_getter]`. In addition, we derive `Deserialize` so that `FullAppBuilder` can be easily loaded from a configuration file in formats such as JSON or TOML.

Next, we wire up the builder context using the `delegate_components!` macro:

```rust
delegate_components! {
    FullAppBuilderComponents {
        ErrorTypeProviderComponent:
            UseAnyhowError,
        ErrorRaiserComponent:
            RaiseAnyhowError,
        HandlerComponent:
            BuildAndMergeOutputs<
                App,
                Product![
                    BuildSqliteClient,
                    BuildHttpClient,
                    BuildOpenAiClient,
                ]>,
    }
}
```

This macro allows us to delegate the implementation of various components of the builder context. First, we configure error handling by using the `cgp-anyhow-error` library. The `UseAnyhowError` provider specifies that our abstract `Error` type will be instantiated to `anyhow::Error`, and the `RaiseAnyhowError` provider allows conversion from error types implementing `core::error::Error`, like `sqlx::Error` and `reqwest::Error`, into `anyhow::Error`.

## Builder Dispatcher

In the example above, we used a special **builder dispatcher** called `BuildAndMergeOutputs` to implement the `HandlerComponent`. This dispatcher allows us to construct the final `App` type by sequentially combining the outputs of multiple builder providers. We specify the target `App` type as the output of the build process, and then pass in a _type-level list_ of builder providers using the `Product!` macro. In this case, we used `BuildSqliteClient`, `BuildHttpClient`, and `BuildOpenAiClient`, all of which we implemented previously.

To understand how `BuildAndMergeOutputs` operates under the hood, let us walk through a manual implementation that performs the same task:

```rust
#[cgp_new_provider]
impl<Code: Send, Input: Send> Handler<FullAppBuilder, Code, Input> for BuildApp {
    type Output = App;

    async fn handle(context: &FullAppBuilder, code: PhantomData<Code>, _input: Input) -> Result<App, Error> {
        let app = App::builder()
            .build_from(BuildSqliteClient::handle(context, code, ()).await?)
            .build_from(BuildHttpClient::handle(context, code, ()).await?)
            .build_from(BuildOpenAiClient::handle(context, code, ()).await?)
            .finalize_build();

        Ok(app)
    }
}
```

This manual implementation demonstrates the boilerplate that would be necessary if we did not use `BuildAndMergeOutputs`. Here, we define `BuildApp` as a _context-specific_ provider for the `FullAppBuilder` context. It implements the `Handler` trait for any `Code` and `Input` types.

Within the `handle` method, we construct the `App` in a step-by-step manner, similar to how we built complex types earlier in the [safe struct building](#safe-struct-building) section. We begin by initializing an empty builder with `App::builder()`. Next, we invoke the `handle` method on each of the individual providers — `BuildSqliteClient`, `BuildHttpClient`, and `BuildOpenAiClient` — passing them the shared context and `PhantomData` for the code. The resulting outputs are incrementally merged into the builder using `build_from`, and finally, `finalize_build` is called to produce the completed `App` instance.

In this example, we ignore the original `Input` parameter and instead pass `()` to each sub-handler for simplicity. In the actual implementation of `BuildAndMergeOutputs`, a *reference* to the intermediate builder is instead passed along as input to each sub-handler to support more advanced use cases. However, we have omitted that detail here to focus on the overall structure.

While the manual implementation of `BuildApp` is relatively easy to follow, it is also quite repetitive. The main benefit of `BuildAndMergeOutputs` is that it eliminates this boilerplate by abstracting away the repetitive logic of chaining multiple builder steps and threading intermediary results. Furthermore, `BuildAndMergeOutputs` is implemented with the necessary generic parameters and constraints to work with _any_ context type, as compared to being tied to the `App` context that we defined.

Aside from this reduction in verbosity, the behavior remains conceptually the same as what is shown in the manual example.

## Building the App

With the builder context defined, we can now construct the full `App` by simply instantiating the builder and calling its `handle` method:

```rust
async fn main() -> Result<(), Error> {
    let builder = FullAppBuilder {
        db_options: "file:./db.sqlite".to_owned(),
        db_journal_mode: "WAL".to_owned(),
        http_user_agent: "SUPER_AI_AGENT".to_owned(),
        open_ai_key: "1234567890".to_owned(),
        open_ai_model: "gpt-4o".to_owned(),
        llm_preamble: "You are a helpful assistant".to_owned(),
    };

    let app = builder.handle(PhantomData::<()>, ()).await?;

    /* Call methods on the app here */

    Ok(())
}
```

In this example, we initialize `FullAppBuilder` by filling in the required configuration values. We then call `builder.handle()` to construct the `App`. The `handle` method requires two arguments: a `Code` type and an `Input` value. However, because neither of these are constrained in any way in our example, we can simply pass *any* type we want, such as the unit type `()` for both. This simplifies to the equivalent of calling `builder.handle()` with no argument in practice.

This example illustrates how CGP allows new builder contexts to be defined with minimal effort by composing multiple independent builder providers — none of which require knowledge of the final type being constructed.

Rather than writing custom constructor functions that take numerous arguments, we define a builder struct where each required input becomes a field. Instead of manually constructing each component of the context, we use `delegate_components!` to connect the appropriate builder providers, which handle the construction logic for us.

By embracing this modular builder approach, our code becomes not only more extensible, but also easier to read, test, and maintain.

# More Builder Examples

At this point, some readers may still be skeptical about the value of modularity offered by CGP builders. Since we’ve only shown a **single** application context and **one** corresponding builder context so far, it might not be obvious why we couldn’t just use a simple `new` constructor function like the one defined at the beginning.

To truly demonstrate the power of modular builders, it’s helpful to explore how CGP makes it easy to define **multiple** contexts that are similar but have slight differences. However, if you're an **advanced reader** already familiar with the benefits of modular design, feel free to [**skip ahead**](#conclusion) to the conclusion.

## Default Builder

Earlier, we introduced default builders like `BuildDefaultSqliteClient`, which can construct an `App` with default configuration values. These defaults can be combined to define a minimal builder for `App`:

```rust
#[cgp_context]
#[derive(HasField, Deserialize)]
pub struct DefaultAppBuilder {
    pub db_path: String,
}

delegate_components! {
    DefaultAppBuilderComponents {
        ...
        HandlerComponent:
            BuildAndMergeOutputs<
                App,
                Product![
                    BuildDefaultSqliteClient,
                    BuildDefaultHttpClient,
                    BuildDefaultOpenAiClient,
                ]>,
    }
}
```

In this context, the only required configuration is the `db_path`, simplifying the process of constructing an `App`, especially for use cases like unit testing or demos.

## Postgres App

Now suppose we want an enterprise version of the app that uses **Postgres** instead of SQLite. We can define a new `App` context that swaps in `PgPool`:

```rust
#[cgp_context]
#[derive(HasField, HasFields, BuildField)]
pub struct App {
    pub postgres_pool: PgPool,
    pub http_client: Client,
    pub open_ai_client: openai::Client,
    pub open_ai_agent: Agent<openai::CompletionModel>,
}
```

Since the HTTP and ChatGPT logic remains unchanged, we only need to implement a new builder for Postgres:

```rust
#[cgp_new_provider]
impl<Build, Code: Send, Input: Send> Handler<Build, Code, Input> for BuildPostgresClient
where
    Build: HasPostgresUrl + CanRaiseAsyncError<sqlx::Error>,
{
    type Output = PostgresClient;

    async fn handle(
        build: &Build,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Self::Output, Build::Error> {
        let postgres_pool = PgPool::connect(build.postgres_url())
            .await
            .map_err(Build::raise_error)?;

        Ok(PostgresClient { postgres_pool })
    }
}

#[cgp_auto_getter]
pub trait HasPostgresUrl {
    fn postgres_url(&self) -> &str;
}

#[derive(HasField, HasFields, BuildField)]
pub struct PostgresClient {
    pub postgres_pool: PgPool,
}
```

This builder closely mirrors the SQLite version, but reads the `postgres_url` field from the context instead.

Next, we define a new builder context that includes Postgres configuration:

```rust
#[cgp_context]
#[derive(HasField, Deserialize)]
pub struct AppBuilder {
    pub postgres_url: String,
    pub http_user_agent: String,
    pub open_ai_key: String,
    pub open_ai_model: String,
    pub llm_preamble: String,
}

delegate_components! {
    AppBuilderComponents {
        ...
        HandlerComponent:
            BuildAndMergeOutputs<
                App,
                Product![
                    BuildPostgresClient,
                    BuildHttpClient,
                    BuildOpenAiClient,
                ]>,
    }
}
```

Here, we simply swap in `BuildPostgresClient` instead of `BuildSqliteClient`, while reusing the other builder providers unchanged.

This example highlights a key advantage of CGP over traditional **feature flags**: with CGP, multiple application variants (e.g., SQLite or Postgres) can **coexist** in the same codebase and even be compiled together. In contrast, feature flags often force a binary either/or split at compile time.

By enabling different configurations to exist side-by-side, CGP improves testability and reduces the likelihood of missing edge cases caused by untested feature combinations.

## Anthropic App

Just as we swapped SQLite for Postgres earlier, we can also substitute the AI model used in the application — such as replacing ChatGPT with **Claude**. With CGP, this becomes straightforward: we simply define a new `AnthropicApp` that uses the Anthropic client and agent:

```rust
#[cgp_context]
#[derive(HasField, HasFields, BuildField)]
pub struct AnthropicApp {
    pub sqlite_pool: SqlitePool,
    pub http_client: Client,
    pub anthropic_client: anthropic::Client,
    pub anthropic_agent: Agent<anthropic::completion::CompletionModel>,
}
```

Next, we implement a builder provider to construct the Claude client:

```rust
#[cgp_new_provider]
impl<Build, Code: Send, Input: Send> Handler<Build, Code, Input> for BuildDefaultAnthropicClient
where
    Build: HasAnthropicConfig + HasAsyncErrorType,
{
    type Output = AnthropicClient;

    async fn handle(
        build: &Build,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<Self::Output, Build::Error> {
        let anthropic_client = ClientBuilder::new(build.anthropic_key())
            .anthropic_version(ANTHROPIC_VERSION_LATEST)
            .build();

        let anthropic_agent = anthropic_client
            .agent(anthropic::CLAUDE_3_7_SONNET)
            .preamble(build.llm_preamble())
            .build();

        Ok(AnthropicClient {
            anthropic_client,
            anthropic_agent,
        })
    }
}

#[cgp_auto_getter]
pub trait HasAnthropicConfig {
    fn anthropic_key(&self) -> &str;
    fn llm_preamble(&self) -> &str;
}

#[derive(HasField, HasFields, BuildField)]
pub struct AnthropicClient {
    pub anthropic_client: anthropic::Client,
    pub anthropic_agent: Agent<CompletionModel>,
}
```

With the builder provider in place, we define a new builder context that includes the Anthropic API key and wire it up using `BuildDefaultAnthropicClient`:

```rust
#[cgp_context]
#[derive(HasField, Deserialize)]
pub struct AppBuilder {
    pub db_options: String,
    pub db_journal_mode: String,
    pub http_user_agent: String,
    pub anthropic_key: String,
    pub llm_preamble: String,
}

delegate_components! {
    AppBuilderComponents {
        ...
        HandlerComponent:
            BuildAndMergeOutputs<
                AnthropicApp,
                Product![
                    BuildSqliteClient,
                    BuildHttpClient,
                    BuildDefaultAnthropicClient,
                ]>,
    }
}
```

This example shows how effortlessly CGP supports variation and customization. The same modular pattern can be reused to swap in different components — databases, HTTP clients, or agents — without rewriting core application logic.

In fact, the process becomes so systematic that it’s easy to imagine an AI tool like **Claude Code** automating the entire setup given the right prompt and documentation.

## Anthropic and ChatGPT Builder

It’s impressive that CGP lets us easily swap ChatGPT for Claude. But what’s even better is that we don’t have to choose at all — we can include **both** AI agents in the same application.

This could be useful for scenarios where combining the strengths of multiple models improves the overall intelligence or reliability of your application. More importantly, it demonstrates that CGP is not just about selecting one provider over another — it’s also about composing multiple providers together in a clean, modular way.

We begin by defining an `AnthropicAndChatGptApp` context that includes both Claude and ChatGPT clients:

```rust
#[cgp_context]
#[derive(HasField, HasFields, BuildField)]
pub struct AnthropicAndChatGptApp {
    pub sqlite_pool: SqlitePool,
    pub http_client: Client,
    pub anthropic_client: anthropic::Client,
    pub anthropic_agent: Agent<anthropic::completion::CompletionModel>,
    pub open_ai_client: openai::Client,
    pub open_ai_agent: Agent<openai::CompletionModel>,
}
```

Next, we define a builder context that includes configuration fields for both AI platforms:

```rust
#[cgp_context]
#[derive(HasField, Deserialize)]
pub struct AnthropicAndChatGptAppBuilder {
    pub db_options: String,
    pub db_journal_mode: String,
    pub http_user_agent: String,
    pub anthropic_key: String,
    pub open_ai_key: String,
    pub open_ai_model: String,
    pub llm_preamble: String,
}
```

In the component wiring, we include both `BuildDefaultAnthropicClient` and `BuildOpenAiClient` in the provider list:

```rust
delegate_components! {
    AnthropicAndChatGptAppBuilderComponents {
        ...
        HandlerComponent:
            BuildAndMergeOutputs<
                AnthropicAndChatGptApp,
                Product![
                    BuildSqliteClient,
                    BuildHttpClient,
                    BuildDefaultAnthropicClient,
                    BuildOpenAiClient,
                ]>,
    }
}
```

With just a few extra lines, we’ve created a **dual-agent** AI app that can leverage both Claude and ChatGPT simultaneously.

It’s also worth noting that the `llm_preamble` field is reused by both the Claude and ChatGPT builders. This demonstrates CGP’s flexibility in sharing input values across multiple providers—without requiring any manual coordination or boilerplate.

This kind of seamless reuse and composition is where CGP truly shines: giving you fine-grained control over how your application is assembled, while keeping your code modular and maintainable.

## Multi-Context Builder

Looking closely at the `AnthropicAndChatGptAppBuilder` that we previously defined, we can observe that it already includes all the necessary fields required to construct the Claude-only and ChatGPT-only applications as well. This means we can reuse the same builder to construct **all three** versions of our application contexts, simply by changing how the builder is wired.

To achieve this, we take advantage of the `Code` type parameter, which allows us to emulate DSL-like behavior similar to what is seen in [Hypershell](/blog/hypershell-release/#abstract-syntax). We begin by defining distinct marker types that represent the different build modes:

```rust
pub struct BuildChatGptApp;
pub struct BuildAnthropicApp;
pub struct BuildAnthropicAndChatGptApp;
```

Using these types, we can apply the [`UseDelegate`](/blog/hypershell-release/#generic-dispatcher) pattern to route the `Handler` implementation to different builder pipelines depending on the code passed in. This enables conditional wiring based on the selected application mode:

```rust
delegate_components! {
    AnthropicAndChatGptAppBuilderComponents {
        ...
        HandlerComponent:
            UseDelegate<new BuilderHandlers {
                BuildAnthropicAndChatGptApp:
                    BuildAndMergeOutputs<
                        AnthropicAndChatGptApp,
                        Product![
                            BuildSqliteClient,
                            BuildHttpClient,
                            BuildDefaultAnthropicClient,
                            BuildOpenAiClient,
                        ]>,
                BuildChatGptApp:
                    BuildAndMergeOutputs<
                        ChatGptApp,
                        Product![
                            BuildSqliteClient,
                            BuildHttpClient,
                            BuildOpenAiClient,
                        ]>,
                BuildAnthropicApp:
                    BuildAndMergeOutputs<
                        AnthropicApp,
                        Product![
                            BuildSqliteClient,
                            BuildHttpClient,
                            BuildDefaultAnthropicClient,
                        ]>,
            }>,
    }
}
```

Now, when we want to construct a specific application context, we only need to change the `Code` type by using `PhantomData`. This gives us a flexible, type-safe way to select the desired builder pipeline at runtime:

```rust
pub async fn main() -> Result<(), Error> {
    let builder = AnthropicAndChatGptAppBuilder {
        db_options: "file:./db.sqlite".to_owned(),
        db_journal_mode: "WAL".to_owned(),
        http_user_agent: "SUPER_AI_AGENT".to_owned(),
        anthropic_key: "1234567890".to_owned(),
        open_ai_key: "1234567890".to_owned(),
        open_ai_model: "gpt-4o".to_owned(),
        llm_preamble: "You are a helpful assistant".to_owned(),
    };

    let chat_gpt_app: ChatGptApp =
        builder.handle(PhantomData::<BuildChatGptApp>, ()).await?;

    let anthropic_app: AnthropicApp =
        builder.handle(PhantomData::<BuildAnthropicApp>, ()).await?;

    let combined_app: AnthropicAndChatGptApp =
        builder.handle(PhantomData::<BuildAnthropicAndChatGptApp>, ()).await?;

    /* Use the application contexts here */

    Ok(())
}
```

This example highlights how CGP's DSL features are not limited to building full-fledged domain-specific languages like [Hypershell](/blog/hypershell-release/). Even in this lightweight form, they are immensely valuable for **labeling and routing** different behaviors based on combinations of builder providers.

In essence, we are still constructing a mini-DSL, albeit one composed of simple symbolic "statements" without complex language constructs. This approach not only brings expressive power to your builder logic, but also lays the groundwork for future extensions — such as richer abstract syntaxes — using the same techniques introduced by Hypershell.

# Conclusion

In this first installment, we explored how CGP v0.4.2 empowers Rust developers to construct application contexts using modular, extensible builders. You’ve seen how individual providers like `BuildSqliteClient`, `BuildHttpClient`, and `BuildOpenAiClient` can be composed to build complex types without tight coupling or boilerplate. We’ve also demonstrated how the same context can be reused across multiple application variants — from SQLite to Postgres, from ChatGPT to Claude — all through declarative builder composition.

This approach dramatically simplifies configuration management, promotes code reuse, and opens the door to highly flexible, plugin-style architectures in Rust. Whether you're building minimal test contexts or full-featured production systems, CGP gives you the tools to scale your logic modularly and safely.

In [Part 2 of this series, **Modular Interpreters and Extensible Visitors**](/blog/extensible-datatypes-part-2/), we’ll shift gears to look at **extensible variants**, where CGP tackles the expression problem with a modular visitor pattern. If you've ever wanted to define interpreters, pattern match over generic enums, or evolve your data types without breaking existing logic — you won’t want to miss what’s coming next.
