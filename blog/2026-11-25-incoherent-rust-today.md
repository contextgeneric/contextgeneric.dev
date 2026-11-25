---
authors: [soares]
---

# Using an incoherent, dictionary-passing style Rust today

In March, three blog posts started a discussion about traits and coherence in Rust. Nadrieril's [**Elaborating Rust Traits to Explicit Trait Proofs**](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html) and [**What If Traits Carried Values**](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html) read trait impls as values passed through a program, and Boxy's [**An Incoherent Rust**](https://www.boxyuwu.blog/posts/an-incoherent-rust/) used that reading to ask what Rust would look like if traits could opt out of coherence. The discussion has since become a Rust project goal, and Nadrieril and Niko Matsakis have turned to the question of when trait impls may depend on themselves.

[**Context-Generic Programming**](/docs/) (CGP) extends Rust with pluggable trait implementations, chosen per context at compile time, and it runs on stable Rust today. In this post, I explain how CGP relates to the discussion: which parts of Incoherent Rust it can express now, where its context type works like the dictionaries of dictionary-passing style, and where it falls short.

<!--truncate-->

## Outline

This post first sets out the background and a short introduction to CGP. It then desugars several features of Incoherent Rust into CGP code, compares CGP's context type with the dictionaries of dictionary-passing style, and states the limitations of that approach. The later sections cover what CGP adds to named impls, the ideas CGP suggests for Incoherent Rust, and how other languages handle the same trade-offs.

In summary, CGP:

- Provides a working implementation strategy for a significant subset of Incoherent Rust today.
- Offers a natural desugaring path for context and capabilities.
- Does not solve the formalization goal, and only partly addresses ecosystem evolution for existing traits.
- Reveals specific challenges that full dictionary-passing style in Rust would need to address.

---

## Background

This section covers the background as I understand it from publicly available writing. I will likely miss some details that would give a more complete picture, so if something here looks wrong, it may come from gaps in my background. The website's [Rust's own proposals](/docs/comparisons/rust-language-proposals) page summarizes each proposal on its own terms.

Nadrieril's first post reads trait solving as building explicit proofs. A trait is like a struct of methods, an impl is like a value of that struct, and a trait bound is how such values are passed along, which the Haskell literature calls dictionary-passing style. The post belongs to his [**Rust via Desugarings**](https://nadrieril.github.io/rust-via-desugarings/) effort to define Rust by translating it into a smaller language, and his later post on [**a formal specification**](https://nadrieril.github.io/blog/2026/06/16/formal-spec-vision.html) places explicit trait proofs inside that plan. In this post, I call the trait-free target of such a translation **Traitless IR (TIR)**, though Nadrieril's plan names its own target PolyMiniRust.

:::note
Similar to other Rust IRs like HIR, THIR, and MIR, Rust programmers won't be writing TIR programs directly. Technically, there won't be any concrete syntax for TIR, but we will use a Rust-like syntax for TIR to better illustrate the concepts and facilitate the discussion.
:::

His second post lets those dictionaries carry runtime values, which would make Tyler Mandry's [**contexts and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal much easier to express. Boxy's post then explores how the same model could enable **incoherent traits**, which opt out of Rust's coherence rules. I use **Incoherent Rust** for this envisioned version of Rust, broadly covering the features that Nadrieril, Boxy, and Tyler Mandry explore, none of which are settled yet.

Since March, the work has become the types team's [**Dictionary Passing Style Experiment**](https://github.com/rust-lang/goals/issues/630), with a [project repository](https://github.com/rust-lang/project-dictionary-passing) whose reading list gives Boxy's post as its entry on incoherence. Nadrieril's [**When can Traits Depend on Themselves?**](https://nadrieril.github.io/blog/2026/05/14/when-can-traits-depend-on-themselves.html) and Niko Matsakis's [**series on cyclic trait impls**](https://smallcultfollowing.com/babysteps/series/cyclic-trait-impls/) study which cycles between trait impls are sound, a question the section on [context as dictionary](#context-as-dictionary) comes back to. Nadrieril's other recent post, [**Capabilities using Plain Traits**](https://nadrieril.github.io/blog/2026/06/07/capabilities-with-plain-traits.html), expresses `cfg` and target features as trait facts about one concrete type, which is further from CGP's use of a context.

There are multiple objectives being explored across these discussions. The original motivation for TIR was to formalize and improve the correctness of the Rust compiler. It was then discovered that using dictionary-passing style in TIR could also make it easier to implement higher level features such as Incoherent Rust and context and capabilities.

That said, these two objectives are not necessarily tied together. For example, it might be possible to implement TIR without using it to add new features like Incoherent Rust. Similarly, dictionary-passing style and TIR are not the only implementation strategies available to enable features like Incoherent Rust.

In this blog post, I am analyzing the two objectives as separate cases, so that the feasibility of one does not affect how we think about the other.

---

## A brief introduction to CGP

This section gives a brief introduction to CGP, written with Rust compiler team members in mind. The target audience is assumed to already be deeply familiar with advanced Rust concepts like traits, generics, and the coherence rules.

CGP extends Rust with pluggable trait implementations: code is written generically over a context type, which takes the place of `Self` in trait definitions, and each context chooses which implementation it uses. CGP does this with design patterns that work within Rust's coherence rules, so multiple overlapping implementations can coexist on stable Rust.

This section covers only what the rest of the post needs. My RustLab 2025 talk, [**How to stop fighting with coherence and start writing context-generic trait impls**](/blog/rustlab-2025-coherence), explains the motivation in more depth. The website's [coherence](/docs/concepts/coherence), [consumer and provider traits](/docs/concepts/consumer-and-provider-traits), and [modularity hierarchy](/docs/concepts/modularity-hierarchy) pages explain the mechanism, and [`cargo cgp expand`](/docs/tooling/cargo-cgp/expand) shows the code each macro generates.

### Bypassing overlapping implementations

CGP bypasses coherence restrictions at two levels.

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
#[uses(Display)]
impl HashImpl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(&self.to_string(), state)
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

Behind the scenes, the `delegate_components!` macro translates the code into a type-level lookup table through the `DelegateComponent` trait. CGP then uses blanket implementations generated by `#[cgp_component]` to perform a lookup at compile time and implement `Hash` for `Person` using `HashWithDisplay`. Any number of overlapping providers can be defined this way, and each context selects the ones it reuses, one line per trait.

### Bypassing orphan implementations

This first level bypasses the overlap rule, but not the orphan rule. To implement a trait like `Hash`, the `delegate_components!` call must still reside in a crate that owns either `Person` or the trait.

In other words, this alone does not fully solve the ecosystem evolution problem that [**Incoherent Rust**](https://www.boxyuwu.blog/posts/an-incoherent-rust/) is trying to address. However, as explained in my [**RustLab presentation**](/blog/rustlab-2025-coherence), CGP offers a more advanced design pattern that addresses both overlapping and orphan implementations at the same time. This is done by adding an additional type parameter as the main context type, and shifting the original `Self` type into a generic parameter of the consumer trait.

For example, a fully incoherent version of serde's `Serialize` would be defined as follows:

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

Compared to serde's definition, this version uses an additional `Context` type as `Self`, and shifts the original `Self` type to the generic `T` parameter. A `Display`-based provider then takes the value as an argument:

```rust
#[cgp_impl(new SerializeWithDisplay)]
impl<T: Display> SerializeImpl<T> {
    fn serialize<S: Serializer>(
        &self,
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&value.to_string(), serializer)
    }
}
```

This version of `Serialize` makes it possible to wire up implementations through a centralized context type that does not need to reside in the same crate as the value type or the trait. For example:

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

The `open` statement and the `@`-path keys arrived in CGP v0.8.0. Earlier releases wrote the same nested lookup with `UseDelegate`, which is harder to read.

With this approach, third party crates can define their own contexts like `MyApp`, and locally rewire the `Serialize` implementation for a value type without affecting global coherence.

#### Does CGP truly solve the orphan rules problem?

CGP manages to bypass Rust's coherence restriction by adding an extra context type parameter at the `Self` position. This approach only partially solves the orphan rules problem, because it requires all involved traits to adopt CGP and include the extra context parameter in their definition.

This means it is not possible to apply CGP's approach to existing traits like `serde::Serialize` without introducing breaking changes across the entire Rust ecosystem. However, CGP's approach is well suited for defining new traits, since the generic context type can be added from the start without breaking anything.

### CGP as a library or as language sugar

CGP is currently implemented as a library on top of stable Rust. The core concepts behind it are a small set of design patterns, which the website's [Concepts pages](/docs/concepts/) explain. These patterns are deliberately minimal, and they work within the existing rules of the Rust type system rather than requiring any changes to it.

When I describe a strategy for desugaring Incoherent Rust into CGP, I am not suggesting that Rust should adopt CGP as a library dependency. That would be the wrong framing. What I am proposing instead is that Rust could adopt the underlying design patterns that CGP has developed, and promote them into first-class language features. Those features could then serve as the internal mechanism for implementing the syntactic sugar that Incoherent Rust would expose to programmers.

Only a small number of concrete constructs would need to find a home in the language or standard library. For example, the `DelegateComponent` trait provides the type-level lookup table that the rest of the system depends on. These constructs are intentionally simple, and adding them to the standard library would not introduce meaningful complexity to the language.

Adopting the patterns would bring their costs into the language as well. CGP's wiring adds compile-time work, readers have to learn the context and provider model, and errors that pass through generated providers are long in their raw form. CGP ships [`cargo cgp check`](/docs/tooling/cargo-cgp/check) for those errors: `cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet reshape every class. A compiler that understood incoherent traits natively could report such errors in its own terms.

The larger portion of the CGP library consists of procedural macros. These macros handle the ergonomics of writing CGP code by performing syntactic transformations, such as moving the `Context` type into the provider trait, generating blanket implementations, and producing component name structs. If Rust were to adopt these features natively, the compiler would simply perform the same transformations that the proc macros perform today. The desugaring logic would move from a library crate into the compiler itself, but the underlying rules would remain the same.

The practical consequence of this approach is that Incoherent Rust would become the surface syntax that programmers interact with, while CGP would recede into the implementation details of the compiler. Rust programmers would learn about incoherent traits, named implementations, and context-and-capability bindings as language features, with no need to know that CGP patterns are what makes them work under the hood. The library would have served its purpose by proving out the design, and the language would carry it forward.

---

## Implementing incoherence with CGP

With the basic CGP concepts introduced, we can now look at how we could implement Incoherent Rust by desugaring it into CGP code.

The main reason to explore this desugaring is that CGP is implemented as a library on top of stable Rust. This means that any CGP code is already supported by the language today. If we can desugar the new Rust features proposed by Boxy and Nadrieril into CGP, we could effectively implement those features as syntactic sugar without needing to change the core Rust compiler.

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
impl RealName<T> = Name for T {
    const NAME: &'static str = core::any::type_name::<T>();
}

#[derive(Copy, Clone)]
struct Foo<T>(T);
impl MyImpl<T: Name> = Foo<T> {
    pub fn do_stuff(self) {
        println!("{}", <T as Name>::NAME);
    }
}

fn main() {
    let foo = Foo(1);

    // prints "dummy"
    MyImpl<_ + DummyName<_>>::do_stuff(foo);
    // prints "i32"
    MyImpl<_ + RealName<_>>::do_stuff(foo);
}
```

The same code can be desugared into CGP as follows:

```rust title="CGP"
#[cgp_component(NameImpl)]
pub trait Name<T> {
    fn name() -> &'static str;
}

#[cgp_impl(new DummyName)]
impl<T> NameImpl<T> {
    fn name() -> &'static str {
        "dummy"
    }
}

#[cgp_impl(new RealName)]
impl<T> NameImpl<T> {
    fn name() -> &'static str {
        core::any::type_name::<T>()
    }
}

pub struct Foo<T>(pub T);

#[cgp_fn]
#[uses(Name<T>)]
pub fn do_stuff<T>(&self, foo: &Foo<T>) {
    println!("{}", Self::name());
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
                NameImplComponent: RealName,
            }
        }

        // prints "i32"
        Context.do_stuff(&foo);
    }
}
```

A few things are worth noting about the desugared code. The incoherent `Name` trait becomes a CGP component `Name<T>` with a corresponding provider trait `NameImpl`. Its constant becomes an associated function, because `core::any::type_name` is not yet stable as a `const fn`. The generic function `do_stuff` becomes a context-generic CGP function that uses `Name<T>` through the `#[uses]` attribute and accepts `&self` as the implicit context parameter. Each anonymous call site in the original code, such as `MyImpl<_ + DummyName<_>>`, desugars into a locally scoped `Context` struct with a `delegate_components!` block that wires the chosen provider.

The reason `do_stuff` accepts a `&self` argument is that the `Self` context type is what carries the implementation choice at the type level. As we will see later, this `&self` can also carry runtime values, which is what makes context-generic functions suitable for implementing context and capabilities.

### Use of incoherent trait bounds within a struct

Boxy's blog post also discusses binding an incoherent trait implementation directly to a struct, so that the same implementation is always used whenever that struct is passed around:

```rust title="Incoherent Rust"
#[derive(Copy, Clone)]
struct Foo<T: Name>(T);
impl MyImpl<T: Name> = Foo<T> {
    ...
}

fn main() {
    let foo = Foo::<_ + DummyName<_>>(1)

    // prints "dummy"
    MyImpl<_ + DummyName<_>>::do_stuff(foo);

    // errors as `foo` has type `Foo<u8 + DummyName<u8>>` but
    // the impl requires the self type to be `Foo<u8 + RealName<u8>>`
    MyImpl<_ + RealName<_>>::do_stuff(foo);
}
```

With CGP, this pattern desugars by adding the `Context` type as a parameter on the struct itself, so that the struct carries the implementation choice in its type:

```rust title="CGP"
pub struct Foo<Context: ?Sized, T> {
    pub value: T,
    pub phantom: PhantomData<Context>,
}

#[cgp_fn]
#[uses(Name<T>)]
pub fn do_stuff<T>(&self, foo: &Foo<Self, T>) {
    println!("{}", Self::name());
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
        NameImplComponent: RealName,
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
    println!("{}", NameProvider::name());
}

fn main() {
    pub struct ContextA;
    pub struct ContextB;

    let foo: Foo<RealName, i32> = Foo {
        value: 1,
        phantom: PhantomData,
    };

    // Both contexts can call do_stuff on the same foo value,
    // because the Name implementation is fixed by RealName, not by the context.
    ContextA.do_stuff(&foo);
    ContextB.do_stuff(&foo);
}
```

In this version, the struct carries the provider type directly, so the choice of `Name` implementation is locked to `RealName` regardless of which context is used to call `do_stuff`. This is more flexible than the full-context binding, because any context can interact with the struct without needing to match the exact context type that created it.

The trade-off is that this approach only fixes the binding for one specific trait. If multiple incoherent traits are involved and you want each one to be bound independently, the struct would need a separate provider parameter for each trait. Depending on how many traits are involved, that can become verbose. The full-context approach described earlier handles this more uniformly, at the cost of tying the struct to a single concrete context type.

---

## Context as dictionary

If you have been following the desugared CGP code for Incoherent Rust in the previous sections, you may have noticed that the additional generic `Context` type introduced by CGP works in a similar way to the dictionaries described in Nadrieril's post on [**Elaborating Rust Traits to Explicit Trait Proofs**](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html).

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

// I'm imagining generic consts because why not. See how the trait bound becomes a
// const generic argument.
const CLONE_VEC<T, const CLONE_T: Clone<T>>: Clone<Vec<T>> = Clone {
    clone: |vec: &Vec<T>| -> Vec<T> { // Dummy clone impl, it's probably not even that bad
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

Suppose that we want to combine the earlier ideas and make `Clone` an incoherent trait. Nadrieril's post writes the same idea in made-up notation for explicit trait proofs, which names each impl and passes it explicitly:

```rust title="Explicit trait proofs"
trait Clone {
    fn clone(&self) -> Self;
}

// This syntax gives a name to the impl.
impl "clone_u32" Clone for u32 { ... }

// This takes two arguments: the explicit `T`, and the implicit `T: Clone`.
impl<T> "clone_vec" Clone for Vec<T>
where
    clone_t: (T: Clone) // weird syntax don't @ me
{
    fn clone(&self) -> Self {
        vec
            .iter()
            // We use the trait bound explicitly here
            .map(|x: &T| clone_t::clone(x))
            .collect()
    }
}

let my_vec: Vec<u32> = vec![0, 1, 2];
// square brackets is my syntax for passing implicit params:
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

Another important property of this translation is that all trait dictionaries accept both the `Context` type and a `context` value as additional parameters. Because each implementation looks up the others through the context at runtime, the compiler does not need a fixed order for instantiating dictionaries or passing them as arguments to each other. The context acts as a single stable root from which all dependencies can be reached. Whether those dependencies may form cycles is a separate question, and it is the one Nadrieril's cycles post and Niko Matsakis's series study.

This runtime lookup also opens the door to a form of dynamic scoping, where the implementation used for a trait can vary depending on which context value is in scope at the call site.

CGP stops short of that question. Its lookups are trait obligations, so a dependency that leads back to the instance being proved overflows the trait solver with `E0275`. Recursion through a different instance works: the `CloneVec` provider above needs `Clone<T>` for the element type, and that chain ends because the type is finite. A provider for a recursive type is different. A serializer for a linked list that serializes its tail through the context needs the very instance it is defining, and an expression interpreter that dispatches on its own expression type does too.

Today's workaround is a provider written for the concrete context, without the bound that leads back to itself, as the [dispatch layers page](/docs/projects/cgp-examples/expression/architecture/dispatch-layers) explains. The compiler accepts it, most likely because it can check the impl from its header alone, but the workaround costs one hand-written impl per context and per operation. So recursive providers are a concrete use case for the cyclic impls that work studies. The linked list has the shape of perfect derive, the motivating example in Niko Matsakis's series, and sound cyclic impls of that kind could remove the workaround.

### Context and capabilities

Nadrieril's follow-up blog post, [**What If Traits Carried Values**](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html), explores how we could implement Tyler Mandry's [**contexts and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal using dictionary-passing style. The motivating example is passing a `BasicArena` value into an implementation of `Deserialize`:

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

As explained in my [`cgp-serde` blog post](/blog/cgp-serde-release), CGP incoherent traits already support passing runtime values alongside trait implementations through the `Context` type. The **implicit arguments** that CGP added in v0.7.0 make those field values considerably easier to access.

With that, the example above can be desugared into the following CGP code:

```rust title="CGP"
// cgp-serde's deserialization component, with the target `Value` in place of `Self`:
//
//     #[cgp_component(ValueDeserializer)]
//     pub trait CanDeserializeValue<'de, Value> {
//         fn deserialize<D>(&self, deserializer: D) -> Result<Value, D::Error>
//         where
//             D: serde::Deserializer<'de>;
//     }

#[cgp_impl(new DeserializeFooRef)]
#[uses(CanDeserializeValue<'de, Foo>)]
impl<'de, 'a> ValueDeserializer<'de, &'a Foo> {
    fn deserialize<D>(
        &self,
        // The arena is "passed" as an implicit argument through the context.
        // The double reference (&&) arises because implicit arguments always
        // borrow from `&self`, and the field's type is itself already a reference.
        #[implicit] arena: &&'a Arena<Foo>,
        deserializer: D,
    ) -> Result<&'a Foo, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let foo = self.deserialize(deserializer)?;
        Ok(arena.alloc(foo))
    }
}

// The `Context` type now contains fields instead of being an empty struct.
#[derive(HasField)]
pub struct Context<'a> {
    pub arena: &'a Arena<Foo>,
}

delegate_components! {
    <'a> Context<'a> {
        open ValueDeserializerComponent;

        ErrorTypeProviderComponent:
            UseAnyhowError,
        ErrorRaiserComponent:
            RaiseAnyhowError,
        @ValueDeserializerComponent.u64:
            UseSerde,
        @ValueDeserializerComponent.Foo:
            DeserializeRecordFields,
        @ValueDeserializerComponent.<'b> &'b Foo:
            DeserializeFooRef,
        // A context-generic provider from cgp-serde that implements
        // deserialization using the `Extend` method of the container.
        @ValueDeserializerComponent.<'b> Vec<&'b Foo>:
            DeserializeExtend,
    }
}

fn main() -> Result<(), anyhow::Error> {
    let json = read_some_json()?;
    let arena = Arena::new();

    // The `with` binding becomes a field assignment to a local `context` value.
    let context = Context { arena: &arena };

    // `deserialize_json_string` is a method from cgp-serde-json that wraps
    // `serde_json` deserialization in a context-generic way.
    let foos: Vec<&Foo> = context.deserialize_json_string(&json)?;
    println!("foos: {:?}", foos);

    Ok(())
}
```

The advantage of passing an additional context through `&self` becomes clear once we consider that the `Context` type can carry runtime values alongside trait implementations.

---

## Limitations and open challenges

CGP's single-context approach to passing dependencies via a generic `Context` type provides several advantages. It makes the desugaring process simple and straightforward to understand, and the TIR code is visually similar to the CGP code. The use of a single `Context` type also makes it easy to reason about the semantics of the Incoherent Rust code. Essentially, the `Context` type maintains a locally scoped coherence, which ensures that all code that uses the same `Context` type will receive the same trait implementation.

By using the `Context` type as a top-level dictionary for other trait dictionaries, it also simplifies the resolution of how to instantiate trait dictionaries with their dependencies, though not of cyclic ones, as the [context as dictionary](#context-as-dictionary) section explains.

That said, there are several limitations that may prevent the single-context approach from working in the general case.

### Mutable or owned value access

The biggest limitation of the single-context approach is that the `Context` type is not able to easily supply mutable or owned values to trait implementations. Since the trait methods are desugared to always accept a `&self` parameter, we cannot get mutable or owned values out of the borrowed context value.

Furthermore, we cannot change the parameter to accept a `&mut self` or owned `self`. Doing so would virally affect all other code that attempts to call the method, essentially forcing all methods to accept `&mut self` or `self`. The use of `&mut self` or `self` also significantly limits the ability to call methods in any order, especially if the method also returns references that are borrowed from other parameters.

In Nadrieril's blog post, it is also mentioned that when capabilities containing `&mut` or owned values are used, the Rust compiler would have to keep track of the ownership and their use in trait implementations, and prevent multiple instantiation of trait implementations that may violate the ownership rules.

A simple way to work around this is to simply disallow the use of `&mut` or owned values as capabilities. After all, there are workarounds such as using `RefCell`, `Arc<Mutex>`, and `Clone` to mutate or copy values from a borrowed context.

However, in case the Rust compiler team decides to also support `&mut` or owned values as capabilities, then it may be necessary to explicitly pass everything as separate parameters, without grouping them through a single context.

### Nested dynamic-scoped bindings

Another key limitation of CGP's approach is that all bindings of runtime values and trait implementations must be done at one place where the concrete context type is defined. This can severely limit the expressiveness of where one can call an incoherent trait implementation.

In the blog post **What If Traits Carried Values**, Nadrieril presented the idea of **scoped impls**, where a trait impl may be defined anywhere, and the use of the trait within that scope would use the given local impl, which overrides any global or outer impls.

For example, a user may want to freely introduce new bindings across different function calls, instead of limiting them into one place:

```rust title="Incoherent Rust"
fn deserialize_and_print_foo(json_string: &str) {
    let foo: Vec<&Foo> =
        with arena = &arena::BasicArena::new() {
            serde_json::deserialize(json_string).unwrap()
        };

    println!("foos: {:?}", foo);
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

CGP v0.8.0 already has a static form of this inheritance for wiring. A [namespace](/docs/concepts/namespaces) lets a context inherit a shared table and fill in the entries it leaves unbound, but a context cannot replace an inherited binding. CGP rules out that override on purpose, and scoped impls need exactly that override.

Readers coming from Scala may also notice that this kind of code strongly resembles implicit parameters in Scala, which might not be what we want to support in Rust. CGP's implicit arguments are different from implicit parameters in Scala, in that all implicit arguments must be supplied in a flattened context type.

Due to these limitations, it may be worth considering whether it is worth allowing the user to bind capabilities or incoherent trait implementations anywhere in the code. Although it may be possible to desugar such code into dictionary-passing style in other ways, it may significantly complicate how we could reason about Incoherent Rust code.

On the other hand, the restricted version of Incoherent Rust provided by CGP may result in more disciplined Incoherent Rust code being written. Because all bindings have to be done at the same place as where a `Context` type is defined, CGP code usually has centralized locations for the reader to find out which implementations are used for that context. This results in a relatively simple semantics for reasoning about how context-generic code would behave within a specific context type.

---

## Where CGP diverges from Incoherent Rust

So far, this post has covered how CGP can implement Incoherent Rust. This section turns to the other direction: what CGP's providers and contexts can do that the current designs of Incoherent Rust do not support directly.

### Named impls as provider types

The most notable difference is that the named implementations in Boxy's and Nadrieril's blog posts are treated as a different kind of entity than normal types in Rust. Given named impls like:

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

Since named impls in CGP are just types, it becomes much easier to group related implementations together so they can be referred to by a single name, which CGP calls an [aggregate provider](/docs/concepts/aggregate-providers). For example, we might want to group the `Serialize` and `Deserialize` implementations of all domain types defined across different modules.

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
    new MyAppSerde {
        open { SerializeImplComponent, DeserializeImplComponent };

        @SerializeImplComponent.Matrix:
            MyMatrixSerialize,
        @DeserializeImplComponent.Matrix:
            MyMatrixDeserialize,
        @SerializeImplComponent.Scalar:
            MyScalarSerialize,
        @DeserializeImplComponent.Scalar:
            MyScalarDeserialize,
    }
}
```

#### Higher-order providers

CGP also supports [higher-order providers](/docs/concepts/higher-order-providers), where a provider explicitly accepts other providers as generic parameters. A higher-order provider binds its inner providers directly, rather than through the context as other providers do.

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

Since CGP's named implementations are just types, higher-order providers also enable design patterns that go beyond what Incoherent Rust currently supports. [Hypershell](/docs/projects/hypershell/), for example, uses higher-order providers to build a type-level DSL for shell scripting in Rust, a pattern the [type-level DSLs](/docs/concepts/type-level-dsls) page explains.

After desugaring, the DSL code becomes simple Rust types that look like:

```rust title="CGP"
// A Hypershell program is just a type containing higher-order providers
pub type Program = Pipe<Product![
    SimpleExec<
        StaticArg<Symbol!("echo")>,
        WithStaticArgs<Product![
            Symbol!("hello"),
            Symbol!("world!"),
        ]>,
    >,
    StreamToStdout,
]>;
```

Which corresponds to the shell command `echo hello world!`. If Incoherent Rust treats named implementations differently from regular Rust types, it would be much more difficult to implement such type-level DSLs through named impls.

### Contexts as explicit types

Another key distinction is that CGP contexts are explicit Rust types. This means we can use the context as a regular Rust type inside a larger context. Incoherent Rust, on the other hand, assumes an implicit ambient context that propagates named impls from callers to callees. This makes it more restrictive to use Incoherent Rust when implementing traits for concrete types.

For example, in CGP it is common to define an application context that contains both provider wirings and runtime fields, together with additional constructors for building that context:

```rust title="CGP"
#[derive(HasField)]
pub struct App {
    // Runtime values accessible by providers
    pub database: Database,
    pub s3_client: S3Client,
    pub redis_cache: redis::Client,
    ...
}

delegate_components! {
    App {
        // Use higher-order provider to provide caching middlewares
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

## Ideas from CGP for Incoherent Rust

The rest of CGP's divergences are ideas the current designs could take up: functions that defer their implementation choices, traits without a `Self` type, and types chosen by the context.

### Incoherent functions

One notable CGP feature that we have seen earlier is CGP functions annotated with `#[cgp_fn]`, which can be considered as a new concept that I call **incoherent functions**.

The core idea behind incoherent functions is that they can use an incoherent trait without knowing which implementation is actually chosen. Instead, the choice of implementation is deferred to a coherent caller.

For example, with the `do_stuff` CGP function from earlier:

```rust title="CGP"
#[cgp_fn]
#[uses(Name<T>)]
pub fn do_stuff<T>(&self, foo: &Foo<T>) {
    println!("{}", Self::name());
}
```

We can think of it as being equivalent to the following incoherent Rust function:

```rust title="Incoherent Rust"
pub incoherent fn do_stuff<T>(foo: &Foo<T>)
where
    impl NameImpl: Name<T>,
{
    println!("{}", Name::<T>::name());
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
    self.do_stuff(foo);
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
        println!("{}", Self::name());
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
        self.do_stuff(foo);
    }
}
```

As we can see, incoherent functions desugar into traits that have blanket implementations over a generic context. By making use of impl-side dependencies, we can hide the dependencies of other CGP functions behind the trait interface. In CGP, the boundary between the incoherent and coherent worlds is determined by where a top-level context type is defined. Once that context type is defined, all incoherent implementation bindings are finalized, and we return to the coherent world that does not require the use of a generic context.

As mentioned earlier, this clean separation between the incoherent and coherent worlds is only possible in the absence of dynamic-scoped impls and capabilities. A full version of Incoherent Rust may not preserve such a clear boundary, which would lead to quite different semantics compared to CGP.

#### Incoherence as coeffect

For readers who are familiar with the concept of monads and algebraic effects, the incoherent world with generic context can be understood as a form of effect, or more precisely, a [**coeffect**](https://tomasp.net/coeffects/).

In CGP, each incoherent trait definition can be thought of as defining a new kind of effect, and each CGP provider implementation can be thought of as an effect handler for that effect. The definition of concrete contexts and the binding of incoherent implementations through `delegate_components!` plays the same role as defining concrete monads and configuring effect handlers in algebraic effects systems found in functional languages like Haskell. The CGP bindings effectively handle the incoherence effects by delegating them to CGP providers, and the result is code that operates coherently within a fixed context, analogous to returning a pure value once all effects have been handled.

That said, the effect system implied by CGP is considerably more limited than a full algebraic effects system. CGP does not capture continuations, so the correspondence covers only handlers that resume once, in place, as the [algebraic effects comparison](/docs/comparisons/algebraic-effects) explains.

CGP also departs from algebraic effects in what a handler can supply. Through associated types, a CGP context also chooses *abstract types*, while the effect systems that comparison covers pass operations and values through their handlers, not types.

These observations point toward a better fit in the [**coeffects**](https://tomasp.net/coeffects/) framework, which studies how contextual requirements flow through a computation rather than how computational effects flow out of it. Where effects describe what a computation produces or performs, coeffects describe what a computation requires from its environment. The generic context in CGP serves precisely this role: it carries the implementation choices and values that a computation depends on, and those dependencies are resolved all at once when a concrete context is defined. A full introduction to coeffects is beyond the scope of this blog post, but the connection is worth noting for readers interested in the theoretical foundations of what CGP is doing.

### Zero-arity traits

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
    name: &String,
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
        #[implicit] name: &String,
    ) {
        println!("Hello, {}!", name);
    }
}
```

### Abstract types

Associated types within zero-arity traits give CGP's [abstract types](/docs/concepts/abstract-types). For example, we can introduce an abstract type called `Name` with the following trait:

```rust title="CGP"
#[cgp_type]
pub trait HasNameType {
    type Name: Display;
}
```

The key property of an abstract type is that it is not tied to any particular value type. Its concrete form is chosen by the ambient context. We can modify the earlier `GreetHello` implementation to use the abstract `Name` type:

```rust title="CGP"
#[cgp_impl(new GreetHello)]
#[use_type(HasNameType.Name)]
impl GreetImpl {
    fn greet(
        &self,
        #[implicit] name: &Name,
    ) {
        println!("Hello, {}!", name);
    }
}
```

In this updated implementation, the hardcoded `String` type for the `name` parameter is replaced by the abstract type `HasNameType::Name`. This allows the same provider to be reused across different name types, whichever the context chooses to wire in.

For example, we can define a `FullName` type and use it as the concrete `Name`:

```rust
pub struct FullName {
    pub first_name: String,
    pub last_name: String,
}

impl Display for FullName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}", self.first_name, self.last_name)
    }
}

#[derive(HasField)]
pub struct Context {
    pub name: FullName,
}

delegate_components! {
    Context {
        NameTypeProviderComponent:
            UseType<FullName>,
        GreetImplComponent:
            GreetHello,
    }
}

fn main() {
    let context = Context {
        name: FullName {
            first_name: "John".to_owned(),
            last_name: "Smith".to_owned(),
        },
    };

    // Prints "Hello, John Smith!"
    context.greet();
}
```

The `Context` type wires `FullName` as the concrete `Name` through `NameTypeProviderComponent`, and wires `GreetHello` as the greeting implementation. Because `GreetHello` uses `HasNameType::Name` rather than a hardcoded `String`, it adapts automatically to the `FullName` type without any changes to the provider itself.

#### Abstract types in Incoherent Rust

To see how abstract types might look in Incoherent Rust, we would start by defining `HasNameType` as a zero-arity trait:

```rust title="Incoherent Rust"
pub trait HasNameType for ! {
    type Name: Display;
}
```

Since abstract types are built on top of zero-arity traits, using them requires new syntax to bind and reference the abstract type:

```rust title="Incoherent Rust"
impl GreetHello = Greet for !
where
    impl NameType: HasNameType,
with
    name: &NameType::Name,
{
    fn greet() {
        println!("Hello, {}!", name);
    }
}
```

Here, we require the context to provide an implementation of `HasNameType` and bind it to the local identifier `NameType`. The abstract type is then referenced as `NameType::Name` when specifying the type of the `name` capability.

With the implementation in place, we can set up the wiring and call `GreetHello::greet()` as follows:

```rust title="Incoherent Rust"
impl UseFullNameType = HasNameType {
    type Name = FullName;
}

fn main() {
    with
        UseFullNameType,
        name = FullName {
            first_name: "John".to_owned(),
            last_name: "Smith".to_owned(),
        }
    {
        // Prints "Hello, John Smith!"
        GreetHello::greet();
    }
}
```

Here, we choose `UseFullNameType` as the incoherent implementation for `HasNameType`, and then bind the capability `name` with a `FullName` value. When `GreetHello` is called, both the type implementation and the name value are provided through the trait system, and the greeting is printed.

#### Using abstract types across incoherent traits

The `Name` example leaves open why an associated type on a zero-arity trait is better than one on an ordinary trait. Consider a `touch` function that sets the `last_modified` field of any type that supports it. With plain Rust traits, the time type either becomes an extra generic parameter, which still ties `touch` to `SystemTime::now()`, or becomes an associated type of `UpdateLastModified`, which gives every value type its own time type and clock, so swapping in a mock clock for a test means touching each of them.

With zero-arity traits and Incoherent Rust, we can write the following instead:

```rust title="Incoherent Rust"
// An abstract type trait is a zero-arity trait with an associated type
pub trait HasTimeType for ! {
    type Time;
}

// The current time is provided by the ambient context using a zero-arity trait
pub trait CurrentTime for !
where
    impl TimeImpl: HasTimeType,
{
    fn current_time() -> TimeImpl::Time
}

// The `Time` type does not depend on `Self`
pub trait UpdateLastModified
where
    impl TimeImpl: HasTimeType,
{
    fn update_last_modified(
        &mut self,
        time: TimeImpl::Time,
    );
}

// The time type and current time implementations are not bound to `T`
pub incoherent fn touch<T: UpdateLastModified>(
    value: &mut T,
)
where
    impl TimeImpl: HasTimeType + CurrentTime,
{
    let time = TimeImpl::current_time();
    value.update_last_modified(time);
}
```

Zero-arity traits allow us to define implementations that are not bound to any particular `Self` type or generic parameter. When two different value types, such as `Folder` and `File`, both need `touch` to be called on them, they can share the same time implementation through the ambient context rather than each carrying their own independent copy.

The same example can be expressed in CGP today as:

```rust title="CGP"
#[cgp_type]
pub trait HasTimeType {
    type Time;
}

#[cgp_component(CurrentTimeImpl)]
#[use_type(HasTimeType.Time)]
pub trait CurrentTime {
    fn current_time(&self) -> Time;
}

#[cgp_component(UpdateLastModifiedImpl)]
#[use_type(HasTimeType.Time)]
pub trait UpdateLastModified<T> {
    fn update_last_modified(value: &mut T, time: Time);
}

#[cgp_fn]
#[uses(CurrentTime, UpdateLastModified<T>)]
pub fn touch<T>(&self, value: &mut T) {
    let time = self.current_time();
    Self::update_last_modified(value, time);
}
```

CGP converts zero-arity traits into single-arity traits that are bound to the generic context, while the original `Self` type from the Incoherent Rust version becomes an explicit generic parameter `T`. With this structure, traits like `HasTimeType` and `CurrentTime` are clearly bound to the context rather than to any particular value type. This example makes concrete the benefit of abstract types anchored to the ambient context: they allow multiple value types to share a consistent, interchangeable abstraction without requiring each one to carry its own copy.

---

## Related work

Other languages have already given up global coherence, each in its own way. Cairo infers named impls from those in scope at each call site, Scala resolves `given` instances by scope and priority, OCaml's modular implicits choose module arguments from scope and reject ambiguity, and Haskell offers overlapping and incoherent instances as exceptions to global uniqueness. The website's [traits without coherence](TODO:traits-without-coherence) page compares these designs and Boxy's sketch with CGP, and its [ML modules](/docs/comparisons/ml-modules) and [implicit parameters](/docs/comparisons/implicit-parameters) pages cover OCaml and Scala in more depth.

### Lessons for Rust

The designs that give up global coherence share three problems. Call sites can resolve the same type differently, a choice can change with what a module imports, and a data structure such as a hash map needs one implementation used consistently. They also answer these problems in three ways. Cairo, Scala, and modular implicits infer the choice from what is in scope, Boxy's sketch names it explicitly at each call, and CGP centralizes it on a context type that is defined once.

Inference from scope is convenient in small programs, but it ties the choice to imports. Scala, for one, has reworked its mechanism: Scala 3 separates `given` definitions from `using` parameters, where Scala 2 wrote `implicit` for both. Explicit naming is precise, but it repeats the choice at every call. CGP's centralized context is more verbose up front, yet every call through a context uses the same choice, and a reader finds that choice in one `delegate_components!` block. If Incoherent Rust adopts scoped or inferred impls, it is worth considering whether it should also offer a context-like mechanism that fixes the choices for a whole program, alongside the lower-level machinery.

---

## Conclusion

Throughout this blog post, we have explored how CGP relates to the recent discussions around dictionary-passing style and Incoherent Rust. Let us summarize what we have learned.

### What CGP helps solve

#### An implementation strategy for Incoherent Rust

The most direct takeaway is that CGP already provides a working implementation strategy for many of the ideas discussed in Boxy's blog post on Incoherent Rust. By using an explicit generic `Context` type as the `Self` parameter of provider traits, CGP allows multiple overlapping trait implementations to coexist without violating Rust's coherence rules. Named implementations become regular Rust provider structs, and the wiring of those implementations is deferred to a concrete context type using `delegate_components!`.

This means that a significant subset of what Incoherent Rust proposes can already be expressed today in stable Rust, using CGP as the underlying mechanism. If the Rust compiler team is considering how to implement Incoherent Rust, CGP's approach of desugaring incoherent traits to coherent provider traits with an explicit context parameter is worth considering as an implementation strategy.

#### An implementation strategy for context and capabilities

CGP's support for implicit arguments and field-carrying context types also provides a natural implementation strategy for Tyler Mandry's context and capabilities proposal. Rather than threading runtime values through an ambient implicit scope, CGP passes them as fields in a concrete context struct, and provider implementations retrieve those values through `HasField` and implicit arguments.

As shown in the desugaring of the `Deserialize` with arena example, this approach handles the core use case of capabilities cleanly. The context struct acts as a typed, locally scoped carrier for both trait implementations and runtime values, and provider code can access those values without needing to know the concrete context type in advance.

### What CGP partially solves

#### Ecosystem evolution

Boxy's blog post makes a compelling case that the orphan rules prevent healthy ecosystem evolution, particularly around foundational traits like `Serialize`. CGP partially addresses this problem by allowing incoherent traits to be defined from scratch with an explicit context parameter. What CGP cannot do is retroactively fix existing traits in the ecosystem that were designed without one.

For traits that are already widely used, such as `Serialize` or `Hash`, adopting CGP's approach would require breaking changes to those trait definitions. That is not a realistic path in the short term, which means that CGP provides a good model for how new incoherent traits should be designed in the future, but it does not address the ecosystem fragmentation that already exists today.

If CGP were used as the implementation strategy behind Incoherent Rust, the situation would be somewhat better. In that case, programmers would not need to include an explicit context parameter in their traits or in code that uses those traits, because the compiler would handle that transformation internally. However, a trait like `Hash` or `Serialize` would still need to opt into incoherence by some means, such as by adding an `incoherent` keyword, and it is an open question whether that could be done without introducing a breaking change.

That question is a separate challenge that applies regardless of whether CGP or dictionary-passing style is used as the underlying implementation strategy. Before committing to either approach, the Rust community would need to decide how existing traits can evolve toward incoherence without fracturing the code that currently depends on them.

### What CGP doesn't solve

#### Formalization of Rust

The primary motivation for the dictionary-passing style work described in Nadrieril's blog posts is to create a formal model of Rust's trait system that can be used to prove soundness and avoid compiler bugs. CGP does not contribute to this goal. CGP is a library-level pattern built on top of the existing trait system. It relies on coherence to function correctly, and it cannot be used to replace or formalize the trait solver itself.

For the purpose of building a formal model of Rust, dictionary-passing style in TIR remains the right direction. On the other hand, implementing dictionary-passing style in Rust may require TIR to be implemented with advanced language features such as higher-ranked types, dependent types, and type equality. These are implementation challenges that need to be addressed independently of what CGP can offer.

#### Dynamic scoping and mutable values

The two [limitations](#limitations-and-open-challenges) described earlier also remain unsolved. CGP binds every implementation and value where the concrete context is defined, so it cannot express the nested, dynamically scoped bindings that scoped impls envision, and its shared `&self` context cannot hand providers mutable or owned values without interior mutability.

### The future of CGP with an Incoherent Rust

If features like Incoherent Rust and context and capabilities are eventually implemented in Rust, they would likely make parts of the CGP library unnecessary. The boilerplate of defining provider traits, implementing `DelegateComponent`, and wiring components through `delegate_components!` exists precisely because today's Rust does not support incoherent traits natively.

However, the underlying design patterns that CGP promotes, such as writing generic code against a context type, deferring implementation choices to a top-level context, and separating the definition of an interface from its wiring, would remain valuable regardless of how the language evolves. CGP's goal has never been to create a framework that developers depend on indefinitely. The goal is to enable context-generic programming patterns in Rust, and ideally those patterns should eventually become idiomatic Rust that requires no special library support.

In that sense, the discussions happening now around Incoherent Rust and dictionary-passing style are very much aligned with where CGP is headed. The hope is that the design patterns CGP has developed over time can inform how these future language features are designed, and that the two efforts can learn from each other as they move forward.
