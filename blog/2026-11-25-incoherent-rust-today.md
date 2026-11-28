---
authors: [soares]
---

# Trying Incoherent Rust today with Context-Generic Programming

In March, three blog posts started a discussion about traits and coherence in Rust. Nadrieril's [**Elaborating Rust Traits to Explicit Trait Proofs**](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html) and [**What If Traits Carried Values**](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html) read trait impls as values passed through a program, and Boxy's [**An Incoherent Rust**](https://www.boxyuwu.blog/posts/an-incoherent-rust/) used that reading to ask what Rust would look like if traits could opt out of coherence. The discussion has since become a Rust project goal, and Nadrieril and Niko Matsakis have turned to the question of when trait impls may depend on themselves.

[**Context-Generic Programming**](/docs/) (CGP) extends Rust with pluggable trait implementations, chosen per context at compile time, and it runs on stable Rust today. In this post, I explain how CGP relates to the discussion: which parts of Incoherent Rust it can express now, where its context type works like the dictionaries of dictionary-passing style, and where it falls short.

<!--truncate-->

:::note[Author's note]
This post was first drafted in April, after Boxy's and Nadrieril's posts from March, and was put on hold for lack of time and because the work on [CGP v0.8.0](/blog/v0.8.0-release) came first. It was revised before publication to follow the discussion since then.
:::

## Outline

This post first sets out the background and a short introduction to CGP. It then desugars several features of Incoherent Rust into CGP code, compares CGP's context type with the dictionaries of dictionary-passing style, and states the limitations of expressing it this way. The later sections cover what CGP's providers and contexts can do beyond the current designs, the ideas CGP suggests for Incoherent Rust, and how other languages handle the same trade-offs.

In summary, CGP:

- Lets a significant subset of Incoherent Rust be tried on stable Rust today.
- Lets contexts and capabilities be tried through context fields.
- Does not solve the formalization goal, and only partly addresses ecosystem evolution for existing traits.
- Reveals specific challenges that full dictionary-passing style in Rust would need to address.

---

## Background

This section summarizes the discussion as I read it from public writing, so it may miss context that the participants have. The website's [Rust's own proposals](/docs/comparisons/rust-language-proposals) page summarizes each proposal on its own terms.

Nadrieril's first post reads trait solving as building explicit proofs. A trait is like a struct of methods, an impl is like a value of that struct, and a trait bound is how such values are passed along. This way of passing impls is called dictionary-passing style in the Haskell literature. Making the proofs explicit turns each trait choice into a value that can be checked. A footnote in the post connects the idea to his [**Rust via Desugarings**](https://nadrieril.github.io/rust-via-desugarings/) book, which explains Rust by translating it into a simpler subset of the language. His later post on [**a formal specification**](https://nadrieril.github.io/blog/2026/06/16/formal-spec-vision.html) builds on that approach. The language it translates into, **PolyMiniRust**, makes every trait proof explicit: it is Ralf Jung's MiniRust, an interpreter for Rust's runtime semantics, extended with generics and trait proofs.

His second post lets those dictionaries carry runtime values. It credits Tyler Mandry, the author of the 2021 [**contexts and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal, with pointing out that the idea resembles that proposal. Boxy's post then explores how the same model could enable **incoherent traits**, which opt out of Rust's coherence rules. That post starts from the problem of ecosystem evolution, with `serde` as its example, and works through what such traits would mean for `HashMap` and associated types. I use **Incoherent Rust** for this envisioned version of Rust, broadly covering the features that Nadrieril, Boxy, and Tyler Mandry explore, none of which are settled yet.

Since March, the work has become the types team's [**Dictionary Passing Style Experiment**](https://github.com/rust-lang/goals/issues/630), with a [project repository](https://github.com/rust-lang/project-dictionary-passing) whose reading list gives Boxy's post as its entry on incoherence. Nadrieril's [**When can Traits Depend on Themselves?**](https://nadrieril.github.io/blog/2026/05/14/when-can-traits-depend-on-themselves.html) and Niko Matsakis's [**series on cyclic trait impls**](https://smallcultfollowing.com/babysteps/series/cyclic-trait-impls/), which takes perfect derive as its motivating case, study which cycles between trait impls are sound, a question the section on [context as dictionary](#context-as-dictionary) takes up again. Nadrieril's other recent post, [**Capabilities using Plain Traits**](https://nadrieril.github.io/blog/2026/06/07/capabilities-with-plain-traits.html), expresses `cfg` and target features as traits, including *selfless* traits without a `Self` type, which the section on [zero-arity traits](#zero-arity-traits) comes back to.

These discussions now run along three lines. The types team's experiment asks whether rustc can compute each trait proof once, during type checking, instead of solving the same bounds again during borrow checking and code generation. Nadrieril's vision for a formal specification translates Rust into PolyMiniRust, where every trait proof is explicit, and the project has since accepted a related goal for an [**executable specification**](https://goals.rust-lang.org/2026/executable-spec.html), built on a-mir-formality and MiniRust. And the feature posts show what explicit proofs would enable, such as incoherent traits and traits that carry values.

The three need not go together. The compiler might compute proofs once without ever exposing incoherent traits, and a feature like Incoherent Rust could be designed before the compiler supports it. This post therefore treats them separately, so that how feasible one is does not decide how we think about another.

---

## A brief introduction to CGP

This section introduces CGP for readers who already know Rust's traits, generics, and coherence rules in depth, such as members of the Rust compiler and types teams.

CGP extends Rust with pluggable trait implementations: code is written generically over a context type, which takes the place of `Self` in trait definitions, and each context chooses which implementation it uses. CGP does this with design patterns that work within Rust's coherence rules, so multiple overlapping implementations can coexist on stable Rust. A consumer trait remains an ordinary trait, which can still be implemented directly.

This section covers only what the rest of the post needs. My RustLab 2025 talk, [**How to stop fighting with coherence and start writing context-generic trait impls**](/blog/rustlab-2025-coherence), explains the motivation in more depth. The website's [coherence](/docs/concepts/coherence), [consumer and provider traits](/docs/concepts/consumer-and-provider-traits), and [modularity hierarchy](/docs/concepts/modularity-hierarchy) pages explain the mechanism, and [`cargo cgp expand`](/docs/tooling/cargo-cgp/expand) shows the code each macro generates.

### Bypassing coherence

CGP bypasses coherence restrictions at two levels.

The first level lets a trait have multiple overlapping implementations, through a layer of indirection called a provider trait. The [`#[cgp_component]`](/docs/reference/macros/cgp_component) macro turns a trait into a component with its own provider trait, and each implementation becomes a named provider: a struct that is the `Self` type of its impl, so the impls cannot conflict with each other. A concrete context then chooses one provider per trait, one line each, through [`delegate_components!`](/docs/reference/macros/delegate_components), which turns the wiring into a type-level lookup table, written as impls of the [`DelegateComponent`](/docs/reference/traits/wiring/delegate_component) trait. The [Bypassing coherence](/docs/concepts/coherence) page walks through this level with an encoder example, and [Type-level tables](/docs/concepts/type-level-tables) explains how the compiler looks the table up.

This first level bypasses the overlap rule, but not the orphan rule: the wiring must still live in a crate that owns either the context or the trait. So this level alone does not solve the ecosystem evolution problem that [**Incoherent Rust**](https://www.boxyuwu.blog/posts/an-incoherent-rust/) addresses. As my [**RustLab presentation**](/blog/rustlab-2025-coherence) explains, though, CGP has a more advanced pattern that handles overlapping and orphan implementations together: a new type parameter becomes the context, in the `Self` position, and the original `Self` type moves into a generic parameter of the consumer trait.

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

Compared to serde's definition, this version uses an additional `Context` type as `Self` and moves the original `Self` type to the generic `T` parameter. A `Display`-based provider, written with [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), then takes the value as an argument:

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

Implementations of this `Serialize` are wired on a central context type, which does not need to live in the same crate as the value type or the trait. Here the value type is `Person`, a struct with first and last name fields that implements `Display`:

```rust
// inside a third party crate
pub struct MyApp;

delegate_components! {
    MyApp {
        // Choose a provider per value of the component's generic parameter
        open SerializeImplComponent;

        // Path notation that names the generic parameter as part of the
        // component path.
        @SerializeImplComponent.Person: SerializeWithDisplay,
    }
}
```

The [`open` statement](/docs/reference/macros/delegate_components#choosing-a-provider-per-type-the-open-statement) lets the context choose a provider for each value of the component's generic parameter, and each `@`-path key names one such value.

With this approach, third-party crates can define their own contexts like `MyApp`, and locally rewire the `Serialize` implementation for a value type without affecting global coherence. The [modularity hierarchy](/docs/concepts/modularity-hierarchy#tier-4-one-provider-per-target-type) page explains this pattern in more depth, as one tier among the ways CGP can arrange contexts and providers.

The added context type only partly solves the orphan rules problem, though. Every trait involved must adopt CGP and include the context parameter in its definition, so the approach cannot be applied to existing traits like `serde::Serialize` without breaking changes across the entire Rust ecosystem. It suits new traits well, since they can take the context parameter from the start without breaking anything.

### CGP as a library or as language sugar

CGP is currently a library on stable Rust. Behind it is a small set of design patterns, which the website's [Concepts pages](/docs/concepts/) explain. The patterns are deliberately minimal, and they work within the existing rules of Rust's type system rather than requiring changes to it.

The sections below desugar Incoherent Rust into CGP, but I am not suggesting that the compiler should implement Incoherent Rust this way. If rustc computes explicit trait proofs, a named impl is already a value the compiler can pass around. Translating it first into CGP's provider traits and wiring, and then elaborating those traits into proofs, would only add a detour through the coherence rules that incoherent traits set out to relax.

What CGP offers instead is a place to try these ideas now. The desugarings below are ordinary CGP code that compiles today, so Rust programmers can write code in the style Incoherent Rust envisions without waiting for incoherent traits to reach the language. The experience from real CGP projects can then feed into the design questions that Boxy's post leaves open, such as syntax, ergonomics, and whether the complexity is worth it.

Trying the ideas through CGP has its costs, though. CGP's wiring adds compile-time work, readers have to learn the context and provider model, and errors that pass through generated providers are long in their raw form. For those errors, the project ships an error toolchain, cargo-cgp: [`cargo cgp check`](/docs/tooling/cargo-cgp/check) leads with the root cause for the classes it recognizes, and the tool does not yet reshape every class. A compiler that understood incoherent traits natively could report such errors in its own terms.

---

## Expressing incoherence with CGP

With these concepts in place, we can look at how the features that Boxy and Nadrieril propose can be expressed in CGP.

### Incoherent traits

In [An Incoherent Rust](https://www.boxyuwu.blog/posts/an-incoherent-rust/#incoherent-traits), Boxy proposes introducing an `incoherent` keyword on traits to allow overlapping implementations. For example:

```rust title="Incoherent Rust"
// crate a
pub incoherent trait Serialize {
    fn serialize(&self) -> String;
}
```

With CGP, an incoherent trait like this desugars into a component whose `Self` type is a generic context:

```rust title="CGP"
// crate a
#[cgp_component(SerializeImpl)]
pub trait Serialize<T> {
    fn serialize(&self, value: &T) -> String;
}
```

The original `Self` type, which represents the value being serialized, becomes the explicit generic parameter `T`. The new `Self` position is now the context type, which carries the implementation choices and lets multiple overlapping implementations coexist without violating coherence.

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

Because the `Self` type of each provider trait implementation is the provider struct itself, which is defined in the local crate, the two implementations do not conflict under coherence even when both target the same value type `b::Matrix`.

### Using incoherent trait bounds

Uses of incoherent trait bounds desugar too. Boxy's post gives this example, which calls the same function with two implementations of the same trait:

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

In CGP, the same code becomes:

```rust title="CGP"
// Needed for `core::any::type_name` in a constant; see below.
#![feature(const_type_name)]

#[cgp_component(NameImpl)]
pub trait Name<T> {
    const NAME: &'static str;
}

#[cgp_impl(new DummyName)]
impl<T> NameImpl<T> {
    const NAME: &'static str = "dummy";
}

#[cgp_impl(new RealName)]
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
                NameImplComponent: RealName,
            }
        }

        // prints "i32"
        Context.do_stuff(&foo);
    }
}
```

In the desugared code, the incoherent `Name` trait becomes a CGP component `Name<T>`, with the provider trait `NameImpl`. The generic function `do_stuff` becomes a context-generic CGP function that uses `Name<T>` through the [`#[uses]`](/docs/reference/attributes/uses) attribute and accepts `&self` as the implicit context parameter. Each anonymous call site in the original code, such as `MyImpl<_ + DummyName<_>>`, desugars into a locally scoped `Context` struct with a `delegate_components!` block that wires the chosen provider.

The constant stays an associated constant, since a CGP component can declare one just as an ordinary trait does. Only `RealName`'s constant needs care: it calls `core::any::type_name`, which is not yet stable as a `const fn`. That is why the snippet opens with `#![feature(const_type_name)]` and needs a nightly compiler, as do the later examples that reuse `Name`. The requirement comes from the standard library, not from CGP.

`do_stuff` takes `&self` because the context type, `Self`, carries the implementation choice at the type level. As we will see later, the same `&self` can also carry runtime values, which makes context-generic functions suitable for expressing contexts and capabilities.

### Binding an implementation to a struct

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

Because the `Context` type is part of the struct's type signature, a value of type `Foo<ContextA, i32>` and a value of type `Foo<ContextB, i32>` are distinct types. This distinction is the property Boxy describes: once a struct is created with a particular incoherent implementation bound to it, that binding cannot be silently swapped out at another call site.

This property is especially useful for the hash table problem that Boxy's post raises. A hash map can include the `Context` type in its definition, becoming `HashMap<Context, Key, Value>`, to ensure that the same `Hash` implementation is always used when inserting or looking up entries.

One downside of binding the full `Context` type to a struct is that two contexts with identical provider wirings are still different types. The difference gets in the way when you want to share a struct value across modules or functions that use different contexts.

An alternative is to bind a specific provider type to the struct rather than a full context. The provider type becomes a generic parameter of the struct, and a CGP function binds it the way a [higher-order provider](/docs/concepts/higher-order-providers#a-provider-carried-by-a-value) binds its inner provider:

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

In this version, the struct carries the provider type directly, so the choice of `Name` implementation is locked to `RealName` regardless of which context is used to call `do_stuff`. This provider binding is more flexible than the full-context binding, because any context can interact with the struct without needing to match the exact context type that created it.

The trade-off is that this approach only fixes the binding for one specific trait. If multiple incoherent traits are involved and you want each one to be bound independently, the struct would need a separate provider parameter for each trait. Depending on how many traits are involved, those parameters can become verbose. The full-context approach described earlier handles several traits more uniformly, at the cost of tying the struct to a single concrete context type.

---

## Context as dictionary

In the desugared CGP code of the previous sections, the additional generic `Context` type works in a similar way to the dictionaries described in Nadrieril's post on [**Elaborating Rust Traits to Explicit Trait Proofs**](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html).

That post rewrites Rust code in dictionary-passing style: a trait becomes a struct of function pointers, each impl becomes a constant of that struct, and a trait bound becomes a const generic argument that passes one of those constants along.

### Context-aware dictionaries

Suppose that we want to combine the earlier ideas and make `Clone` an incoherent trait. Nadrieril's post writes a `Clone` example in made-up notation for explicit trait proofs, which names each impl and passes it explicitly:

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

Following the earlier sections, the CGP translation is:

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

Set beside Nadrieril's version, the CGP code shows its `Context` type acting as a type-level dictionary, with `delegate_components!` declaring its entries. Each entry plays the part of one of Nadrieril's dictionaries, and the context holds them all, so CGP already follows the programming model of dictionary-passing style closely. If the CGP code were rewritten in dictionary-passing style, the context would become a value-level dictionary instead, holding the other dictionaries as fields, nearly one for each `delegate_components!` entry.

Every trait dictionary in that form would also take the context as an extra parameter, and each implementation would look up the others through it. The compiler would then need no fixed order for instantiating dictionaries or passing them to each other, because the context would be a single stable root from which every dependency can be reached. Nadrieril's dictionary-passing example fixes its dictionaries at compile time, passing each one as a const generic argument. Passing the context as a value at runtime instead would be a different design, one that would allow a form of dynamic scoping, in which the implementation of a trait depends on which context value is in scope at the call site.

Whether those dependencies may form cycles is a separate question, the one that Nadrieril's cycles post and Niko Matsakis's series study, and CGP stops short of it. CGP's lookups are trait obligations, so a provider that needs the very instance it is defining overflows the trait solver with `E0275`. A serializer for a linked list that serializes its tail through the context needs exactly that, and so does an expression interpreter that dispatches on its own expression type. Today's workaround is a provider written for the concrete context, which costs one hand-written impl per context and per operation; [Rust's own proposals](/docs/comparisons/rust-language-proposals#the-context-as-a-dictionary) explains the limit and the workaround. So recursive providers are a concrete use case for the cyclic impls that Nadrieril and Niko Matsakis study. The linked list has the shape of perfect derive, the motivating example in Niko Matsakis's series, and sound cyclic impls of that kind could remove the workaround.

### Contexts and capabilities

Nadrieril's follow-up blog post, [**What If Traits Carried Values**](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html), explores how Tyler Mandry's [**contexts and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal could be implemented using dictionary-passing style. The motivating example is passing a `BasicArena` value into an implementation of `Deserialize`:

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

As my [`cgp-serde` blog post](/blog/cgp-serde-release) explains, CGP's incoherent traits already pass runtime values alongside trait implementations through the `Context` type. CGP's [**implicit arguments**](/docs/concepts/implicit-arguments) make the values in the context's fields easier to read.

With them, the example above desugars into this CGP code:

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
        // Its field already holds a reference, which the argument borrows as `&&`.
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

Passing the context through `&self` pays off here: the same value that selects the implementations also carries the arena they need. cgp-serde's [context services](/docs/projects/cgp-serde/architecture/context-services) page explains how a deserializer takes a service such as an arena from its context.

### Zero-arity traits

As the section on [incoherent traits](#incoherent-traits) showed, CGP desugars an incoherent trait into a CGP trait whose `Self` is the context, moving the original `Self` type into a generic parameter, as `Serialize<T>` does. CGP code also commonly defines traits with no extra generic parameter, such as:

```rust title="CGP"
#[cgp_component(GreetImpl)]
pub trait Greet {
    fn greet(&self);
}
```

These traits are what Nadrieril, in [**Capabilities using Plain Traits**](https://nadrieril.github.io/blog/2026/06/07/capabilities-with-plain-traits.html), calls *selfless traits*: traits without a `Self` type, which he writes with a `#[selfless]` attribute. His selfless traits state facts about the environment, such as `Unix`, and he notes that a capability is "just a ZST context", calling his capability traits "a bit of an effect system". The platform fixes those facts, while an incoherent selfless trait would leave its implementation to the context. In Incoherent Rust, the `Greet` trait above would be such a zero-arity trait. The Incoherent Rust snippets in this section are my own sketches, combining Boxy's named impls with Nadrieril's `#[selfless]` attribute:

```rust title="Incoherent Rust"
#[selfless]
pub incoherent trait Greet {
    fn greet(); // no use of `self` or `Self`
}
```

Incoherence and capabilities together suggest a useful pattern: zero-arity incoherent traits whose implementations work only with the ambient context. For example, the `Greet` trait could be implemented with:

```rust title="Incoherent Rust"
impl GreetHello = Greet
with
    name: &String,
{
    fn greet() {
        println!("Hello, {}!", name);
    }
}
```

With CGP, since the context type is explicit, zero-arity incoherent traits become unary CGP traits, which need no new syntax:

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

---

## Limitations and open challenges

Passing every dependency through one generic `Context` type has several advantages. It keeps the desugaring simple, and it makes the semantics of Incoherent Rust code easy to reason about: the context maintains a locally scoped coherence, so all code that uses the same `Context` type receives the same trait implementation.

Because the context serves as a top-level dictionary for the other trait dictionaries, it also simplifies how they are instantiated with their dependencies, though not when those dependencies form cycles, as the [context as dictionary](#context-as-dictionary) section explains.

That said, several limitations may prevent the single-context approach from working in the general case.

### Mutable or owned value access

The biggest limitation is that a single context cannot easily supply mutable or owned values to trait implementations. The desugared trait methods always take `&self`, and mutable or owned values cannot be taken out of a borrowed context.

Changing the parameter to `&mut self` or `self` does not help either. The change would spread to every caller, forcing all methods to take `&mut self` or `self`, and it would significantly limit the order in which methods can be called, especially methods that return references borrowed from other parameters.

Nadrieril's post also notes that bounds whose dictionaries carry such values would need ownership semantics: a dictionary that carries a `&mut` must not be passed to two functions at once, and its methods would implement `FnMut` or `FnOnce` rather than plain function pointers.

A simple way around the problem, which Nadrieril also proposes, is to limit capabilities to `Copy` values and use interior mutability. After all, `RefCell`, `Arc<Mutex>`, and `Clone` already let code mutate or copy values from a borrowed context.

But if Rust chooses to support `&mut` or owned values as capabilities too, it may need to pass each of them as a separate parameter, rather than grouping them in a single context.

### Nested dynamically scoped bindings

Another key limitation is that CGP binds every runtime value and trait implementation in one place, where the concrete context type is defined. Binding everything in one place can severely limit where an incoherent trait implementation can be called.

In **What If Traits Carried Values**, Nadrieril presents **scoped impls**: a trait impl may be defined anywhere, and uses of the trait within its scope take that local impl, overriding any global or outer one.

For example, a user may want to introduce new bindings freely in different functions, rather than in one place:

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
    deserialize_and_print_foo(json_string);
}
```

With a single context, changing its bindings according to where it is called would be very hard. The compiler would have to choose one place to define the context, such as inside `deserialize_and_print_foo`. That anonymous context would then be out of reach of `deserialize_and_print_upper_foo`, which could not override the implementation of `Deserialize` for `String`.

Supporting such scoped bindings through a single context type might lead to something that resembles **inheritance** in OOP: to modify the trait bindings at different scopes, the context would need some kind of inheritance relationship with a parent context.

I point out the resemblance not to advocate OOP-style inheritance in Rust, but to show that supporting this kind of binding may bring inheritance-like behavior into Rust code.

CGP already has a static form of this inheritance for wiring: a [namespace](/docs/concepts/namespaces) lets a context inherit a shared table, but a context [cannot replace a binding it inherits](/docs/concepts/namespaces#overriding-a-bound-entry), and scoped impls need exactly that override.

Readers who know Scala may notice that this kind of code strongly resembles Scala's [implicit parameters](/docs/comparisons/implicit-parameters), which Rust might not want to support. CGP's implicit arguments differ from them: every implicit argument must come from one flattened context type.

These limitations raise the question of whether Rust should let users bind capabilities or incoherent trait implementations anywhere in the code. Such code might desugar into dictionary-passing style in other ways, but those ways may make Incoherent Rust code much harder to reason about.

On the other hand, the restricted Incoherent Rust that CGP provides may encourage more disciplined code. Because every binding is made where a context type is defined, a reader can usually find the implementations a context uses in one place, which keeps the semantics of context-generic code within a specific context relatively simple.

---

## Where CGP diverges from Incoherent Rust

So far, this post has covered how CGP can express Incoherent Rust. This section turns to the other direction: what CGP's providers and contexts can do that the current designs of Incoherent Rust do not support directly. The dictionary model behind those designs can express several of these ideas, since its dictionaries can take other dictionaries and hold types, so I offer them in the hope that Incoherent Rust will support what CGP users already rely on. The Incoherent Rust snippets in this section are my own sketches of how the current designs might extend, not syntax from the proposals.

### Named impls as provider types

The most notable difference is that Boxy's and Nadrieril's posts treat named implementations as a different kind of entity from ordinary Rust types. Given named impls like:

```rust title="Incoherent Rust"
impl Impl1 = Clone for MyType { ... }
impl Impl2 = Clone for MyType { ... }
```

the implementations `Impl1` and `Impl2` are not Rust types. They can be passed where a trait bound is expected, but they cannot stand where a type does, such as in a type alias or a type-level list.

In contrast, CGP's named implementations, called providers, are ordinary Rust types that compose with other Rust types.

#### Multiple implementations on the same name

Among other things, a CGP provider can implement several provider traits. In Incoherent Rust, the equivalent might look like:

```rust title="Incoherent Rust"
impl MyMatrixSerde = Serialize for Matrix { ... }

// How to disambiguate intentional name sharing vs accidental naming?
impl MyMatrixSerde = Deserialize for Matrix { ... }
```

Here `MyMatrixSerde` is one named implementation of both `Serialize` and `Deserialize` for `Matrix`. In CGP, it becomes:

```rust title="CGP"
// A provider is just a struct
pub struct MyMatrixSerde;

#[cgp_impl(MyMatrixSerde)]
impl SerializeImpl<Matrix> { ... }

#[cgp_impl(MyMatrixSerde)]
impl DeserializeImpl<Matrix> { ... }
```

#### Grouping named impls

Because CGP's named impls are types, related implementations can be grouped under a single name, which CGP calls an [aggregate provider](/docs/concepts/aggregate-providers). For example, we might want to group the `Serialize` and `Deserialize` implementations of all domain types defined across different modules.

With Incoherent Rust, we might need additional syntax to support aliasing, such as:

```rust title="Incoherent Rust"
impl MyAppSerde = Serialize for Matrix = MyMatrixSerialize;
impl MyAppSerde = Deserialize for Matrix = MyMatrixDeserialize;
impl MyAppSerde = Serialize for Scalar = MyScalarSerialize;
impl MyAppSerde = Deserialize for Scalar = MyScalarDeserialize;
```

CGP has dedicated notation for such grouping: `delegate_components!` with the `new` keyword declares the aggregate provider and its table in one block, and an `open` statement inside it keys each entry by component and value type, as in `@SerializeImplComponent.Matrix: MyMatrixSerialize`.

#### Higher-order providers

CGP also supports [higher-order providers](/docs/concepts/higher-order-providers), where a provider explicitly accepts other providers as generic parameters. A higher-order provider binds its inner providers directly, rather than through the context as other providers do.

For example, a `SerializeIterator` named impl in Incoherent Rust might look like:

```rust title="Incoherent Rust"
impl<T, ItemSerializer> SerializeIterator<ItemSerializer> =
    Serialize for T
where
    T: IntoIterator,
    ItemSerializer: Serialize for T::Item,
{ ... }
```

Here the named impl `SerializeIterator` takes an inner named impl, `ItemSerializer`, which serializes the `<T as IntoIterator>::Item` type.

In CGP, `SerializeIterator<ItemSerializer>` is a plain struct with the inner provider as a phantom type parameter, and its implementation binds that parameter with `#[use_provider]`, as the higher-order providers page shows with a scaling wrapper.

#### Type-level DSLs

Since CGP's named implementations are just types, higher-order providers also enable design patterns that go beyond what the current sketches of Incoherent Rust describe. [Hypershell](/docs/projects/hypershell/), for example, uses higher-order providers to build a type-level DSL for shell scripting in Rust, a pattern the [type-level DSLs](/docs/concepts/type-level-dsls) page explains.

After desugaring, a Hypershell program is a plain Rust type: the shell command `echo hello world!` becomes a `Pipe` of higher-order providers that names the command, its arguments as type-level strings, and where its output goes. If Incoherent Rust treats named implementations differently from Rust types, such type-level DSLs would be much more difficult to build from named impls.

### Contexts as explicit types

Another key distinction is that CGP contexts are explicit Rust types, so a context can be used like any other Rust type, even inside a larger context. The capability designs instead assume an implicit ambient context that carries impls and values from callers to callees, which makes them more restrictive when implementing traits for concrete types.

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

With Incoherent Rust, it might be possible to implement something like `query_user` through named impls and capabilities, but each binding would be explicit, so developers may need more code to bridge into concrete contexts:

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

Because CGP works with concrete Rust types, an explicit context type avoids the manual bridging that ambient capabilities would require.

---

## Ideas from CGP for Incoherent Rust

The rest of CGP's divergences are ideas the current designs could take up: functions that defer their implementation choices and types chosen by the context.

### Incoherent functions

The CGP functions annotated with [`#[cgp_fn]`](/docs/reference/macros/cgp_fn), seen earlier, suggest a new concept that I call **incoherent functions**: functions that use an incoherent trait without knowing which implementation is chosen, leaving that choice to a coherent caller.

Take the `do_stuff` CGP function from earlier:

```rust title="CGP"
#[cgp_fn]
#[uses(Name<T>)]
pub fn do_stuff<T>(&self, foo: &Foo<T>) {
    println!("{}", Self::NAME);
}
```

It can be read as equivalent to this Incoherent Rust function:

```rust title="Incoherent Rust"
pub incoherent fn do_stuff<T>(foo: &Foo<T>)
where
    impl NameImpl: Name<T>,
{
    println!("{}", NameImpl::NAME);
}
```

Other incoherent functions can then call `do_stuff` without restating its `where` bound:

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

So incoherent functions can call each other without spelling out every transitive incoherent trait bound of the functions they call. All those bounds are gathered and resolved at once, when a coherent function calls into the incoherent world.

Together, incoherent functions set up an incoherent world, isolated from the coherent world where all existing Rust code lives. As with async functions, the split colors functions: incoherent traits and functions can freely call coherent or incoherent ones, but coherent code cannot call incoherent code without explicitly binding every incoherent trait implementation it requires.

This separation between the incoherent and coherent worlds is essential, because it simplifies the semantics and creates a clear boundary for where the bindings for incoherent implementations can be specified.

In CGP, the incoherent world is any code that works with a generic context. Each `#[cgp_fn]` function desugars into a trait with a blanket implementation over the context, as [Impl-side dependencies](/docs/concepts/impl-side-dependencies) explains, and those impl-side dependencies hide each function's use of other CGP functions behind its trait's interface. The boundary between the two worlds lies where a top-level context type is defined. Once that type is defined, every incoherent binding is final, and we are back in the coherent world, which needs no generic context.

As noted earlier, this clean separation holds only without dynamically scoped impls and capabilities. A full version of Incoherent Rust may not keep such a clear boundary, and its semantics would then differ considerably from CGP's.

For readers familiar with monads and algebraic effects, the incoherent world of a generic context can be understood as a form of effect, or more precisely as a coeffect, which describes what a computation requires from its environment. The [algebraic effects comparison](/docs/comparisons/algebraic-effects#coeffects) develops that reading.

### Abstract types

Associated types within [zero-arity traits](#zero-arity-traits) give CGP's [abstract types](/docs/concepts/abstract-types). For example, we can introduce an abstract type called `Name` with the following trait:

```rust title="CGP"
#[cgp_type]
pub trait HasNameType {
    type Name: Display;
}
```

An abstract type is not tied to any particular value type. The context chooses its concrete form. The earlier `GreetHello` implementation can use the abstract `Name` type instead:

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

The `name` parameter now has the abstract type `HasNameType::Name` instead of a hardcoded `String`, so the same provider works with whichever name type the context wires in.

A context then chooses the concrete type in its wiring through `UseType`, such as a `FullName` struct with first and last name fields, as the abstract types page shows.

#### Abstract types in Incoherent Rust

In Incoherent Rust, abstract types would start from `HasNameType` as a zero-arity trait:

```rust title="Incoherent Rust"
#[selfless]
pub incoherent trait HasNameType {
    type Name: Display;
}
```

Since abstract types are built on top of zero-arity traits, using them requires new syntax to bind and reference the abstract type:

```rust title="Incoherent Rust"
impl GreetHello = Greet
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

Here, `GreetHello` requires the context to provide an implementation of `HasNameType` and binds it to the local identifier `NameType`. The abstract type is then referenced as `NameType::Name` when specifying the type of the `name` capability.

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

Here, `UseFullNameType` is chosen as the incoherent implementation for `HasNameType`, and the capability `name` is bound to a `FullName` value. When `GreetHello` is called, both the type implementation and the name value are provided through the trait system, and the greeting is printed.

#### Using abstract types across incoherent traits

The `Name` example leaves open why an associated type on a zero-arity trait is better than one on an ordinary trait. Consider a `touch` function that sets the `last_modified` field of any type that supports it. With plain Rust traits, the time type either becomes an extra generic parameter, which still ties `touch` to `SystemTime::now()`, or becomes an associated type of `UpdateLastModified`, which gives every value type its own time type and clock, so swapping in a mock clock for a test means changing each of them.

With zero-arity traits and Incoherent Rust, we can write the following instead:

```rust title="Incoherent Rust"
// An abstract type trait is a zero-arity trait with an associated type
#[selfless]
pub incoherent trait HasTimeType {
    type Time;
}

// The current time is provided by the ambient context using a zero-arity trait
#[selfless]
pub incoherent trait CurrentTime
where
    impl TimeImpl: HasTimeType,
{
    fn current_time() -> TimeImpl::Time;
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

With zero-arity traits, implementations need not be bound to any particular `Self` type or generic parameter. When two value types, such as `Folder` and `File`, both need `touch`, they can share one time implementation through the ambient context instead of each carrying its own.

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

CGP converts zero-arity traits into single-arity traits that are bound to the generic context, while the original `Self` type from the Incoherent Rust version becomes an explicit generic parameter `T`. With this structure, traits like `HasTimeType` and `CurrentTime` are bound to the context rather than to any particular value type. The example shows the benefit of anchoring abstract types to the context: several value types share one interchangeable choice, and none of them carries its own copy.

---

## Related work

Other languages have already given up global coherence, each in its own way. Cairo infers named impls from those in scope at each call site, Scala resolves `given` instances by scope and priority, OCaml's modular implicits choose module arguments from scope and reject ambiguity, and Haskell offers overlapping and incoherent instances as exceptions to global uniqueness. The website's [traits without coherence](/docs/comparisons/traits-without-coherence) page compares these designs with CGP, and its [ML modules](/docs/comparisons/ml-modules) and [implicit parameters](/docs/comparisons/implicit-parameters) pages cover OCaml and Scala in more depth.

### Lessons for Rust

The designs that give up global coherence share three problems. Call sites can resolve the same type differently, a choice can change with what a module imports, and a data structure such as a hash map needs one implementation used consistently. They answer these problems in different ways. Haskell keeps one answer for the whole program and relaxes it only by exception. Cairo, Scala, and modular implicits infer the choice from what is in scope. Boxy's sketch names it explicitly at each call, and CGP centralizes it on a context type that is defined once.

A single global answer needs no wiring at all, but where Haskell's overlap extensions apply, GHC's manual warns that different parts of a program can choose different instances. Inference from scope is convenient in small programs, but it ties the choice to imports. Scala, for one, has reworked its mechanism: Scala 3 separates `given` definitions from `using` parameters, where Scala 2 wrote `implicit` for both. Explicit naming is precise, but it repeats the choice at every call. CGP's centralized context is more verbose up front, yet every call through a context uses the same choice, and a reader finds that choice in one `delegate_components!` block. If Incoherent Rust adopts scoped or inferred impls, a context-like mechanism that fixes the choices for a whole program, alongside the lower-level machinery, is worth considering too.

---

## Conclusion

This post has explored how CGP relates to the discussion of dictionary-passing style and Incoherent Rust. The sections below sum up where CGP helps, where it helps only partly, and where it does not help.

### What CGP helps solve

#### A testing ground for Incoherent Rust

The most direct takeaway is that CGP already lets programmers try, on stable Rust, many of the ideas in Boxy's blog post: overlapping implementations become named providers, and a concrete context chooses among them through `delegate_components!`. CGP's design has been refined across several releases and used in projects such as cgp-serde and Hypershell, so its patterns have already met real code. CGP does not show how the compiler should implement these features: with explicit trait proofs, the compiler could desugar named impls into proofs directly, without going through CGP's traits. What CGP can show is how code written in this style feels in practice, and which features real programs come to need.

#### Trying contexts and capabilities today

CGP's implicit arguments and field-carrying contexts also provide a way to try Tyler Mandry's contexts and capabilities proposal today. Rather than threading runtime values through an ambient implicit scope, CGP passes them as fields of a concrete context, which providers read through [`HasField`](/docs/reference/traits/field-access/has_field) and implicit arguments. As the arena-backed `Deserialize` example shows, this covers the core use case of capabilities.

### What CGP partially solves

#### Ecosystem evolution

Boxy's blog post shows how the orphan rules hold back ecosystem evolution around foundational traits, with serde's `Serialize` as the example. CGP partially addresses this problem by allowing incoherent traits to be defined from scratch with an explicit context parameter. What CGP cannot do is retroactively fix existing traits in the ecosystem that were designed without one.

For traits that are already widely used, such as `Serialize` or `Hash`, adopting CGP's approach would require breaking changes to those trait definitions. Such breaking changes are not a realistic path in the short term. CGP offers a model for designing new incoherent traits, but it does not address the ecosystem fragmentation that already exists.

A language feature would do better here. Programmers would not need to add a context parameter to their traits, because the compiler would handle incoherent traits itself. A trait like `Hash` or `Serialize` would still need to opt into incoherence somehow, such as with an `incoherent` keyword, and whether that opt-in could avoid a breaking change remains an open question.

That question stands however the compiler implements incoherent traits. Before committing to a design, the Rust community would need to decide how existing traits can evolve toward incoherence without fracturing the code that depends on them.

### What CGP doesn't solve

#### Formalization of Rust

The dictionary-passing work began with soundness: computing each trait proof once, checking it, and giving Rust's trait system a formal footing that can help avoid compiler bugs. CGP does not contribute to that goal. It is a library-level pattern built on the existing trait system, it relies on coherence to function correctly, and it cannot replace or formalize the trait solver itself.

Explicit trait proofs, in the compiler and in a specification language such as Nadrieril's PolyMiniRust, remain the right direction for that work, and CGP code would benefit from them like any other Rust code.

#### Dynamic scoping and mutable values

The two [limitations](#limitations-and-open-challenges) described earlier also remain unsolved. CGP binds every implementation and value where the concrete context is defined, so it cannot express the nested, dynamically scoped bindings that scoped impls envision, and its shared `&self` context cannot hand providers mutable or owned values without interior mutability.

### The future of CGP with an Incoherent Rust

If features like Incoherent Rust and contexts and capabilities are eventually implemented in Rust, they would likely make parts of the CGP library unnecessary. The boilerplate of defining provider traits, implementing `DelegateComponent`, and wiring components through `delegate_components!` exists precisely because today's Rust does not support incoherent traits natively.

However, the underlying design patterns that CGP promotes, such as writing generic code against a context type, deferring implementation choices to a top-level context, and separating the definition of an interface from its wiring, would remain valuable regardless of how the language evolves. CGP's goal has never been to create a framework that developers depend on indefinitely. The goal is to enable context-generic programming patterns in Rust, and ideally those patterns should eventually become idiomatic Rust that requires no special library support.

Several of CGP's features go beyond the current sketches of Incoherent Rust, such as providers that can be named and composed as types, incoherent functions, and abstract types chosen by a context. I would like to work with the Rust project on the design of these language features, so that Incoherent Rust can support what CGP users rely on today. If it does not, CGP will keep its library encoding for those features, on top of the coherent trait system, beside the language features that cover the rest.

If the Rust project adopts explicit trait proofs, and nightly Rust lets programs write them directly, I would like to experiment with desugaring CGP code into them. The experiment would be independent of Incoherent Rust, not as a rival to it, but because explicit proofs suit CGP whatever design Incoherent Rust settles on. CGP's macros could then emit named impls and explicit proofs in place of provider traits and wiring tables. The experiment would also test whether recursive providers, which today's trait solver rejects, become expressible.

The current discussions of Incoherent Rust and dictionary-passing style point the same way CGP is headed, and I hope the two efforts keep learning from each other.

I am grateful to Nadrieril, Boxy, Tyler Mandry, and Niko Matsakis for writing these ideas down in public, and to the types team for turning them into project work. This post reads their designs from the outside, so I may have misread some of them. If I have, I would like to hear about it.
