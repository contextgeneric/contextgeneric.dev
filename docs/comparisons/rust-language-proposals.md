---
sidebar_label: "Rust's own proposals"
sidebar_position: 1
description: 'How CGP compares with Rust proposals for specialization, named implementations, dictionary passing, and contexts and capabilities.'
---

# Rust's own proposals

CGP offers explicit implementation selection and access to context values through a
[stable Rust library](/docs/) . It is a language extension with pluggable implementations behind
ordinary Rust traits. This page compares those choices with specialization, named impls, dictionary
passing, and contexts and capabilities, including what each language proposal can express beyond
CGP.

## In your terms

A **context** is the type a CGP method runs on. It supplies data through fields and chooses
implementations through wiring. Often it is a type defined to represent an application.

The proposals' constructs map onto CGP as follows:

| In the proposals | In CGP |
| --- | --- |
| A named impl (`impl Name<T> = Trait<T> for T`) | A **provider**: a zero-sized type that implements a component's provider trait |
| An `incoherent trait` bound passed at a call | A consumer-trait bound on the context, resolved by the context's **wiring** |
| An impl passed as a parameter (`impl TDrop: Drop<T>` in Cairo) | A **[higher-order provider](/docs/reference/glossary#higher-order-provider)**, bound with `#[use_provider]` |
| A capability in a `with` clause | An `#[implicit]` argument read from a context field |
| The `with` block that binds everything | The definition of a context type and its `delegate_components!` table |
| The root dictionary of a dictionary-passing elaboration | The context itself |

## The idea, briefly

The proposals on this page explore changes to trait selection or how values reach generic code.
Rust's coherence rules set the starting point. The *[orphan
rule](/docs/reference/glossary#orphan-rule)* says an `impl<P1..=Pn> Trait<T1..=Tn> for T0` is valid
only if `Trait` is a local trait, or at least one of the types `T0..=Tn` is a local type and no
uncovered type parameter appears before it. The *overlap rule* rejects two implementations that "can
be instantiated with the same type" ([Rust
Reference](https://doc.rust-lang.org/reference/items/implementations.html);
[RFC 2451](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html) ).

These rules keep trait selection consistent across a program. For example, operations on a
`HashMap<K, V>` must agree on the hashing and equality implementations for `K` . The
[type classes](./type-classes.md) page explains the broader reasons for coherence.

### Specialization

RFC 1210 proposes letting two impls overlap when one is *strictly more specific*, with the compiler
choosing the more specific one. Its own example refines a blanket `AddAssign` :

```rust
impl<R, T: Add<R> + Clone> AddAssign<R> for T {
    default fn add_assign(&mut self, rhs: R) {
        let tmp = self.clone() + rhs;
        *self = tmp;
    }
}
```

The `default` keyword marks an item that a more specific impl may override ([RFC
1210](https://rust-lang.github.io/rfcs/1210-impl-specialization.html)). The full feature remains
unstable, with soundness problems involving lifetime information that is erased before code
generation. The standard library uses the more restricted `min_specialization` feature internally
([tracking issue #31844](https://github.com/rust-lang/rust/issues/31844)). On a stable toolchain the
snippet above fails with `E0658` .

Specialization cannot choose between overlapping impls when neither is more specific. For example,
`T: Display` and `T: AsRef<[u8]>` describe intersecting sets of types, but neither set contains the
other. CGP addresses this case by letting the context name its choice. The
[RustLab 2025 talk](/blog/rustlab-2025-coherence) develops the distinction through a hash-table
example.

### Dictionary passing

Dictionary passing models a trait implementation as data supplied to generic code. A trait becomes a
record of methods, an impl supplies that record, and a trait bound becomes an extra argument.
Nadrieril illustrates this model with the following Rust-like pseudocode:

```rust
struct Clone<Self> {
    clone: fn(&Self) -> Self,
}

const CLONE_U32: Clone<u32> = Clone { clone: |x: &u32| *x };
```

Trait resolution supplies the dictionary in this model. Passing dictionaries explicitly would let
callers choose different implementations, while coherence restricts that choice. Nadrieril uses the
translation to examine which restrictions belong to Rust's design and which follow from the
underlying mechanism ([*Dictionary-passing
style*](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html)).

### Named and incoherent impls

Boxy's *An Incoherent Rust* proposes replacing the coherence rules with explicit impl selection. The
motivation is *ecosystem evolution*. The post argues that once a foundational crate such as `serde`
establishes the trait impls for common types, an alternative cannot gain adoption, because
downstream crates cannot implement the competing trait for types they do not own. The sketch names
an impl and passes it where a bound is required:

```rust
impl Name<T> = Trait<T> for T { /* ... */ }

function::<T + TraitImpl<T> + OtherTraitImpl<T>>(/* ... */)
```

The sketch preserves agreement within a `HashMap` by putting its `Hash` and `Eq` choices on the type
definition. It also makes impl identity part of types to distinguish associated-type choices. These
are proposed answers, with syntax, ergonomics, migration, and complexity still open ([Boxy, *An
Incoherent Rust*](https://www.boxyuwu.blog/posts/an-incoherent-rust/)).

### Contexts and capabilities

Tyler Mandry's 2021 proposal addresses a different limitation. A function or impl can reach a value
from its caller's environment, such as an allocator or a logger, only by taking it as a parameter or
reading a global. The proposal adds a `with` clause on a declaration, which names the context value
it requires, and a `with` block at a call site, which supplies it:

```rust
fn deserialize<'a>(bytes: &[u8]) -> Result<&'a Foo, Error>
with
    arena::basic_arena: &'a arena::BasicArena,
{
    arena::basic_arena.alloc(Foo::from_bytes(bytes)?)
}

with arena::basic_arena = &arena::BasicArena::new() {
    let foo: &Foo = deserialize(bytes)?;
}
```

The proposal's distinctive feature is a *context-dependent trait impl*, which is valid only while a
given capability is in scope. The proposed mechanism compiles to ordinary function arguments,
without a runtime lookup. Its open questions concern thread boundaries and scope validity ([Mandry,
*Contexts and capabilities in
Rust*](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/)). The proposal borrows
the word "capability" from a security model whose other properties it does not claim; the
[capabilities](./capabilities.md) page separates the senses.

Nadrieril's follow-up extends dictionary passing so that a trait bound can carry a runtime value. It
names the difficulties that follow: ownership modes for the carried value, the same type satisfying
a trait differently in different scopes, and methods that become closures ([*What If Traits Carried
Values*](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html)).

### Cairo: a Rust-like language that shipped named impls

Cairo provides a working example of named impls in a language with Rust-like syntax. A generic
function can take an impl as an explicit parameter, named or anonymous:

```rust
// Cairo
fn largest_list<T, impl TDrop: Drop<T>>(l1: Array<T>, l2: Array<T>) -> Array<T> { /* ... */ }

fn smallest_element<T, +PartialOrd<T>, +Copy<T>, +Drop<T>>(list: @Array<T>) -> T { /* ... */ }
```

A caller can let the compiler infer the impl from those visible at the call site ([Cairo Book,
*Traits*](https://www.starknet.io/cairo-book/ch08-02-traits-in-cairo.html);
[*Generic Data Types*](https://www.starknet.io/cairo-book/ch08-01-generic-data-types.html) ). This
inference matters for the comparison: because resolution uses what is in scope, a call site's
imports decide which implementation it uses.

## How CGP expresses it

CGP records provider choices on a context. Named-impl designs can instead select an impl at a call
site, and capability designs can bind a value within a scope. The examples below show the practical
effect of that difference.

### Naming implementations with providers {#a-named-impl-is-a-provider}

A CGP provider corresponds to an impl named with `impl Name<T> = Trait<T> for T` in Boxy's proposal.
Each implementation has its own zero-sized type. Overlapping implementations of one trait can
therefore coexist, because each implements the provider trait for its own type. The encoder pair
from the [Introduction](/docs/) shows this. Rust rejects two blanket impls of one trait for
`T: Display` and `T: AsRef<[u8]>` , since `String` satisfies both. Written as providers, both
compile:

```rust
#[cgp_component(Encoder)]
pub trait CanEncode<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}

#[cgp_impl(new EncodeWithDisplay)]
impl<Value: Display> Encoder<Value> {
    fn encode(&self, value: &Value) -> Vec<u8> {
        value.to_string().into_bytes()
    }
}

#[cgp_impl(new EncodeBytes)]
impl<Value: AsRef<[u8]>> Encoder<Value> {
    fn encode(&self, value: &Value) -> Vec<u8> {
        value.as_ref().to_vec()
    }
}
```

`EncodeWithDisplay` and `EncodeBytes` play the role of named impls. Both accept `String` , and a
context selects one. A downstream crate can define another provider even if it owns neither the
component nor the value type, because it implements the provider trait for its own marker type.
Rust's coherence rules still apply to those generated impls.

The encoded value is the `Value` parameter rather than `Self` , so `Self` can stand for an
application. The contexts below are **[environmental
contexts](/docs/reference/glossary#environmental-context)**, and the component is
[parameter-targeted](/docs/reference/glossary#parameter-targeted-component) .

### Selecting implementations on a context {#an-incoherent-bound-resolves-through-the-context-not-at-each-call-site}

Boxy's `function::<T + TraitImpl<T>>` passes the impl at every call. CGP lets generic code require
the consumer trait on the context and makes the choice where the context is defined. Two
applications can choose differently for the same type:

```rust
pub struct ApiServer;
pub struct Firmware;

delegate_components! {
    ApiServer {
        open EncoderComponent;
        @EncoderComponent.String: EncodeWithDisplay,
    }
}

delegate_components! {
    Firmware {
        open EncoderComponent;
        @EncoderComponent.String: EncodeBytes,
    }
}
```

Every call through `ApiServer` uses its selected provider for `String` ; `Firmware` has its own
choice. This gives consistency per context, component, and dispatch key. Boxy's sketch permits
explicit choices at calls, while Cairo can infer them from scope. The
[type classes](./type-classes.md) page compares these choices with Haskell's instance resolution.

### Passing providers as type parameters {#passing-an-impl-explicitly-is-a-higher-order-provider}

Where the proposals and Cairo let a caller pass an impl as a parameter, CGP uses
[higher-order providers](/docs/concepts/higher-order-providers) . A provider takes another provider
as a type parameter and binds it with `#[use_provider]` , which corresponds to Cairo's
`impl TDrop: Drop<T>` written as a Rust generic:

```rust
#[cgp_impl(new ScaledArea<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        InnerCalculator::area(self) * scale_factor * scale_factor
    }
}
```

Wiring `AreaCalculatorComponent: ScaledArea<RectangleArea>` selects both the wrapper and its inner
implementation. The example assumes an `AreaCalculator` component and a `RectangleArea` provider
that reads `width` and `height` . A `ScaledRectangle` context would supply those fields plus
`scale_factor` . It is a **value context** because the measured value carries the wiring.

Higher-order providers let a wrapper fix or parameterize its inner choice. An inner parameter can
also default to [`UseContext`](/docs/reference/providers/use_context) , which delegates that step
back to the context's selected implementation.

### A `with` clause becomes a context field

Mandry's `with arena: &BasicArena` clause declares a value the function needs from its environment.
CGP expresses an environmental dependency as an
[implicit argument](/docs/concepts/implicit-arguments) read from the context. This fragment assumes
a `Greeter` component whose method returns a `String` :

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

The `with` block that supplies the value becomes a context type that carries the field. Constructing
a value of that type makes the binding:

```rust
#[derive(HasField)]
pub struct App {
    pub name: String,
}

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

let app = App { name: "World".to_owned() };
app.greet();   // "Hello, World!"
```

Constructing `App` supplies the value that `GreetHello` reads. The access resolves statically to a
field read, and a missing field causes a compile error when the component is checked or used.
[`check_components!`](/docs/reference/macros/check_components) lets you request that check beside
the wiring. `App` is an environmental context; the greeter is
[self-targeted](/docs/reference/glossary#self-targeted-component) because its method acts on `self`
.

### The context is the dictionary

A CGP context serves a role similar to a root dictionary: it connects generic code to its
dependencies. Each provider receives the context, whose wiring selects other providers. The analogy
describes how dependencies are organized; CGP resolves the selections statically and does not store
a runtime record of function pointers.

Providers can refer to each other's operations through the shared context. This avoids a manual
order for constructing separate dictionaries. Rust still checks the resulting trait dependencies.
The [ML modules](./ml-modules.md) page compares this with explicit functor application.

## What each approach costs

These proposals have different levels of maturity and different open questions. Specialization has
an accepted RFC but remains unstable because of a soundness problem ([tracking issue
#31844](https://github.com/rust-lang/rust/issues/31844)). Named impls and dictionary passing remain
design sketches whose authors discuss syntax, ergonomics, migration, and coherence
([Boxy](https://www.boxyuwu.blog/posts/an-incoherent-rust/);
[Nadrieril](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html) ). Contexts
and capabilities has open questions about thread boundaries and scope
([Mandry](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/)). Cairo's
scope-based resolution requires the caller to import the needed impls; two call sites can then
resolve the same type differently.

CGP requires a component definition and explicit wiring. A provider trait created by
[`#[cgp_component]`](/docs/reference/macros/cgp_component) cannot retrofit a foreign trait such as
`serde::Serialize` without a parallel component. Wiring adds compile-time work and can produce long
trait errors over generated types. [`cargo cgp check`](/docs/cargo-cgp/check) leads with the root
cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every
class. The [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs
against simpler approaches.

## Where a proposal, or a smaller tool, is the better choice

A single specialized fast path may need only manual dispatch on stable Rust. On nightly,
`min_specialization` supports a restricted set of specialization cases without CGP wiring. Ordinary
coherence fits a program that needs one global implementation, such as the `Ord` used by map keys. A
future language feature may express scoped or inferred choices with less code than a library can.

CGP fits programs that need several valid implementations chosen per application, providers for
foreign types, or environment values available in deep code. It offers these choices on stable Rust
through explicit wiring.

## What to expect that differs

CGP's explicit context introduces limits that the proposals address differently:

- **Imports do not select providers.** Wiring records the choice for a context. Rust still resolves
  trait bounds, but CGP does not choose a provider by searching the impls imported at each call.
- **Bindings belong to types.** A lexical block does not shadow a context's wiring. A different
  selection requires another context type or a context adapter.
- **Implicit arguments borrow the context.** A method can take `&self` or `&mut self`, but implicit
  a mutable implicit argument must be the only implicit argument in that method. Independent ownership or mutation
  may need a different data layout, interior mutability, or cloning.
- **Existing traits retain their rules.** CGP introduces provider traits and wiring; it does not
  enable specialization or additional impls of an arbitrary foreign trait.

## Where to go next

These pages develop the constructs and the neighbouring comparisons:

- [Bypassing coherence](/docs/concepts/coherence): the coherence rules and the provider move, without
  reference to the proposals.
- [Higher-order providers](/docs/concepts/higher-order-providers) and
  [implicit arguments](/docs/concepts/implicit-arguments): the two constructs this page maps onto
  passed impls and `with` clauses.
- [Type classes](./type-classes.md): the same trade seen from Haskell's side, including the
  `INCOHERENT` and overlapping-instance extensions.
- [RustLab 2025 talk](/blog/rustlab-2025-coherence): why coherence exists and why specialization does
  not remove the overlap problem, argued on a hash-table example.

## Sources

The Rust snippets from the proposals are quoted from their authors' posts and the Cairo book. The
specialization snippet was compiled against a stable toolchain to confirm it is rejected with
`E0658` ; the CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!`
assertion per wired context.

- [Rust Reference, *Implementations*](https://doc.rust-lang.org/reference/items/implementations.html) and [RFC 2451](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html): the orphan and overlap rules.
- [RFC 1210, *impl specialization*](https://rust-lang.github.io/rfcs/1210-impl-specialization.html) and [tracking issue #31844](https://github.com/rust-lang/rust/issues/31844): the strictly-more-specific rule, the `default` keyword, the lifetime soundness hole, and the `min_specialization` subset.
- [Boxy, *An Incoherent Rust*](https://www.boxyuwu.blog/posts/an-incoherent-rust/): named impls, `incoherent trait`, the ecosystem-evolution motivation, and the `HashMap` and associated-type answers.
- [Nadrieril, *Elaborating Rust Traits to Dictionary-Passing Style*](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html) and [*What If Traits Carried Values*](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html): traits as structs and impls as values, and the extension that carries values.
- [Mandry, *Contexts and capabilities in Rust*](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/): the `with` clause and block, context-dependent trait impls, and the open questions.
- [Cairo Book, *Traits in Cairo*](https://www.starknet.io/cairo-book/ch08-02-traits-in-cairo.html) and [*Generic Data Types*](https://www.starknet.io/cairo-book/ch08-01-generic-data-types.html): named and anonymous impl parameters and compiler inference of the impl.
- [Ixrec, *rust-orphan-rules*](https://github.com/Ixrec/rust-orphan-rules) and [Greyblake, *Alternative blanket implementations for a single Rust trait*](https://www.greyblake.com/blog/alternative-blanket-implementations-for-single-rust-trait/): the orphan rule as a durable frustration and the hand-rolled marker-struct workaround for overlap.
- [RustLab 2025, *How to stop fighting with coherence*](/blog/rustlab-2025-coherence): why specialization does not solve overlap, and the connection to the contexts-and-capabilities proposal.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
