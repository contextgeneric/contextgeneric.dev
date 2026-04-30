---
authors: [soares]
---

# Using an incoherent, dictionary-passing style Rust today

In the past few weeks, the Rust community has had some interesting discussion around Rust traits and coherence. Boxy has written a blog post about [**An Incoherent Rust**](https://www.boxyuwu.blog/posts/an-incoherent-rust/), which explores how it would look if we could write incoherent trait implementations in Rust. The discussion was sparked by Nadri's earlier blog posts on [**Elaborating Rust Traits to Dictionary-Passing Style**](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html) and [**What If Traits Carried Values**](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html), and it reignited some older discussions about supporting [**context and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) in Rust.

In this blog post, I will explain how **Context-Generic Programming** (CGP) relates to these recent discussions, and whether there is common ground we can draw from CGP to help implement dictionary-passing style or incoherence in Rust today.

{/* truncate */}

## Outline

This post is organized as follows. We first introduce CGP's approach to incoherence, then show how several features of Incoherent Rust desugar into CGP code, and examine how CGP's context type corresponds to the dictionaries described in the dictionary-passing style work. We then discuss what this desugaring reveals about the challenges of implementing dictionary-passing style in Rust at the compiler level, and where CGP's model diverges from the full vision of Incoherent Rust.

In summary, CGP:

- Provides a working implementation strategy for a significant subset of Incoherent Rust today.
- Offers a natural desugaring path for context and capabilities.
- Does not solve the formalization goal or ecosystem evolution for existing traits.
- Reveals specific challenges that full dictionary-passing style in Rust would need to address.

---

## Background and Context

This section covers some background on the other blog posts, based on how I understand the situation through publicly available information. I will likely miss some critical details that would give a more complete picture. But I want to lay out what I know here, so that if there is any misunderstanding, you will know it could be due to gaps in my background.

The public discussion started when Nadri shared his blog post on [**elaborating Rust traits to dictionary-passing style**](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html). This appears to be part of the [**Rust via Desugarings**](https://nadrieril.github.io/rust-via-desugarings/) effort to desugar Rust code into a simpler intermediate language that makes it easier to formalize and improve the soundness of Rust.

For the purpose of this blog post, I will refer to this hypothetical intermediate representation as **Traitless IR (TIR)**, since it contains a subset of Rust without the trait system. Under TIR, all original use of Rust traits would be desugared to use techniques such as dictionary-passing style as the underlying implementation.

:::note
Similar to other Rust IRs like HIR, THIR, and MIR, Rust programmers won't be writing TIR programs directly. Technically, there won't be any concrete syntax for TIR, but we will use a Rust-like syntax for TIR to  better illustrate the concepts and facilitate the discussion.
:::

Following the original blog post, Nadri also wrote another post on [**what if traits carried values**](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html), exploring what new capabilities could be built on the trait system if Rust implemented traits by desugaring them into TIR. Most notably, Nadri explained that this would allow traits to carry values through the desugared dictionaries, which in turn would make it much easier to implement Tyler's ideas of [**context and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) in Rust.

The discussion was then followed by Boxy's blog post on [**an Incoherent Rust**](https://www.boxyuwu.blog/posts/an-incoherent-rust/), which explores how dictionary-passing style could also enable **incoherent traits** that are not subject to the usual coherence restrictions in Rust.

For the purpose of this blog post, I will refer to this envisioned future version of Rust as **Incoherent Rust**, since it is a superset of today's Rust that adds support for incoherent traits. The specific features in Incoherent Rust are not yet settled, but I will use the term broadly to cover all the language features explored by Nadri, Boxy, and Tyler.

There are multiple objectives being explored across these discussions. The original motivation for TIR was to formalize and improve the correctness of the Rust compiler. It was then discovered that using dictionary-passing style in TIR could also make it easier to implement higher level features such as Incoherent Rust and context and capabilities.

That said, these two objectives are not necessarily tied together. For example, it might be possible to implement TIR without using it to add new features like Incoherent Rust. Similarly, dictionary-passing style and TIR are not the only implementation strategies available to enable features like Incoherent Rust.

In this blog post, I am analyzing the two objectives as separate cases, so that the feasibility of one does not affect how we think about the other.

---

## A Brief Introduction to CGP

This section gives a brief introduction to CGP, written with Rust compiler team members in mind. The target audience is assumed to already be deeply familiar with advanced Rust concepts like traits, generics, and the coherence rules.

CGP is a modular programming paradigm for Rust that enables developers to write highly modular code that is generic over a context type, which corresponds to the `Self` type in trait definitions. The core idea is to develop design patterns that work within Rust's coherence restrictions, while still making it possible to write multiple overlapping trait implementations that are generic over the context type.

To understand the core concepts in CGP, I recommend starting with my RustLab 2025 presentation titled [**How to stop fighting with coherence and start writing context-generic trait impls**](/blog/rustlab-2025-coherence). The [**area calculation tutorial**](/docs/tutorials/area-calculation/) then walks through some simplified examples of the design patterns that CGP enables. If you want to dig into specific internals, the [**CGP Skills**](/docs/ai/skills/) document can help you explore those questions by attaching the skills to an LLM.

With that background in place, the rest of this section explains how CGP approaches incoherence in practice.

### Bypassing overlapping implementations

Before going into the details, it is worth noting that CGP supports two levels of design patterns to bypass coherence restrictions.

The first level makes it possible to define multiple overlapping implementations of a trait, by introducing a layer of indirection through a provider trait. For example, the [`Hash` trait](https://doc.rust-lang.org/std/hash/trait.Hash.html) could be redefined with CGP as follows:

```rust
#[cgp_component(HashImpl)]
pub trait Hash {
    fn hash<H: Hasher>(&self, state: &mut H);
}
```

Compared to the original definition, the main difference is the `#[cgp_component]` macro, which generates additional constructs that support overlapping implementations through a provider trait called `HashImpl`. Using that, we can define an implementation that hashes using `Display`:

```rust
#[cgp_impl(new HashWithDisplay)]
impl HashImpl
where
    Self: Display,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state)
    }
}
```

The `#[cgp_impl]` macro implements the `HashImpl` provider trait by creating a named provider called `HashWithDisplay`. It is implemented generically for any context that implements `Display`. Behind the scenes, the provider trait `HashImpl` uses `HashWithDisplay` as the actual `Self` type, which is what allows the code to define the implementation without conflicting with other overlapping implementations.

Making this work on a concrete context requires an additional wiring step through `delegate_components!`. For example:

```rust
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} {}", self.first_name, self.last_name)
    }
}

delegate_components! {
    Person {
        HashImplComponent: HashWithDisplay,
    }
}
```

Behind the scenes, the `delegate_components!` macro translates the code into a type-level lookup table through the `DelegateComponent` trait. CGP then uses blanket implementations generated by `#[cgp_component]` to perform a lookup at compile time and implement `Hash` for `Person` using `HashWithDisplay`.

This form of coherence bypass is useful for defining any number of overlapping implementations that can be selectively reused by specific contexts. For example, the same pattern applies to the `Serialize` trait from `serde`:

```rust
#[cgp_component(SerializeImpl)]
pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

#[cgp_impl(new SerializeWithDisplay)]
impl SerializeImpl
where
    Self: Display,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

delegate_components! {
    Person {
        SerializeImpl: SerializeWithDisplay,
    }
}
```

As the example shows, CGP makes it straightforward for a type like `Person` to reuse existing provider implementations for both `Hash` and `Serialize`, with just one line of mapping per trait. Although the example itself is simple enough to implement manually, the same approach scales well to more complex real-world use cases where providers are shared across many types.

### Bypassing orphan implementations

The base CGP techniques described above allow bypassing of overlapping implementations, but they do not fully solve the orphan rule problem. To implement traits like `Hash` and `Serialize`, the `delegate_components!` call must still reside in a crate that owns either `Person` or the trait.

In other words, this alone does not fully solve the ecosystem evolution problem that [**Incoherent Rust**](https://www.boxyuwu.blog/posts/an-incoherent-rust/) is trying to address. However, as explained in my [**RustLab presentation**](/blog/rustlab-2025-coherence), CGP offers a more advanced design pattern that addresses both overlapping and orphan implementations at the same time. This is done by adding an additional type parameter as the main context type, and shifting the original `Self` type into a generic parameter of the consumer trait.

For example, a fully incoherent version of `Serialize` would be defined as follows:

```rust
#[cgp_component(SerializeImpl)]
pub trait Serialize<T> {
    fn serialize<S: Serializer>(
        &self,
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}
```

Compared to the earlier definition, this version uses an additional `Context` type as `Self`, and shifts the original `Self` type to the generic `T` parameter. This version of `Serialize` makes it possible to wire up implementations through a centralized context type that does not need to reside in the same crate as the value type or the trait. For example:

```rust
// inside a third party crate
pub struct MyApp;

delegate_components! {
    MyApp {
        // Specialize SerializeImplComponent based on the generic parameters
        open SerializeImplComponent;

        // New path-based notation to specify the generic parameters as
        // part of the component path.
        @SerializeImplComponent.Person: SerializeWithDisplay,
    }
}
```

:::note
I am using a new improved syntax that will be available in the upcoming version of CGP. The current version of CGP makes use of `UseDelegate` to perform nested table lookup, which is syntactically less readable.
:::

With this approach, third party crates can define their own contexts like `MyApp`, and locally rewire the `Serialize` implementation for a value type without affecting global coherence.

#### Does CGP truly solve the orphan rules problem?

CGP manages to bypass Rust's coherence restriction by adding an extra context type parameter at the `Self` position. This approach only partially solves the orphan rules problem, because it requires all involved traits to adopt CGP and include the extra context parameter in their definition.

This means it is not possible to apply CGP's approach to existing traits like `serde::Serialize` without introducing breaking changes across the entire Rust ecosystem. However, CGP's approach is well suited for defining new traits, since the generic context type can be added from the start without breaking anything.

### CGP as a library vs as a syntactic sugar for Rust

CGP is currently implemented as a library on top of stable Rust. The core concepts behind it are a small set of design patterns, all of which are documented in the book [**Context-Generic Programming Patterns**](https://patterns.contextgeneric.dev/). These patterns are deliberately minimal, and they work within the existing rules of the Rust type system rather than requiring any changes to it.

When I describe a strategy for desugaring Incoherent Rust into CGP, I am not suggesting that Rust should adopt CGP as a library dependency. That would be the wrong framing. What I am proposing instead is that Rust could adopt the underlying design patterns that CGP has developed, and promote them into first-class language features. Those features could then serve as the internal mechanism for implementing the syntactic sugar that Incoherent Rust would expose to programmers.

Only a small number of concrete constructs would need to find a home in the language or standard library. For example, the `DelegateComponent` trait provides the type-level lookup table that the rest of the system depends on. These constructs are intentionally simple, and adding it to the standard library would not introduce meaningful complexity to the language.

The larger portion of the CGP library consists of procedural macros. These macros handle the ergonomics of writing CGP code by performing syntactic transformations, such as moving the `Context` type into the provider trait, generating blanket implementations, and producing component name structs. If Rust were to adopt these features natively, the compiler would simply perform the same transformations that the proc macros perform today. The desugaring logic would move from a library crate into the compiler itself, but the underlying rules would remain the same.

The practical consequence of this approach is that Incoherent Rust would become the surface syntax that programmers interact with, while CGP would recede into the implementation details of the compiler. Rust programmers would learn about incoherent traits, named implementations, and context-and-capability bindings as language features, with no need to know that CGP patterns are what makes them work under the hood. The library would have served its purpose by proving out the design, and the language would carry it forward.

---

## Implementing incoherence with CGP

With the basic CGP concepts introduced, we can now look at how we could implement Incoherent Rust by desugaring it into CGP code.

The main reason to explore this desugaring is that CGP is implemented as a library on top of stable Rust. This means that any CGP code is already supported by the language today. If we can desugar the new Rust features proposed by Boxy and Nadri into CGP, we could effectively implement those features as syntactic sugar without needing to change the core Rust compiler. This could make it significantly easier for Rust to ship these features in the near future.

### Incoherent traits

In [An Incoherent Rust](https://www.boxyuwu.blog/posts/an-incoherent-rust/#incoherent-traits), Boxy proposes introducing an `incoherent` keyword on traits to allow overlapping implementations. For example:

```rust title="Incoherent Rust"
// crate a
pub incoherent trait Serialize {
    fn serialize(&self) -> String;
}
```

With CGP, an incoherent trait like this can be desugared by introducing a generic `Context` parameter and using it as the `Self` type of the provider trait:

```rust title="CGP"
// crate a
#[cgp_component(SerializeImpl)]
pub trait Serialize<T> {
    fn serialize(&self, value: &T) -> String;
}
```

The original `Self` type, which represents the value being serialized, becomes the explicit generic parameter `T`. The new `Self` position is now the context type, which acts as the carrier for implementation choices. This is what allows multiple overlapping implementations to coexist without violating coherence.

### Named impls

In the proposed Incoherent Rust syntax, each implementation is given a name. For example:

```rust title="Incoherent Rust"
// crate b
pub struct Matrix(...)

// crate c
impl CSerialize = a::Serialize for b::Matrix { ... }

// crate d
impl DSerialize = a::Serialize for b::Matrix { ... }
```

With CGP, each named impl becomes a provider struct with an implementation of the corresponding provider trait. The two named implementations above would desugar to:

```rust title="CGP"
// crate c
#[cgp_impl(new CSerialize)]
impl a::SerializeImpl<b::Matrix> {
    ...
}

// crate d
#[cgp_impl(new DSerialize)]
impl a::SerializeImpl<b::Matrix> {
    ...
}
```

Because the `Self` type of each provider trait implementation is the provider struct itself, which is defined in the local crate, there is no coherence conflict even when both implementations target the same value type `b::Matrix`.

### Using incoherent trait bounds

We can also desugar the use of incoherent trait bounds into CGP. Consider the following example from the original blog post, which demonstrates calling the same function with two different implementations of the same trait:

```rust title="Incoherent Rust"
incoherent trait Name {
    const NAME: &'static str;
}

impl DummyName<T> = Name for T {
    const NAME: &'static str = "dummy";
}

impl TypeName<T> = Name for T {
    const NAME: &'static str = core::any::type_name::<T>();
}

struct Foo<T>(T);

impl MyImpl<T: Name> = Foo<T> {
    pub fn do_stuff(&self) {
        println!("{}", <T as Name>::NAME);
    }
}

fn main() {
    let foo = Foo(1);

    // prints "dummy"
    MyImpl<_ + DummyName<_>>::do_stuff(&foo);

    // prints "i32"
    MyImpl<_ + TypeName<_>>::do_stuff(&foo);
}
```

The same code can be desugared into CGP as follows:

```rust title="CGP"
#[cgp_component(NameImpl)]
pub trait Name<T> {
    const NAME: &'static str;
}

#[cgp_impl(new DummyName)]
impl<T> NameImpl<T> {
    const NAME: &'static str = "dummy";
}

#[cgp_impl(new TypeName)]
impl<T> NameImpl<T> {
    const NAME: &'static str = core::any::type_name::<T>();
}

pub struct Foo<T>(pub T);

#[cgp_fn]
#[uses(Name<T>)]
pub fn do_stuff<T>(&self, foo: &Foo<T>) {
    println!("{}", Self::NAME);
}

fn main() {
    let foo = Foo(1);

    {
        struct Context;

        delegate_components! {
            Context {
                NameImplComponent: DummyName,
            }
        }

        // prints "dummy"
        Context.do_stuff(&foo);
    }

    {
        struct Context;

        delegate_components! {
            Context {
                NameImplComponent: TypeName,
            }
        }

        // prints "i32"
        Context.do_stuff(&foo);
    }
}
```

A few things are worth noting about the desugared code. The incoherent `Name` trait becomes a CGP component `Name<T>` with a corresponding provider trait `NameImpl`. The generic function `do_stuff` becomes a context-generic CGP function that uses `Name<T>` through the `#[uses]` attribute and accepts `&self` as the implicit context parameter. Each anonymous call site in the original code, such as `MyImpl<_ + DummyName<_>>`, desugars into a locally scoped `Context` struct with a `delegate_components!` block that wires the chosen provider.

The reason `do_stuff` accepts a `&self` argument is that the `Self` context type is what carries the implementation choice at the type level. As we will see later, this `&self` can also carry runtime values, which is what makes context-generic functions suitable for implementing context and capabilities.

### Use of incoherent trait bounds within a struct

Boxy's blog post also discusses binding an incoherent trait implementation directly to a struct, so that the same implementation is always used whenever that struct is passed around:

```rust title="Incoherent Rust"
struct Foo<T: Name>(T);

impl MyImpl<T: Name> = Foo<T> {
    ...
}

fn main() {
    let foo = Foo::<_ + DummyName<_>>(1)

    // prints "dummy"
    MyImpl<_ + DummyName<_>>::do_stuff(&foo);

    // errors as `foo` has type `Foo<u8 + DummyName<u8>>` but
    // the impl requires the self type to be `Foo<u8 + TypeName<u8>>`
    MyImpl<_ + TypeName<_>>::do_stuff(&foo);
}
```

With CGP, this pattern desugars by adding the `Context` type as a parameter on the struct itself, so that the struct carries the implementation choice in its type:

```rust title="CGP"
pub struct Foo<Context, T>
where
    Context: Name<T>,
{
    pub value: T,
    pub phantom: PhantomData<Context>,
}

#[cgp_fn]
#[uses(Name<T>)]
pub fn do_stuff<T>(&self, foo: &Foo<Self, T>) {
    println!("{}", Self::NAME);
}

struct ContextA;

delegate_components! {
    ContextA {
        NameImplComponent: DummyName,
    }
}

struct ContextB;

delegate_components! {
    ContextB {
        NameImplComponent: TypeName,
    }
}

fn main() {
    let foo: Foo<ContextA, i32> = Foo {
        value: 1,
        phantom: PhantomData,
    };

    ContextA.do_stuff(&foo);

    // type error: foo has type Foo<ContextA, i32> but ContextB expects Foo<ContextB, i32>
    ContextB.do_stuff(&foo);
}
```

Because the `Context` type is part of the struct's type signature, a value of type `Foo<ContextA, i32>` and a value of type `Foo<ContextB, i32>` are distinct types. This is exactly the property Boxy describes: once a struct is created with a particular incoherent implementation bound to it, that binding cannot be silently swapped out at another call site.

This property is especially useful for solving the hash table problem in the presence of incoherence. A hash map can include the `Context` type in its definition, becoming `HashMap<Context, Key, Value>`, to ensure that the same `Hash` implementation is always used when inserting or looking up entries.

### Impl-specific bindings within a struct

One downside of binding the full `Context` type to a struct is that two contexts with identical provider wirings are still considered different types. This can introduce friction when you want to share a struct value across modules or functions that use different context types.

An alternative is to bind a specific provider type to the struct rather than a full context. This is possible in CGP using higher-order providers, where the provider type itself is a generic parameter of the struct:

```rust title="CGP"
pub struct Foo<NameProvider, T> {
    pub value: T,
    pub phantom: PhantomData<NameProvider>,
}

#[cgp_fn]
#[use_provider(NameProvider: NameImpl<T>)]
pub fn do_stuff<NameProvider, T>(&self, foo: &Foo<NameProvider, T>) {
    println!("{}", NameProvider::NAME);
}

fn main() {
    pub struct ContextA;
    pub struct ContextB;

    let foo: Foo<TypeName, i32> = Foo {
        value: 1,
        phantom: PhantomData,
    };

    // Both contexts can call do_stuff on the same foo value,
    // because the Name implementation is fixed by TypeName, not by the context.
    ContextA.do_stuff(&foo);
    ContextB.do_stuff(&foo);
}
```

In this version, the struct carries the provider type directly, so the choice of `Name` implementation is locked to `TypeName` regardless of which context is used to call `do_stuff`. This is more flexible than the full-context binding, because any context can interact with the struct without needing to match the exact context type that created it.

The trade-off is that this approach only fixes the binding for one specific trait. If multiple incoherent traits are involved and you want each one to be bound independently, the struct would need a separate provider parameter for each trait. Depending on how many traits are involved, that can become verbose. The full-context approach described earlier handles this more uniformly, at the cost of tying the struct to a single concrete context type.

---

## Context as dictionary

If you have been following the desugared CGP code for Incoherent Rust in the previous sections, you may have noticed that the additional generic `Context` type introduced by CGP works in a similar way to the dictionaries described in Nadri's blog post on [**Elaborating Rust Traits to Dictionary-Passing Style**](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html).

That blog post explores the idea of lowering high-level Rust code into an intermediate representation that translates trait usage into dictionary-passing style in TIR. For example, given:

```rust title="Rust"
trait Clone {
    fn clone(&self) -> Self;
}

impl Clone for u32 { ... }
impl<T: Clone> Clone for Vec<T> { ... }

let my_vec: Vec<u32> = vec![0, 1, 2];
my_vec.clone(); // uses the two impls above together
```

The blog post shows that we can lower it into TIR:

```rust title="TIR"
struct Clone<Self> {
    clone: fn(&Self) -> Self,
}

const CLONE_U32: Clone<u32> = Clone {
    clone: |x: &u32| -> u32 { *x },
}

const CLONE_VEC<T, const CLONE_T: Clone<T>>: Clone<Vec<T>> = Clone {
    clone: |vec: &Vec<T>| -> Vec<T> {
        vec
            .iter()
            .map(|x: &T| CLONE_T.clone(x))
            .collect()
    }
}

let my_vec: Vec<u32> = vec![0, 1, 2];
(CLONE_VEC::<u32, CLONE_U32>).clone(&my_vec);
```

### Context-aware dictionaries

Suppose that we want to combine the earlier ideas and make `Clone` an incoherent trait. Nadri's blog post suggested a slightly different syntax for Incoherent Rust to express this:

```rust title="Incoherent Rust"
trait Clone {
    fn clone(&self) -> Self;
}

impl "clone_u32" Clone for u32 { ... }

impl<T> "clone_vec" Clone for Vec<T>
where
    clone_t: [T: Clone]
{
    fn clone(&self) -> Self {
        vec
            .iter()
            .map(|x: &T| clone_t::clone(x))
            .collect()
    }
}

let my_vec: Vec<u32> = vec![0, 1, 2];
clone_vec::<u32>[clone_u32]::clone(&my_vec);
```

As shown in the earlier sections, we can translate this to CGP as follows:

```rust title="CGP"
#[cgp_component(CloneImpl)]
trait Clone<T> {
    fn clone(&self, value: &T) -> T;
}

#[cgp_impl(new CloneU32)]
impl CloneImpl<u32> { ... }

#[cgp_impl(new CloneVec)]
#[uses(Clone<T>)]
impl<T> CloneImpl<Vec<T>> {
    fn clone(&self, values: &Vec<T>) -> Vec<T> {
        values
            .iter()
            .map(|value| self.clone(value))
            .collect()
    }
}

fn main() {
    struct Context;

    delegate_components! {
        Context {
            open CloneImplComponent;

            @CloneImplComponent.u32: CloneU32,
            @CloneImplComponent.Vec<u32>: CloneVec,
        }
    }

    let my_vec: Vec<u32> = vec![0, 1, 2];
    let cloned_vec = Context.clone(&my_vec);
}
```

If we were to lower the Rust code into TIR, we could produce the following translation, which sits much closer to both today's Rust and the CGP code above:

```rust title="TIR"
// A dictionary for `Clone` with an additional `Context` parameter
struct Clone<Context, T> {
    clone: fn(&Context, &T) -> T,
}

// A const function for building a `Clone` dictionary for `u32`
// with any `Context`
const fn clone_u32<Context>() -> Clone<Context, u32> {
    Clone {
        clone: |value| *value,
    }
}

// A const function for building a `Clone` dictionary for `Vec<T>`
// with any `Context`.
const fn clone_vec<Context, T>(
    // We require a getter function to get the `Clone` dictionary
    // of `T` from `Context`.
    get_clone_value: fn(Context) -> Clone<Context, T>,
) -> Clone<Context, Vec<T>> {
    Clone {
        clone: |context, values| {
            // The specific `Clone` dictionary for `T` is retrieved
            // at runtime from `context`. This means the implementation
            // can change based on where it is called.
            let clone_value = get_clone_value(context);

            values
                .iter()
                .map(|value| clone_value.clone(context, value))
                .collect()
        },
    }
}

// The compiler generates a `Context` struct that contains all
// dictionaries that are required by a particular Rust code.
struct Context {
    // All dictionaries have a recursive reference back to the
    // `Context` type.
    clone_u32: Clone<Context, u32>,
    clone_vec_u32: Clone<Context, Vec<u32>>,
}

// We can define a const dictionary of `Context` that
// is available at compile time.
const CONTEXT: Context = Context {
    clone_u32: clone_u32::<Context>::(),
    // The compiler needs to generate the getter functions
    // and pass them to dictionary constructors like `clone_vec`.
    clone_vec_u32: clone_vec::<Context, u32>::(
        |context| context.clone_u32,
    ),
}

fn main() {
    let my_vec: Vec<u32> = vec![0, 1, 2];

    // The `Context` type and the `CONTEXT` constant are generated
    // for this line of code to work.
    let cloned_vec = CONTEXT.clone_vec_u32.clone(CONTEXT, &my_vec);
}
```

Comparing the three versions side by side, the `Context` type in CGP acts as a type-level dictionary, where the mappings are declared through `delegate_components!`. When we lower this into TIR, that same `Context` type becomes a value-level dictionary that holds other dictionaries as fields. There is an almost one-to-one correspondence between a `delegate_components!` entry and a field in the TIR context struct, which shows that CGP already closely aligns with the programming model of dictionary-passing style.

Another important property of this translation is that all trait dictionaries accept both the `Context` type and a `context` value as additional parameters. This makes it straightforward to resolve cases where one trait implementation depends on other trait implementations, even when those dependencies form a complex graph with cycles. Because each implementation looks up other implementations through the context at runtime, the compiler does not need to determine a fixed order for instantiating dictionaries or passing them as arguments to other dictionaries. The context acts as a single stable root from which all dependencies can be reached.

This runtime lookup also opens the door to a form of dynamic scoping, where the implementation used for a trait can vary depending on which context value is in scope at the call site.

### Context and capabilities

Nadri's follow-up blog post, [**What If Traits Carried Values**](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html), explores how we could implement Tyler's [**contexts and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal using dictionary-passing style. The motivating example is passing a `BasicArena` value into an implementation of `Deserialize`:

```rust title="Incoherent Rust"
capability arena;

impl<'a> Deserialize for &'a Foo
where
    arena: &'a BasicArena,
{
    ...
}

fn main() -> Result<(), Error> {
    let bytes = read_some_bytes()?;
    with arena = &arena::BasicArena::new() {
        let foos: Vec<&Foo> = deserialize(bytes)?;
        println!("foos: {:?}", foos);
    }
    Ok(())
}
```

As explained in my `cgp-serde` blog post, CGP incoherent traits already support passing runtime values alongside trait implementations through the `Context` type. The recent introduction of **implicit arguments** in CGP makes accessing those field values considerably more ergonomic.

With that, the example above can be desugared into the following CGP code:

```rust title="CGP"
// The deserialize target is now `T` instead of `Self`.
#[cgp_component(DeserializeImpl)]
pub trait Deserialize<'de, T> {
    fn deserialize<D>(&self, deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>;
}

#[cgp_impl(new DeserializeFooRef)]
impl<'de, 'a, Value> ValueDeserializer<'de, &'a Foo> {
    fn deserialize<D>(
        &self,
        // Capabilities are "passed" as implicit arguments through the context.
        // The double reference (&&) arises because implicit arguments always
        // borrow from `&self`, and the capability type is itself already a reference.
        // The outer & comes from the borrow of self; the inner &'a comes from the
        // capability type being &'a BasicArena.
        #[implicit] arena: &&'a BasicArena,
        deserializer: D,
    ) -> Result<&'a Foo, D::Error>
    where
        D: serde::Deserializer<'de>,
    { ... }
}

fn main() -> Result<(), Error> {
    let bytes = read_some_bytes()?;

    // The `Context` type now contains fields instead of being an empty struct.
    #[derive(HasField)]
    struct Context<'a> {
        arena: &'a BasicArena,
    }

    delegate_components! {
        Context {
            open DeserializeImplComponent;

            @DeserializeImplComponent.&'a Foo:
                DeserializeFooRef,
            // A context-generic provider provided by cgp-serde to implement
            // `Deserialize` using the `Extend` method of the container.
            @DeserializeImplComponent.Vec<&'a Foo>:
                DeserializeExtend,
        }
    }

    let arena = arena::BasicArena::new();

    // The `with` bindings become field assignments to a local `context` value.
    let context = Context {
        arena: &arena,
    };

    // Use a helper function provided by cgp-serde to deserialize
    // the JSON content using `serde_json`. The `deserialize_json_bytes` method
    // is a convenience provided by the cgp-serde crate that wraps `serde_json`
    // deserialization in a context-generic way.
    let foos: Vec<&Foo> = context.deserialize_json_bytes(&bytes)?;
    println!("foos: {:?}", foos);

    Ok(())
}
```

The advantage of passing an additional context through `&self` becomes clear once we consider that the `Context` type can carry runtime values alongside trait implementations. We can further desugar the CGP code into TIR to see how everything reduces to dictionary-passing style:

```rust title="TIR"
struct Deserialize<'de, Context, T> {
    // Spoiler: dictionary-passing style requires Rust to also support
    // higher-ranked types.
    pub deserialize:
        for<D> fn(
            &Context,
            D,
            &serde::Deserializer<'de, Context, T>,
        ),
}

const fn deserialize_foo_ref<'de, 'a, Context, T>(
    get_arena: fn(&Context) -> &&'a BasicArena,
) -> Deserialize<'de, Context, &'a Foo> {
    Deserialize {
        deserialize: ...,
    }
}

fn main() -> Result<(), Error> {
    let bytes = read_some_bytes()?;

    struct Context<'de, 'a> {
        arena: &'a BasicArena,
        deserialize_foo_ref:
            Deserialize<'de, Context<'de, 'a>, &'a Foo>,
        deserialize_vec_foo_ref:
            Deserialize<'de, Context<'de, 'a>, Vec<&'a Foo>>,
    }

    let arena = arena::BasicArena::new();

    // Context value now has to be constructed at runtime.
    let context = Context {
        arena: &arena,
        deserialize_foo_ref:
            deserialize_foo_ref(|context| &context.arena),
        deserialize_vec_foo_ref:
            deserialize_extend(...),
    };

    let foos: Vec<&Foo> = context.deserialize_json_bytes(
        &bytes,
        // Pass getter for the dictionary for Deserialize<.., Vec<&'a Foo>>
        |context| &context.deserialize_vec_foo_ref,
    )?;

    println!("foos: {:?}", foos);

    Ok(())
}
```

---

## Limitations and Open Challenges

CGP's single-context approach to passing dependencies via a generic `Context` type provides several advantages. It makes the desugaring process simple and straightforward to understand, the TIR code is visually similar to the CGP code. The use of a single `Context` type also makes it easy to reason about the semantics of the Incoherent Rust code. Essentially, the `Context` type maintains a locally scoped coherence, which ensures that all code that uses the same `Context` type will receive the same trait implementation.

By using the `Context` type as a top-level dictionary for other trait dictionaries, it also simplifies the resolution of how to instantiate trait dictionaries with their dependencies, and allows cyclic dependencies to be resolved through the `Context` type.

That said, there are several limitations that may prevent the single-context approach from working in the general case.

### Limitations of CGP's single-context approach

#### Mutable or owned value access

The biggest limitation of the single-context approach is that the `Context` type is not able to easily supply mutable or owned values to trait implementations. Since the trait methods are desugared to always accept a `&self` parameter, we cannot get mutable or owned values out of the borrowed context value.

Furthermore, we cannot change the parameter to accept a `&mut self` or owned `self`. Doing so would virally affect all other code that attempts to call the method, essentially forcing all methods to accept `&mut self` or `self`. The use of `&mut self` or `self` also significantly limits the ability to call methods in any order, especially if the method also returns references that are borrowed from other parameters.

In Nadri's blog post, it is also mentioned that when capabilities containing `&mut` or owned values are used, the Rust compiler would have to keep track of the ownership and their use in trait implementations, and prevent multiple instantiation of trait implementations that may violate the ownership rules.

A simple way to work around this is to simply disallow the use of `&mut` or owned values as capabilities. After all, there are workarounds such as using `RefCell`, `Arc<Mutex>`, and `Clone` to mutate or copy values from a borrowed context.

However, in case the Rust compiler team decides to also support `&mut` or owned values as capabilities, then it may be necessary to explicitly pass everything as separate parameters, without grouping them through a single context.

#### Nested dynamic-scoped bindings

Another key limitation of CGP's approach is that all bindings of runtime values and trait implementations must be done at one place where the concrete context type is defined. This can severely limit the expressiveness of where one can call an incoherent trait implementation.

In the blog post **What If Traits Carried Values**, Nadri presented the idea of **scoped impls**, where a trait impl may be defined anywhere, and the use of the trait within that scope would use the given local impl, which overrides any global or outer impls.

For example, a user may want to freely introduce new bindings across different function calls, instead of limiting them into one place:

```rust title="Incoherent Rust"
fn deserialize_and_print_foo(json_string: &str) {
    let foo: Vec<&Foo> =
        with arena = &arena::BasicArena::new() {
            serde_json::deserialize(json_string).unwrap();
        };

    println!("foos: {:?}", foos);
}

fn deserialize_and_print_upper_foo(json_string: &str) {
    local impl<'de> Deserialize<'de> for String {
        // Deserialize string as uppercase
        ...
    }

    // Expect the string fields to become uppercase
    deserialize_and_print_foo();
}
```

With CGP's single-context approach, it would be very challenging to modify the bindings of the `Context` type dynamically depending on where it is called. The compiler would have to choose a place to define the `Context`, such as within `deserialize_and_print_foo`. But that would mean that this anonymous `Context` type would be inaccessible by `deserialize_and_print_upper_foo`, meaning that it would not be able to override the implementation of `Deserialize` for `String`.

If we think hard enough about how to support this through a single-context type, we might arrive at something that resembles **inheritance** in OOP. In fact, the `Context` type would need to have some kind of inheritance relationship with a parent context, if we want to modify the trait bindings at different scopes.

This is worth pointing out not to advocate for introducing OOP-style inheritance in Rust, but rather to highlight that the cost of supporting this kind of binding may result in some kind of inheritance-like behavior in Rust code.

Readers coming from Scala may also notice that this kind of code strongly resembles implicit parameters in Scala, which might not be what we want to support in Rust. CGP's implicit arguments are different from implicit parameters in Scala, in that all implicit arguments must be supplied in a flattened context type.

Due to these limitations, it may be worth considering whether it is worth allowing the user to bind capabilities or incoherent trait implementations anywhere in the code. Although it may be possible to desugar such code into dictionary-passing style in other ways, it may significantly complicate how we could reason about Incoherent Rust code.

On the other hand, the restricted version of Incoherent Rust provided by CGP may result in more disciplined Incoherent Rust code being written. Because all bindings have to be done at the same place as where a `Context` type is defined, CGP code usually has centralized locations for the reader to find out which implementations are used for that context. This results in a relatively simple semantics for reasoning about how context-generic code would behave within a specific context type.

### Challenges for implementing dictionary-passing style in Rust

It is worth getting excited about desugaring Rust traits into dictionary-passing style. However, there are some inherent challenges in implementing dictionary-passing style in Rust, due to it lacking some functional programming constructs that are common in languages like Haskell.

One reason that many people tend to overlook is just how powerful Rust traits really are. For most developers, Rust traits are conceptually similar to interfaces in OOP languages, and they most often encounter them in the context of `dyn` traits. For these basic use cases, it is easy to imagine how they could be desugared to dictionary-passing style with just vanilla Rust.

However, as demonstrated by CGP, Rust traits are powerful enough to almost rival advanced functional language features such as ML modules. With const generics, it is even possible to use the trait system to write type-level code that resembles dependent types. As such, it is natural that if we want to desugar traits into a simpler language, that simpler language would still need to offer equally powerful capabilities as what traits provide.

:::note
The main goal of this section is just to give a better understanding of the challenges involved in implementing dictionary-passing style in Rust, and to explore some potential solutions to those challenges.

The ideas presented in this section are entirely speculative. We are mainly exploring what kind of features might be needed in Rust to make it feasible to support dictionary-passing style. It is entirely possible that not all of these features are truly necessary. There may be other approaches not mentioned here that could also work.

For example, the use of dictionary-passing style could be significantly simplified if we disregard type safety and simply use `unsafe` and raw pointers to implement and use the dictionaries. That said, if the main goal of TIR is to implement dictionary-passing style in a type-safe manner, then the challenges described in this section might still be relevant.
:::

#### Currying and first-class closures

The first challenge is the lack of currying and first-class closure types in Rust. If you examine the conceptual desugaring of dictionary-passing style in Rust, you will notice that we are essentially writing functions that return other functions, or more accurately, closures.

This style of programming is natural through currying in languages like Haskell. But in Rust, it requires fairly heavy syntax such as the following:

```rust title="TIR"
pub const fn clone_vec_fn<T>(
    clone_value_fn: impl Fn(&T) -> T,
) -> impl Fn(&Vec<T>) -> Vec<T> {
    ...
}
```

The main issue with this approach is that `Fn` is still a trait in Rust. So if the main goal is to eliminate generic traits in an intermediate representation, we cannot keep the `Fn` trait in its current form either.

Furthermore, since each Rust closure has its own unique type, trait dictionaries would technically need to contain generic parameters to accommodate those closures. For example, the `Clone` dictionary would technically need to be defined as:

```rust title="TIR"
pub struct Clone<CloneFn, T>
where
    CloneFn: Fn(&T) -> T,
{
    pub clone: CloneFn,
}
```

One possible workaround is to support some form of const currying at compile time using `const` functions. This would allow us to define curried closures with static allocation. For example:

```rust title="TIR"
pub const fn clone_vec_fn<T>(
    clone_value_fn: fn(&T) -> T,
) -> fn(&Vec<T>) -> Vec<T> {
    ...
}
```

The key change here is that `clone_vec_fn` now accepts a `fn` function pointer, and returns a new `fn` pointer. This would require some compiler support to capture the given `clone_value_fn` pointer at compile time and define a new Rust function at the call site during compilation.

Such compile-time currying might not be too difficult to implement in Rust. If it could be done, we could simplify trait dictionaries to use function pointers again:

```rust title="TIR"
pub struct Clone<T> {
    pub clone: fn(&T) -> T,
}
```

We can then define a generic `const fn` that builds a `Clone` dictionary for `Vec<T>` by accepting a `Clone<T>` dictionary and returning a new `Clone<Vec<T>>` dictionary:

```rust title="TIR"
pub const fn clone_vec_dictionary<T>(
    clone_value_dictionary: Clone<T>,
) -> Clone<Vec<T>> {
    Clone {
        clone: clone_vec_fn::<T>(clone_value_dictionary),
    }
}
```

##### Const-generic syntax for compile-time currying

It is worth noting that the curried `const fn` described here works similarly to Nadri's use of const generics to curry dictionary constructor functions. For example, we could redefine `clone_vec_fn` and `clone_vec_dictionary` to become constants that accept their parameters through const generics:

```rust title="TIR"
pub const CLONE_VEC_FN<
    T,
    const CLONE_VALUE_FN: fn(&T) -> T,
>: fn(&Vec<T>) -> Vec<T> =
    fn(values: &Vec<T>) -> Vec<T> { ... };

pub const CLONE_VEC_DICTIONARY<
    T,
    CLONE_VALUE_DICTIONARY: Clone<T>,
>: Clone<Vec<T>> = Clone {
    clone: CLONE_VEC_FN::<T, CLONE_VALUE_DICTIONARY.clone>,
}
```

Regardless of whether we use const generics or curried const functions, the two approaches are almost semantically equivalent. We could desugar almost all uses of const values parameterized by const generics into curried const functions, or vice versa. As a result, this is more a matter of syntactic preference for the desugaring strategy. In this blog post, we use curried const functions, since their syntax is closer to functional programming and makes the use of compile-time currying more explicit.

#### Higher-ranked types

Aside from the lack of curried functions, Rust also lacks support for higher-ranked types, which would be required to properly support generic functions in trait dictionaries. We have already seen this issue when trying to desugar the `Deserialize` trait into a dictionary:

```rust title="Rust"
pub trait Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>;
}
```

Since the `deserialize` function accepts a generic `D` parameter, we would need higher-ranked types to desugar it to something like:

```rust title="TIR"
pub struct Deserialize<'de> {
    pub deserialize:
        for<D> fn(
            deserializer: D,
            deserializer_dictionary: &Deserializer<'de>,
        ) -> Result<Self, D::Error>;
}
```

Since Rust does not currently support the use of higher-ranked `for<D>` in `fn` pointer types, it is challenging to define such a `Deserialize` dictionary.

One potential workaround is to add limited support for generics inside valid `fn` pointers. After all, if the Rust compiler already has all metadata about a `fn` pointer at compile time, it may be able to recover that information and perform the necessary monomorphization when a generic `fn` pointer is called.

#### Dependent types or type families

Another key issue is the lack of type families to support the desugaring of associated types in traits. If we carefully examine the desugared `Deserialize` struct, we would notice that the `D::Error` type is technically `<D as Deserializer<'de>>::Error`, which still refers to the `Deserializer` trait that is supposed to have been turned into a dictionary value.

If we were to extract the `Error` type from `deserializer_dictionary`, that would effectively require some kind of dependent type support in Rust:

```rust title="TIR"
pub struct Deserializer<'de> {
    pub Error: Type, // type as value
    ...
}

pub struct Deserialize<'de> {
    pub deserialize:
        for<D> fn(
            deserializer: D,
            deserializer_dictionary: &Deserializer<'de>,
        ) -> Result<Self, deserializer_dictionary.Error>;
}
```

Notice that the returned error type is now `Result<Self, deserializer_dictionary.Error>`, indicating that the `Error` type depends on the value of `deserializer_dictionary`.

Supporting even a subset of dependent types in Rust would be a significant undertaking. But if we were to go down this path, one possibility is to have limited support for dependent types through a new `const fn` pointer such as:

```rust title="TIR"
pub const struct Deserializer<'de> {
    pub Error: Type, // type as value
    ...
}

pub struct Deserialize<'de> {
    pub deserialize:
        for<D> const fn(
            deserializer_dictionary: &'static Deserializer<'de>,
        ) ->
            fn(deserializer: D) ->
                Result<Self, deserializer_dictionary.Error>;
}
```

In the above example, the `deserialize` field becomes a curried `const fn` pointer that can not only use generics, but also return `fn` pointers that contain types depending on its arguments. We also introduce a new `const struct` type for defining trait dictionaries, which allows `Type` fields to be present in the struct but only usable at compile time.

There would still be many unknowns around whether we can truly implement something like `const fn` in Rust that supports currying, higher-ranked types, and dependent types. The challenges may be too significant to make it worthwhile, but from a programming language theory perspective, it is likely at least theoretically possible.

It is also worth noting that something like the `const fn` concept shown here could be useful for the current work on [compile-time reflection](https://rust-lang.github.io/rust-project-goals/2025h2/reflection-and-comptime.html). It might even be possible for Rust to support the full `comptime` concept through `const fn`. In that sense, support for curried `const fn` could be valuable not only for dictionary-passing style but also for reflection and comptime in Rust.

#### Ownership and lifetime

As mentioned earlier, Rust's borrow checker makes it challenging to pass runtime dependencies through a common `&Context` reference. This limitation is unique to Rust. Functional languages like Haskell only have immutable values that are reference counted, and this makes dictionary-passing style relatively straightforward to implement.

However, it is worth noting that with the recent introduction of linear types, even Haskell's approach to dictionary-passing style faces similar issues, in that it cannot fully support linear values. For example, it is not possible to bind linear values to implicit parameters. To support constraints that can only be used exactly once, Haskell requires even more advanced features such as [linear constraints](https://www.tweag.io/blog/2023-01-26-linear-constraints-freeze/) to be implemented in GHC.

This shows that features like linear or affine types can significantly complicate the implementation strategy for both the surface language and the desugared dictionary-passing code.

#### Type equalities

Nadri's blog post also mentioned the challenges with desugaring type equalities. Suppose that we want to write a `sum_u64` function that can sum over any `Iterator` implementation with the `Item` type being `u64`. We can easily implement this in Rust today as:

```rust title="Rust"
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

pub fn sum_u64<I>(
    values: I,
) -> u64
where
    I: Iterator<Item = u64>,
{
    let mut acc: u64 = 0;
    acc
}
```

The key thing to note here is that `sum_u64` works with a generic `I` type implementing `Iterator`, with the requirement that `<I as Iterator>::Item === u64`. The challenge is how to desugar this type equality into TIR.

One potential solution is to introduce a new primitive like `T: TypeEq<u64>`, to allow us to encode the type equality relationship without using the associated type equality syntax:

```rust title="Incoherent Rust"
pub fn sum_u64<I>(
    values: I,
) -> u64
where
    I: Iterator,
    I::Item: TypeEq<u64>, // New TypeEq primitive
{
    let mut acc: u64 = 0;
    acc
}
```

We can then desugar `sum_u64` to accept `TypeEq` as a type equality proof, in addition to the `Iterator` dictionary:

```rust title="TIR"
pub struct Iterator<I> {
    pub Item: Type,
    pub next: fn(&mut I) -> Option<self.Item>; // a little bit of self-reference magic
}

pub const fn sum_u64<I>(
    iterator_dictionary: Iterator<I>,
    item_eq_u64: TypeEq<iterator_dictionary.Item, u64>,
) -> fn(I) -> u64
{
    |mut values| {
        let mut acc: u64 = 0;

        while let Some(value) = iterator_dictionary.next(&mut values) {
            acc += value
        }

        acc
    }
}
```

The key thing to notice here is that `TypeEq` becomes a built-in Rust construct that allows the compiler to treat `iterator_dictionary.Item` and `u64` as being the same type. With the type equality proof in scope, the expression `acc += value` can be type checked successfully.

It is worth noting that today's Rust does not have any equivalent to the `TypeEq` construct. For example, it is impossible to define something like the following in Rust:

```rust title="Rust"
pub fn transmute<T, U>(
    value: T,
) -> U
where
    // Not possible in Rust today
    T: TypeEq<U>,
{
    value
}
```

In today's Rust it is impossible to specify a constraint that two generic types are equal. Instead, Rust provides a more restrictive form of type equality, where at least one side of the equality must be an associated type. A main reason for this restriction is that a fully generalized type equality constraint could significantly complicate the type checker. If we want to desugar the type equality constraints in TIR, then Rust would need to support fully generalized type equality that can handle not only equality between two generic types, but also value-dependent equality between generic types and type fields in dictionary items.

#### Dictionary equality

In addition to how to desugar type equalities, Nadri's blog post also mentioned the challenges of asserting the equality of associated types in trait dictionaries. This is particularly an issue when trait dictionaries are generated from different sources, such as through blanket implementations as compared to being passed explicitly.

One subtle challenge with mixing coherent and incoherent traits in dictionary-passing style is this problem of dictionary equality. To understand it concretely, we will use a trait that makes the issue visible.

Consider the following trait, renamed here from Nadri's original example to make the concept more vivid:

```rust title="Rust"
trait Refl {
    type Refl;
}

impl<T> Refl for T {
    type Refl = T;
}
```

The `Refl` trait contains a `Refl` associated type, and it has a blanket implementation for all `T` by setting `Refl` to be the same as `T`. This means that `T === <T as Refl>::Refl` for all types.

With the coherence rules, it must always be the case that `T === <T as Refl>::Refl`, because it is impossible to define a conflicting implementation that overrides the associated type:

```rust title="Rust"
// Error: conflicting implementations of trait `Refl` for type `u32`
impl Refl for u32 {
    type Refl = u64;
}
```

With this guarantee in place, Rust allows us to define a function like the following:

```rust title="Rust"
// No `T: Refl` bound
pub fn reflect<T>(t: T) -> <T as Refl>::Refl {
    t
}
```

The `reflect` function takes any value of type `T` and returns that value as the type `<T as Refl>::Refl`. Since Rust knows this must always be the case through the blanket implementation, it allows the code to compile successfully. It is worth noting that there is no trait bound in `reflect` that requires `T` to implement `Refl`. Despite that, we are still able to use the `Refl` trait in `<T as Refl>::Refl` through the blanket implementation.

Now let's see what happens if we add the `T: Refl` trait bound explicitly:

```rust title="Rust"
pub fn cast<T: Refl>(t: T) -> <T as Refl>::Refl {
    // Error: mismatched types
    t
}
```

When the local `T: Refl` trait bound is added, Rust essentially shadows the global blanket implementation and assumes that this `Refl` implementation could differ from the blanket one. Because of that, we can no longer treat `T === T::Refl` and return `t` here.

Interestingly, implementing `cast` by calling `reflect` works fine:

```rust title="Rust"
pub fn cast<T: Refl>(t: T) -> <T as Refl>::Refl {
    reflect(t) // It works!
}
```

The reason this works is that Rust sees `reflect` returns a `<T as Refl>::Refl`, but it does not check that this `<T as Refl>::Refl` uses the global `impl<T> Refl for T` implementation rather than the local `T: Refl` implementation given to `cast`. Intuitively, this makes sense because the coherence rules already enforce that there can never be two overlapping implementations. So if there are two uses of `<T as Refl>::Refl` from different places, they must always be equal.

The issue Nadri mentioned arises when we try to desugar the Rust code into dictionary-passing style:

```rust title="TIR"
pub const struct Refl<T> {
    pub Refl: Type,
}

pub const fn global_refl<T>() -> Refl<T> {
    Refl {
        Refl: T,
    }
}
```

First, `Refl` becomes a dictionary struct containing a `Refl` type field. We then have a `global_refl` dictionary constructor that returns a `Refl<T>` for all `T`, with the `Refl` field being `T`.

When desugaring `reflect`, it becomes:

```rust title="TIR"
pub fn reflect<T>(t: T) -> global_refl::<T>().Refl {
    t
}
```

The desugared `cast` function becomes:

```rust title="TIR"
pub const fn cast<T>(local_refl: Refl<T>) ->
    fn(T) -> local_refl.Refl
{
    | t | {
        // Type mismatch: requires local_refl === global_refl::<T>()
        // Or: local_refl.Refl === global_refl::<T>().Refl
        reflect::<T>()
    }
}
```

It now becomes clear that the desugared `cast` accepts a `local_refl` dictionary for `Refl` and uses `local_refl.Refl` as the return type. But since there is no evidence that `local_refl` must always be the same as `global_refl::<T>()`, type checking would fail here.

Thinking of this from the context of Incoherent Rust makes a lot of sense. If `Refl` were an incoherent trait, the assumption that `local_refl` is always the same as `global_refl` would be invalidated, since someone could implement a local impl of `Refl` with a different `Refl::Refl` type.

However, if we want coherent and incoherent traits to coexist in Rust, there needs to be some way to allow dictionary-passing style to still work correctly for coherent traits.

As Nadri mentioned, finding a robust solution to the problem of dictionary equality remains an open research problem. One speculative idea is that Rust could generate axioms about coherence properties when desugaring high-level Rust code into TIR. The axiom would look something like:

```rust title="TIR"
pub const fn coherent_refl::<T>(
    local_refl: Refl<T>,
) -> TypeEq<local_refl.Refl, global_refl::<T>().Refl> {
    // built-in implementation
}
```

The syntax starts to break down here. But essentially, we are asserting that for any `local_refl` value of type `Refl<T>`, there is a proof that the type `local_refl.Refl` must be identical to the type in `global_refl::<T>().Refl`.

Using `TypeEq`, we can implement `cast` as something like the following:

```rust title="TIR"
pub const fn cast<T>(local_refl: Refl<T>) ->
    fn(T) -> local_refl.Refl
{
    // pattern matches and bring the equality proof into scope
    let TypeEq(_) = coherent_refl(local_refl);

    | t | {
        // the compiler now considers `local_refl.Refl === global_refl::<T>().Refl`
        reflect::<T>()
    }
}
```

Using some equality machinery that does not currently exist in Rust, we would allow these equality constraints to be present in the type checker and allow the code to compile.

An axiom like `coherent_refl` would only be generated for coherent traits. This means that for incoherent traits, these axioms would not be present in TIR, making it impossible to accidentally mix up associated types from incoherent traits.

---

## Where CGP diverges from Incoherent Rust

In this blog post, we have so far covered how CGP can be used to implement Incoherent Rust. While that is all exciting, it is also worth pointing out some features in CGP that are not directly supported by the current designs of Incoherent Rust.

### Named impls as provider types

The most notable difference is that the named implementations in Boxy's and Nadri's blog posts are treated as a different kind of entity than normal types in Rust. Given named impls like:

```rust title="Incoherent Rust"
impl Impl1 = Clone for MyType { ... }
impl Impl2 = Clone for MyType { ... }
```

the implementations `Impl1` and `Impl2` are not Rust types, and thus cannot be used as types or passed to generic parameters.

In contrast, CGP's named implementations are called providers, and they are just regular Rust types that can be composed with other Rust types.

#### Multiple implementations on the same name

Most notably, this allows a CGP provider to implement multiple provider traits. For example, the equivalent in Incoherent Rust would be something like:

```rust title="Incoherent Rust"
impl MyMatrixSerde = Serialize for Matrix { ... }

// How to disambiguate intentional name sharing vs accidental naming?
impl MyMatrixSerde = Deserialize for Matrix { ... }
```

Where `MyMatrixSerde` is a named implementation that implements both `Serialize` and `Deserialize` for `Matrix`. With CGP, this would be implemented as:

```rust title="CGP"
// A provider is just a struct
pub struct MyMatrixSerde;

#[cgp_impl(MyMatrixSerde)]
impl SerializeImpl<Matrix> { ... }

#[cgp_impl(MyMatrixSerde)]
impl DeserializeImpl<Matrix> { ... }
```

#### Grouping named impls

Since named impls in CGP are just types, it becomes much easier to group related implementations together so they can be referred to by a single name. For example, we might want to group the `Serialize` and `Deserialize` implementations of all domain types defined across different modules.

With Incoherent Rust, we might need additional syntax to support aliasing, such as:

```rust title="Incoherent Rust"
impl MyAppSerde = Serialize for Matrix = MyMatrixSerialize;
impl MyAppSerde = Deserialize for Matrix = MyMatrixDeserialize;
impl MyAppSerde = Serialize for Scalar = MyScalarSerialize;
impl MyAppSerde = Deserialize for Scalar = MyScalarDeserialize;
```

On the other hand, such grouping is considerably more streamlined in CGP, with specialized notation in `delegate_components!`:

```rust title="CGP"
delegate_components! {
    MyAppSerde {
        open SerializeComponent, DeserializeComponent;

        @SerializeComponent.Matrix:
            MyMatrixSerialize,
        @DeserializeComponent.Matrix:
            MyMatrixDeserialize,
        @SerializeComponent.Scalar:
            MyScalarSerialize,
        @DeserializeComponent.Scalar:
            MyScalarDeserialize,
    }
}
```

#### Higher order providers

CGP also supports the concept of higher-order providers, where a provider explicitly accepts other providers as generic parameters. This enables powerful design patterns where provider bindings become strongly coupled, rather than the default loose coupling through the context.

For example, we could define a `SerializeIterator` provider like:

```rust title="Incoherent Rust"
impl<T, ItemSerializer> SerializeIterator<ItemSerializer> =
    Serialize for T
where
    T: IntoIterator,
    ItemSerializer: Serialize for T::Item,
{ ... }
```

Where the named impl `SerializeIterator` accepts an inner named impl `ItemSerializer` that serializes the inner `<T as IntoIterator>::Item` type.

With CGP, this would be implemented as:

```rust title="CGP"
// A higher-order provider is just a struct with phantom generic parameters
pub struct SerializeIterator<ItemSerializer>(pub PhantomData<ItemSerializer>);

#[cgp_impl(SerializeIterator<ItemSerializer>)]
// Inner provider import
#[use_provider(ItemSerializer: SerializeImpl<T::Item>)]
impl<T, ItemSerializer> SerializeImpl<T>
where
    T: IntoIterator,
{ ... }
```

#### Type-level DSLs

Since CGP's named implementations are just types, the use of higher order providers also enables design patterns that go beyond what Incoherent Rust currently supports. For example, in the [Hypershell demo](/blog/hypershell-release), we use higher-order providers to build a type-level DSL for shell scripting in Rust.

After desugaring, the DSL code becomes simple Rust types that look like:

```rust title="CGP"
// A Hypershell program is just a type containing higher order providers
pub type Program = Pipe<Product![
    SimpleExec<
        StaticArg<Symbol!("echo")>,
        WithStaticArgs<Product![
            Symbol!("hello"),
            Symbol!("world!"),
        ]>,
    >,
    StreamToStdout,
]>
```

Which corresponds to the shell command `echo hello world!`. If Incoherent Rust treats named implementations differently from regular Rust types, it would be much more difficult to implement such type-level DSLs through named impls.

### Incoherent Functions

One notable CGP feature that we have seen earlier is CGP functions annotated with `#[cgp_fn]`, which can be considered as a new concept that I call **incoherent functions**.

The core idea behind incoherent functions is that they can use an incoherent trait without knowing which implementation is actually chosen. Instead, the choice of implementation is deferred to a coherent caller.

For example, with the `do_stuff` CGP function from earlier:

```rust title="CGP"
#[cgp_fn]
#[uses(Name<T>)]
pub fn do_stuff<T>(&self, foo: &Foo<T>) {
    println!("{}", Self::NAME);
}
```

We can think of it as being equivalent to the following incoherent Rust function:

```rust title="Incoherent Rust"
pub incoherent fn do_stuff<T>(foo: &Foo<T>)
where
    impl NameImpl: Name<T>,
{
    println!("{}", Name::<T>::NAME);
}
```

The key thing to notice is that `do_stuff` can use an incoherent trait like `Name<T>` without knowing which implementation is being used. Beyond that, it also allows other incoherent functions to call `do_stuff` without requiring the caller to explicitly specify the `where` bound. For example:

```rust title="Incoherent Rust"
pub incoherent fn do_more_stuff<T>(foo: &Foo<T>) {
    // no need to specify `where impl NameImpl: Name<T>`
    do_stuff(foo);
}
```

which is equivalent to the CGP function:

```rust title="CGP"
#[cgp_fn]
#[uses(DoStuff<T>)]
pub fn do_more_stuff<T>(&self, foo: &Foo<T>) {
    do_stuff(foo);
}
```

The core idea here is that incoherent functions can call other incoherent functions without needing to specify all transitive incoherent trait bounds required by the inner functions. Instead, all incoherent trait bounds are gathered and resolved all at once when we exit the incoherent world and call it from a coherent function.

This effectively introduces an incoherent world that is isolated from the coherent world, where all existing Rust code lives. This has a similar function coloring property to async functions: incoherent traits and functions can freely call other coherent or incoherent traits and functions, but coherent traits and functions cannot call incoherent traits and functions without explicitly binding all the required incoherent trait implementations.

This separation between the incoherent and coherent worlds is essential, because it simplifies the semantics and creates a clear boundary for where the bindings for incoherent implementations can be specified.

In CGP, the incoherent world is simply any code that works with a generic context. The earlier functions, for instance, desugar into blanket implementations as follows:

```rust title="Rust"
pub trait DoStuff<T> {
    fn do_stuff(&self, foo: &Foo<T>);
}

impl<Context, T> DoStuff<T> for Context
where
    Context: Name<T>,
{
    fn do_stuff(&self, foo: &Foo<T>) {
        println!("{}", Self::NAME);
    }
}

pub trait DoMoreStuff<T> {
    fn do_more_stuff(&self, foo: &Foo<T>);
}

impl<Context, T> DoMoreStuff<T> for Context
where
    Context: DoStuff<T>,
{
    fn do_more_stuff(&self, foo: &Foo<T>) {
        Self::do_stuff(foo);
    }
}
```

As we can see, incoherent functions desugar into traits that have blanket implementations over a generic context. By making use of impl-side dependencies, we can hide the dependencies of other CGP functions behind the trait interface. In CGP, the boundary between the incoherent and coherent worlds is determined by where a top-level context type is defined. Once that context type is defined, all incoherent implementation bindings are finalized, and we return to the coherent world that does not require the use of a generic context.

As mentioned earlier, this clean separation between the incoherent and coherent worlds is only possible in the absence of dynamic-scoped impls and capabilities. A full version of Incoherent Rust may not preserve such a clear boundary, which would lead to quite different semantics compared to CGP.

#### Incoherence as coeffect

For readers who are familiar with the concept of monads and algebraic effects, the incoherent world with generic context can be understood as a form of effect, or more precisely, a [**coeffect**](https://tomasp.net/coeffects/).

In CGP, each incoherent trait definition can be thought of as defining a new kind of effect, and each CGP provider implementation can be thought of as an effect handler for that effect. The definition of concrete contexts and the binding of incoherent implementations through `delegate_components!` plays the same role as defining concrete monads and configuring effect handlers in algebraic effects systems found in functional languages like Haskell. The CGP bindings effectively handle the incoherence effects by delegating them to CGP providers, and the result is code that operates coherently within a fixed context, analogous to returning a pure value once all effects have been handled.

That said, the effect system implied by CGP is considerably more limited than a full algebraic effects system. In particular, CGP effects only support linear continuations, because Rust does not have support for delimited continuations.

There is also an important way in which CGP diverges from traditional algebraic effects. CGP supports the injection of *abstract types* through associated types in trait definitions. Traditional algebraic effects have no equivalent notion of passing a type through an effect handler, unless the system also incorporates dependent types that allow types to be treated as ordinary values. This gives CGP a character that is genuinely distinct from the algebraic effects model.

These observations point toward a better fit in the [**coeffects**](https://tomasp.net/coeffects/) framework, which studies how contextual requirements flow through a computation rather than how computational effects flow out of it. Where effects describe what a computation produces or performs, coeffects describe what a computation requires from its environment. The generic context in CGP serves precisely this role: it carries the implementation choices and capabilities that a computation depends on, and those dependencies are resolved all at once when a concrete context is defined. A full introduction to coeffects is beyond the scope of this blog post, but the connection is worth noting for readers interested in the theoretical foundations of what CGP is doing.


### 0-arity traits

CGP provides a desugaring of incoherent Rust traits into CGP traits that have an additional `Context` type, with the original `Self` type moved to an explicit generic parameter. For example, given the incoherent trait:

```rust title="Incoherent Rust"
pub incoherent trait Serialize {
    fn serialize(&self) -> String;
}
```

it becomes the CGP trait:

```rust title="CGP"
#[cgp_component(SerializeImpl)]
pub trait Serialize<T> {
    fn serialize(&self, value: &T) -> String;
}
```

with the original `Self` type becoming the explicit `T` parameter.

However, in CGP it is also common to define CGP traits with no extra generic parameter, such as:

```rust title="CGP"
#[cgp_component(GreetImpl)]
pub trait Greet {
    fn greet(&self);
}
```

These traits would be conceptually equivalent to an incoherent Rust trait with zero arity, meaning one without a `Self` type:

```rust title="Incoherent Rust"
pub incoherent trait Greet
    for ! // placeholder: forbid use of `Self`
{
    fn greet(); // no use of `self` or `Self`
}
```

With incoherence and capabilities, a useful design pattern that emerges is the use of zero-arity incoherent traits to implement behaviors by working solely with the ambient context. For example, we could implement the `Greet` trait with:

```rust title="Incoherent Rust"
impl GreetHello = Greet for !
with
    name: String,
{
    fn greet() {
        println!("Hello, {}!", name);
    }
}
```

With CGP, since the context type is explicit, zero-arity incoherent traits become unary CGP traits, which is less awkward to express:

```rust title="CGP"
#[cgp_impl(new GreetHello)]
impl GreetImpl {
    fn greet(
        &self,
        #[implicit] name: String,
    ) {
        println!("Hello, {}!", name);
    }
}
```

### Contexts as explicit types

Another key distinction is that CGP contexts are explicit Rust types. This means we can use the context as a regular Rust type inside a larger context. Incoherent Rust, on the other hand, assumes an implicit ambient context that propagates named impls from callers to callees. This makes it more restrictive to use Incoherent Rust when implementing traits for concrete types.

For example, in CGP it is common to define an application context that contains both provider wirings and runtime fields, together with additional constructors for building that context:

```rust title="CGP"
#[derive(HasField)]
pub struct App {
    // Runtime capabilities accessible by providers
    pub database: Database,
    pub s3_client: S3Client,
    pub redis_cache: redis::Client,
    ...
}

delegate_components! {
    App {
        // Use higher order provider to provide caching middlewares
        UserQuerierComponent:
            CachedQueryUser<QueryUserWithDatabase>,
        ...
    }
}

// App is used in the return type
pub async fn build_app_from_config(
    config: &Config,
) -> Result<App, Error> {
    ...
}
```

With Incoherent Rust, it might be possible to implement something like `query_user` through named impls and capabilities, but each binding would need to be done explicitly. As a result, developers may need to write considerably more verbose code to bridge usage into concrete contexts:

```rust title="Incoherent Rust"
impl App {
    pub async fn query_user(
        &self,
        user_id: &UserId,
    ) -> Result<User, Error> {
        // convert field values into capabilities using `with`
        with
            database = &self.database,
            redis_cache = &self.redis_cache,
        {
            // Explicitly call named impls
            <CachedQueryUser<QueryUserWithDatabase>
                as UserQuerier
            >::query_user().await
        }
    }
}
```

Because CGP works directly with concrete Rust types rather than ambient capabilities, it becomes much easier to implement incoherent traits on an explicit context type, compared to the manual bridging that would be required with zero-arity incoherent trait implementations.

---

## Related Work

It is worth briefly exploring other approaches that touch on similar ideas to Incoherent Rust, aside from CGP and dictionary-passing style.

### Cairo

One of the less well-known languages with a trait system that resembles Incoherent Rust is [Cairo](https://www.cairo-lang.org/), which is a smart contract language heavily inspired by Rust. While Cairo shares many surface-level similarities with Rust, its [trait system](https://www.starknet.io/cairo-book/ch08-02-traits-in-cairo.html) works quite differently. In fact, Cairo's design is much closer in spirit to what Incoherent Rust proposes.

For example, the `Serde` trait in Cairo is defined as something like the following. To make it easier for Rust users to follow, some Cairo syntax has been adjusted to look more like Rust. Cairo uses `Array` instead of `Vec` to represent a contiguous list of values, and `felt252` is a Cairo-specific primitive representing an integer from 0 up to the prime number 2^252, used to support zero-knowledge proofs.

```rust title="Cairo"
pub trait Serde<T> {
    fn serialize(self: &T, output: &mut Array<felt252>);
}
```

Notice that there is no implicit `Self` type. All type parameters appear explicitly in the trait's generic parameter list. The `self` receiver can be of any type and mainly serves as syntactic sugar for method call syntax.

Cairo allows multiple trait implementations to coexist, and each implementation must be given a name. The following is a generic implementation of `Serde` for `Array<T>`:

```rust title="Cairo"
impl ArraySerde<T, impl ValueSerializer: Serde<T>, +Drop<T>> of Serde<Array<T>> {
    fn serialize(self: &Array<T>, output: &mut Array<felt252>) { ... }
}
```

Cairo's trait implementations follow the syntax `impl ImplName<...> of TraitName<...>`, where `ImplName` is the name given to that particular implementation. All generic types and trait dependencies must be listed explicitly in the generic parameters after the implementation name.

Cairo supports two forms of trait dependencies. The syntax `impl ValueSerializer: Serde<T>` accepts an implementation of `Serde<T>` and binds it locally as `ValueSerializer`. The syntax `+Drop<T>` accepts an implementation of `Drop` without assigning it a local name.

One important consequence of this design is that all transitive dependencies of a trait implementation must be explicitly imported and brought into scope at the call site. For example, to call `serialize` on a type like `Array<(u8, bool)>`, one would write something like the following:

```rust title="Cairo"
// All named impls must be explicitly imported
use crate::serde::Serde;
use crate::array::ArraySerde;
use crate::tuple::TupleSerde;
use crate::primitive::{U8Serde, BoolSerde};

fn test_serialize() -> Array<felt252> {
    let value: Array<(u8, bool)> = array![(1, false), (2, true), (3, true)];
    output
}
```

The Cairo compiler automatically scans the implementations in scope and implicitly passes them to the generic trait implementations to produce a concrete implementation for the given value type. However, this means the caller must know the full chain of intermediate implementations needed. This can be partially mitigated by using a glob import like `use crate::impls::*`, but that introduces a different problem: if multiple overlapping implementations are brought into scope at once, the compiler will fail to find a unique implementation.

Another issue is that since all trait implementations are resolved at each call site independently, different call sites in the same project can end up using different implementations for the same type, depending on what is imported. The complete absence of any coherence guarantee in Cairo makes it hard to enforce a consistent implementation, even within a single project.

#### Comparing Cairo with CGP

CGP supports a similar style of implementation as Cairo, though it is less idiomatic than how CGP code is normally written. The following shows how the earlier Cairo code translates directly into CGP, followed by a more idiomatic CGP version.

```rust title="CGP"
#[cgp_component(SerdeImpl)]
pub trait Serde<T> {
    fn serialize(&self, value: &T, output: &mut Array<felt252>) { ... }
}

#[cgp_impl(new ArraySerde<T, ValueSerializer>)]
#[use_provider(ValueSerializer: SerdeImpl<T>)]
impl SerdeImpl<Array<T>> {
    fn serialize(&self, value: &Array<T>, output: &mut Array<felt252>) { ... }
}
```

The `Serde<T>` trait is defined using `#[cgp_component]`, with an explicit `Context` type as `Self` and the value type `T` as an explicit generic parameter. The `ArraySerde` provider is defined as a higher-order provider that accepts both `T` and the generic `ValueSerializer` provider. The `#[use_provider]` attribute adds the constraint that `ValueSerializer` must implement `SerdeImpl<T>`.

Calling `serialize` on an `Array<(u8, bool)>` value would then look like this in direct CGP style:

```rust title="CGP"
use crate::serde::SerdeImpl;
use crate::array::ArraySerde;
use crate::tuple::TupleSerde;
use crate::primitive::{U8Serde, BoolSerde};

fn test_serialize() -> Array<felt252> {
    let value: Array<(u8, bool)> = array![(1, false), (2, true), (3, true)];
    let mut output: Array<felt252> = array![];

    // Dummmy context
    struct Context;

    <ArraySerde<(u8, bool), TupleSerde<U8Serde, BoolSerde>>
        as SerdeImpl<Array<(u8, bool)>>
    >::serialize(&Context, &mut output);

    output
}
```

In the `test_serialize` function, the provider `ArraySerde<(u8, bool), TupleSerde<U8Serde, BoolSerde>>` is explicitly assembled and called with a dummy `Context` type. This explicit assembly is precisely what Cairo infers implicitly from the implementations in scope. Without that implicit inference, using higher-order providers in this direct style can be quite tedious.

In idiomatic CGP, providers are not usually assembled and passed around explicitly at call sites. Instead, the provider configuration is deferred to wherever a top-level context is defined. The `ArraySerde` provider can be simplified as follows:

```rust title="CGP"
#[cgp_impl(new ArraySerde)]
#[uses(Serde<T>)]
impl SerdeImpl<Array<T>> {
    fn serialize(&self, value: &Array<T>, output: &mut Array<felt252>) { ... }
}
```

The `ArraySerde` provider no longer takes an explicit `ValueSerializer` parameter. Instead of requiring the caller to pass in a provider for the inner value type, the `#[uses]` attribute declares that the context must implement the consumer trait `Serde<T>`. This defers the resolution of which provider handles `T` to whichever context is used at the call site.

The `test_serialize` function becomes:

```rust title="CGP"
// Only import the consumer trait
use crate::serde::Serde;

#[cgp_fn]
#[uses(Serde<Array<(u8, bool)>>)]
fn test_serialize(&self) -> Array<felt252> {
    let value: Array<(u8, bool)> = array![(1, false), (2, true), (3, true)];
    output
}
```

The function is now context-generic and only imports the consumer trait `Serde`. The providers for inner types like `ArraySerde` and `TupleSerde` are no longer imported here. Instead, that configuration is moved to where a concrete context is defined:

```rust title="CGP"
use crate::serde::Serde;
use crate::array::ArraySerde;
use crate::tuple::TupleSerde;
use crate::primitive::{U8Serde, BoolSerde};

pub struct MyApp;

delegate_component! {
    MyApp {
        open SerdeComponent;

        <T> @SerdeComponent.Array<T>: ArraySerde,
        <T, U> @SerdeComponent.(T, U): TupleSerde,
        u8: U8Serde,
        bool: BoolSerde,
    }
}

fn main() {
    let app = MyApp;

    let value = app.test_serialize();
    ...
}
```

At first glance this might seem like the same amount of work, since the provider configuration still needs to be written somewhere. The difference is that this configuration is written only once inside the top-level context. Any other function that also uses `Serde` can be written in terms of the consumer trait and reuse the same context wiring, without repeating the provider assembly at every call site.

#### Lessons for Rust

If the Rust compiler team is committed to implementing Incoherent Rust, it is worth studying Cairo's trait system carefully and thinking through the developer experience implications. Cairo's approach of resolving implementations implicitly from whatever is in scope is convenient in simple cases but creates real fragility at scale. CGP's approach of deferring all configuration to a top-level context is more verbose upfront but leads to more predictable and maintainable code.

Both approaches have trade-offs, and the right choice may depend on the specific use case. It is worth considering whether Incoherent Rust should support both styles, or commit to one as the primary design.

### ML Modules

In the desugaring of traits into dictionary-passing style in TIR, the presence of types and functions inside a dictionary struct bears a strong resemblance to ML modules. Similarly, defining a `const fn` that accepts a dictionary struct and returns a new dictionary struct works much like how ML functors work.

Given how closely TIR resembles ML modules, a key question for the implementors is whether they are effectively building an ML-module-like system inside Rust. That would certainly unlock capabilities well beyond dictionary-passing style, but it also comes with significant technical challenges that would need careful evaluation before committing to the approach.

### Scala Implicits

In the full version of Incoherent Rust that incorporates scoped impls and capabilities, the resulting code can look very similar to how Scala implicits are used in practice.

In Scala, an implicit value can be introduced anywhere in the code and can be overridden by another implicit value in an inner scope. The Scala compiler applies a set of rules to determine which implicit value takes precedence depending on where the code is called, what is imported, and what other implicits a given implicit value depends on. These rules are notoriously complex and have been a persistent source of confusion among Scala developers.

Incoherent Rust could face the same complexity. The `with` keyword for introducing capabilities and the scoped impl mechanism for overriding incoherent trait implementations would both allow bindings to be introduced at arbitrary scopes. This could lead to situations where the choice of implementation changes depending on subtle details about import order or scope nesting, making it difficult to reason about what code is actually being called.

In practice, the flexibility of Scala implicits has led to code that is hard to read and even harder to debug. Changing an import or reordering definitions can silently change which implicit is selected, and tracing the resolution chain through multiple layers of delegation is often non-trivial. These issues became significant enough that the Scala community debated the feature at length, eventually leading to a redesign in Scala 3 with the introduction of `given` and `using` as more explicit alternatives.

In contrast, CGP is considerably more constrained. It does not support the same level of dynamic scoping as Scala implicits. All provider configurations must be wired explicitly into a concrete context, and the context type itself is part of the static type of every expression that depends on it. This means that for any given context type, it is always clear which provider is being used without needing to trace through scope rules.

Given these considerations, it is worth asking whether Incoherent Rust should restrict the flexibility of scoped impls and capabilities to something closer to CGP's model, where all bindings are visible and non-overlapping within a given context. Doing so would avoid the ambiguity resolution problems that have made Scala implicits controversial, while still enabling the core use cases that motivate Incoherent Rust in the first place.

---

## Conclusion

Throughout this blog post, we have explored how CGP relates to the recent discussions around dictionary-passing style and Incoherent Rust. Let us summarize what we have learned.

### What CGP helps solve

#### An implementation strategy for Incoherent Rust

The most direct takeaway is that CGP already provides a working implementation strategy for many of the ideas discussed in Boxy's blog post on Incoherent Rust. By using an explicit generic `Context` type as the `Self` parameter of provider traits, CGP allows multiple overlapping trait implementations to coexist without violating Rust's coherence rules. Named implementations become regular Rust provider structs, and the wiring of those implementations is deferred to a concrete context type using `delegate_components!`.

This means that a significant subset of what Incoherent Rust proposes can already be expressed today in stable Rust, using CGP as the underlying mechanism. If the Rust compiler team is considering how to implement Incoherent Rust, CGP's approach of desugaring incoherent traits to coherent provider traits with an explicit context parameter is worth considering as an implementation strategy. It avoids the need to introduce dictionary-passing style at the compiler level, and instead relies on the existing trait system to do the heavy lifting.

#### An implementation strategy for context and capabilities

CGP's support for implicit arguments and field-carrying context types also provides a natural implementation strategy for Tyler's context and capabilities proposal. Rather than threading runtime values through an ambient implicit scope, CGP passes capabilities as fields in a concrete context struct, and provider implementations retrieve those values through `HasField` and implicit arguments.

As shown in the desugaring of the `Deserialize` with arena example, this approach handles the core use case of capabilities cleanly. The context struct acts as a typed, locally scoped carrier for both trait implementations and runtime values, and provider code can access those values without needing to know the concrete context type in advance.

#### An alternative implementation strategy to dictionary-passing style

As we have explored, desugaring Incoherent Rust into dictionary-passing style in TIR comes with significant challenges that may require many advanced language features to be added to Rust. Even though TIR is technically an IR and thus has fewer restrictions than the surface Rust language, it may still face challenges that require as much complexity as adding some of these features to surface Rust itself.

In comparison, CGP's approach provides a viable alternative implementation strategy for Incoherent Rust that can be implemented in stable Rust today, without requiring significant changes to the existing trait system or type checker.

### What CGP partially solves

#### Ecosystem evolution

Boxy's blog post makes a compelling case that the orphan rules prevent healthy ecosystem evolution, particularly around foundational traits like `Serialize`. CGP partially addresses this problem by allowing incoherent traits to be defined from scratch with an explicit context parameter. What CGP cannot do is retroactively fix existing traits in the ecosystem that were designed without one.

For traits that are already widely used, such as `Serialize` or `Hash`, adopting CGP's approach would require breaking changes to those trait definitions. That is not a realistic path in the short term, which means that CGP provides a good model for how new incoherent traits should be designed in the future, but it does not address the ecosystem fragmentation that already exists today.

If CGP were used as the implementation strategy behind Incoherent Rust, the situation would be somewhat better. In that case, programmers would not need to include an explicit context parameter in their traits or in code that uses those traits, because the compiler would handle that transformation internally. However, a trait like `Hash` or `Serialize` would still need to opt into incoherence by some means, such as by adding an `incoherent` keyword, and it is an open question whether that could be done without introducing a breaking change.

That question is a separate challenge that applies regardless of whether CGP or dictionary-passing style is used as the underlying implementation strategy. Before committing to either approach, the Rust community would need to decide how existing traits can evolve toward incoherence without fracturing the code that currently depends on them.

### What CGP doesn't solve

#### Formalization of Rust

The primary motivation for the dictionary-passing style work described in Nadri's blog posts is to create a formal model of Rust's trait system that can be used to prove soundness and avoid compiler bugs. CGP does not contribute to this goal. CGP is a library-level pattern built on top of the existing trait system. It relies on coherence to function correctly, and it cannot be used to replace or formalize the trait solver itself.

For the purpose of building a formal model of Rust, dictionary-passing style in TIR remains the right direction. The challenges we discussed, such as higher-ranked types, dependent types, and type equality axioms, are real obstacles that need to be addressed independently of what CGP can offer.

#### Fully dynamic-scoped impls and capabilities

As discussed in the limitations section, CGP's single-context approach requires all bindings of implementations and runtime values to be specified at one place, when the concrete context type is defined. This makes it difficult to support the kind of nested dynamic-scoped bindings that Incoherent Rust and context and capabilities envision, where a caller can locally override a trait implementation for the duration of a call.

Supporting this kind of dynamic scoping would require either a form of context inheritance, or a mechanism for threading modified contexts through the call stack without explicit propagation. Neither of these is supported by CGP today, and introducing them could significantly complicate the semantics of how contexts are used.

#### Use of mutable or owned values as capabilities

Since CGP contexts are always accessed through a shared `&self` reference, provider implementations cannot receive mutable or owned values directly from the context without interior mutability. This is a limitation that comes from how Rust's borrow checker works, and it affects both CGP and any dictionary-passing style approach that uses a shared context reference.

If Incoherent Rust were to support capabilities backed by `&mut` or owned values, the compiler would need to track ownership and linearity through capability usage, which is a significantly more complex problem than what CGP addresses.

### The future of CGP with an Incoherent Rust

If features like Incoherent Rust and context and capabilities are eventually implemented in Rust, they would likely make parts of the CGP library unnecessary. The boilerplate of defining provider traits, implementing `DelegateComponent`, and wiring components through `delegate_components!` exists precisely because today's Rust does not support incoherent traits natively.

However, the underlying design patterns that CGP promotes, such as writing generic code against a context type, deferring implementation choices to a top-level context, and separating the definition of an interface from its wiring, would remain valuable regardless of how the language evolves. CGP's goal has never been to create a framework that developers depend on indefinitely. The goal is to enable context-generic programming patterns in Rust, and ideally those patterns should eventually become idiomatic Rust that requires no special library support.

In that sense, the discussions happening now around Incoherent Rust and dictionary-passing style are very much aligned with where CGP is headed. The hope is that the design patterns CGP has developed over time can inform how these future language features are designed, and that the two efforts can learn from each other as they move forward.
