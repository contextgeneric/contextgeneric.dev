---
sidebar_label: "Rust's own proposals"
sidebar_position: 1
description: 'CGP read against specialization, named impls, dictionary passing, and contexts and capabilities, for a reader who follows the coherence debate.'
---

# Rust's own proposals

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who has
followed Rust's own debate about relaxing coherence: *specialization*, the *dictionary-passing*
account of traits, *named or incoherent impls*, and *contexts and capabilities*. It shows which
fragment of each proposal CGP provides today, how the correspondence runs construct by construct,
and the two places where CGP cannot follow the proposals. It ends with what to expect that differs
and where to read further.

## In your terms

The proposals and CGP share a vocabulary once the names are lined up. A **context** in CGP is the
type the method runs on, which supplies the values it needs as its fields; in most CGP code it is a
type you define to stand for an application.

| In the proposals | In CGP |
| --- | --- |
| A named impl (`impl Name<T> = Trait<T> for T`) | A **provider**: a zero-sized type that implements a component's provider trait |
| An `incoherent trait` bound passed at a call | A consumer-trait bound on the context, resolved by the context's **wiring** |
| An impl passed as a parameter (`impl TDrop: Drop<T>` in Cairo) | A **higher-order provider**, bound with `#[use_provider]` |
| A capability in a `with` clause | An `#[implicit]` argument read from a context field |
| The `with` block that binds everything | The definition of a context type and its `delegate_components!` table |
| The root dictionary of a dictionary-passing elaboration | The context itself |

## The idea, briefly

Rust's Reference gives the two rules every proposal here relaxes. The *orphan rule* says an
`impl<P1..=Pn> Trait<T1..=Tn> for T0` is valid only if `Trait` is a local trait, or at least one of
the types `T0..=Tn` is a local type and no uncovered type parameter appears before it. The *overlap
rule* rejects two implementations that "can be instantiated with the same type"
([Rust Reference](https://doc.rust-lang.org/reference/items/implementations.html);
[RFC 2451](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html)). Together they
guarantee that any trait lookup finds exactly one impl, and that guarantee lets generic code resolve
`T: Hash` transitively and keeps a `HashMap<K, V>` sound. The
[type classes](./type-classes.md) page covers why coherence exists in the wider tradition; this page
takes it as given.

### Specialization

RFC 1210 proposes letting two impls overlap when one is *strictly more specific*, with the compiler
picking the specific one. Its own example refines a blanket `AddAssign`:

```rust
impl<R, T: Add<R> + Clone> AddAssign<R> for T {
    default fn add_assign(&mut self, rhs: R) {
        let tmp = self.clone() + rhs;
        *self = tmp;
    }
}
```

The `default` keyword marks an item that a more specific impl may override
([RFC 1210](https://rust-lang.github.io/rfcs/1210-impl-specialization.html)). The feature has been
unstable since 2016. Its tracking issue states that the full feature "as implemented currently is
*unsound*", because dispatch information about lifetimes is erased before code generation, and the
`min_specialization` subset the standard library uses internally is the usable fragment
([tracking issue #31844](https://github.com/rust-lang/rust/issues/31844)). On a stable toolchain
the snippet above fails with `E0658`.

Specialization also does not address the case CGP starts from. Two impls of equal generality, such as
`Hash for T: Display` and `Hash for T: AsRef<[u8]>`, are not ordered by specificity, so specialization
rejects them as coherence does. And generic code can bind a blanket impl through a bound that never
mentions the trait, so by the time the concrete type is known the specialization is invisible. The
[RustLab 2025 talk](/blog/rustlab-2025-coherence) builds this argument on a hash-table example.

### Dictionary passing

A second line of thought treats the trait system as sugar over *dictionary passing*: a trait becomes
a struct of function pointers, an impl becomes a value of that struct, and a trait bound becomes an
argument the caller passes. Nadrieril works it out for Rust:

```rust
struct Clone<Self> {
    clone: fn(&Self) -> Self,
}

const CLONE_U32: Clone<u32> = Clone { clone: |x: &u32| *x };
```

Trait solving is then the process that supplies these values, and coherence is the point of
friction, since a global impl constrains a caller in ways the caller cannot see
([Nadrieril, *Dictionary-passing style*](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html)).
Once impls are values, *choosing* an impl is passing a different value, which is the freedom
coherence forbids.

### Named and incoherent impls

Boxy's *An Incoherent Rust* proposes dropping the coherence rules in favour of explicit impl
selection. The motivation is *ecosystem evolution*: once a foundational crate such as `serde`
establishes the trait impls for common types, an alternative cannot gain adoption, because downstream
crates cannot implement the competing trait for types they do not own. The sketch names an impl and
passes it where a bound is required:

```rust
impl Name<T> = Trait<T> for T { /* ... */ }

function::<T + TraitImpl<T> + OtherTraitImpl<T>>(/* ... */)
```

The post answers the two classic justifications for coherence on their own terms. The `HashMap`
problem is solved by moving the `Hash` and `Eq` bounds onto the type definition, so every operation
on one map agrees on one impl. Associated-type soundness holds because the impl is part of the type.
It names the remaining challenges as substantial: syntax, ergonomics, migration, and whether the
complexity is worth it ([Boxy, *An Incoherent Rust*](https://www.boxyuwu.blog/posts/an-incoherent-rust/)).

### Contexts and capabilities

Tyler Mandry's 2021 proposal addresses a different limitation: a function or impl cannot reach a value
from its caller's environment, such as an allocator or a logger, except by threading it as a
parameter or storing it in a global. A `with` clause on a declaration names the context it requires,
and a `with` block at a call site supplies it:

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

The distinctive power is a *context-dependent trait impl*, valid only when a given capability is in
scope. The mechanism compiles to ordinary function arguments, so it costs nothing at runtime, and the
open questions concern thread boundaries and scope validity
([Mandry, *Contexts and capabilities in Rust*](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/)).
The proposal borrows the word "capability" from a security model whose other properties it does not
claim; the [capabilities](./capabilities.md) page separates the senses. Nadrieril's follow-up extends
dictionary passing so a trait bound may carry a runtime value, and names the difficulties that follow:
ownership modes for the carried value, the same type satisfying a trait differently in different
scopes, and methods that become closures
([*What If Traits Carried Values*](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html)).

### Cairo: a Rust-like language that shipped named impls

Cairo borrows Rust's surface syntax and gives every impl a name, so it is the nearest thing to a
shipped Incoherent Rust. A generic function may take an impl as an explicit parameter, named or
anonymous:

```rust
// Cairo
fn largest_list<T, impl TDrop: Drop<T>>(l1: Array<T>, l2: Array<T>) -> Array<T> { /* ... */ }

fn smallest_element<T, +PartialOrd<T>, +Copy<T>, +Drop<T>>(list: @Array<T>) -> T { /* ... */ }
```

A caller does not pass the impl: the compiler infers it from the impls visible at the call site
([Cairo Book, *Traits*](https://www.starknet.io/cairo-book/ch08-02-traits-in-cairo.html);
[*Generic Data Types*](https://www.starknet.io/cairo-book/ch08-01-generic-data-types.html)). That
inference is the design decision that matters for the comparison, because resolving from scope means
the set of imports decides which implementation a call uses.

## How CGP expresses it

CGP provides a fragment of each proposal as a library, and the correspondence is close enough to
state construct by construct. What the proposals would add to the language, CGP adds as a library,
with one standing restriction: every binding lives in one place, the context.

### A named impl is a provider

A CGP provider is the thing Boxy's proposal names with `impl Name<T> = Trait<T> for T`. Every
implementation gets its own zero-sized type, so two overlapping impls of one trait coexist because
each implements the provider trait for its own name. The encoder pair from the
[Introduction](/docs/) shows it. Rust rejects two blanket impls of one trait for `T: Display` and
`T: AsRef<[u8]>`, since `String` satisfies both. As providers, both compile:

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

`EncodeWithDisplay` and `EncodeBytes` are named impls in the proposal's sense. They overlap on
`String`, they are selected by name, and a crate that owns neither `Display` nor `String` may define
more, because the provider trait is always implemented for a type the crate owns. This is the
overlap and orphan relief specialization cannot give, since the two providers are equally general.
The encoded value has moved from `Self` into the `Value` parameter, so `Self` is free to be an
application: the contexts below are **[environmental contexts](/docs/reference/glossary#environmental-context)**, types that stand for an application,
and the component is [parameter-targeted](/docs/reference/glossary#parameter-targeted-component).

### An incoherent bound resolves through the context, not at each call site

Boxy's `function::<T + TraitImpl<T>>` passes the impl at every call. CGP lets generic code require
the consumer trait on the context and defers the choice to the place the context is defined. Two
applications answer differently for the same type:

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

This is the proposal's incoherence with a different resolution point. Boxy and Cairo resolve at the
call site, explicitly or from what is in scope. CGP resolves at the context, so every call through
`ApiServer` agrees. That is also how CGP answers the `HashMap` concern: a context selects one
provider per component, so all code sharing a context agrees on one impl, which is the property the
proposal recovers by moving `Hash` onto the type definition. The [type classes](./type-classes.md)
page develops this as incoherent instances made deterministic.

### Passing an impl explicitly is a higher-order provider

Where the proposals and Cairo let a caller pass an impl as a parameter, CGP has
[higher-order providers](/docs/concepts/higher-order-providers). A provider takes another provider as
a type parameter and binds it with `#[use_provider]`, which is Cairo's `impl TDrop: Drop<T>` as a
Rust generic:

```rust
#[cgp_impl(new ScaledArea<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        InnerCalculator::area(self) * scale_factor * scale_factor
    }
}
```

A context wires `AreaCalculatorComponent: ScaledArea<RectangleArea>`, naming the inner impl where a
Cairo caller would have the compiler infer it. Assembling providers at every call site is possible in
CGP but verbose, and idiomatic CGP defers the assembly to the context. A higher-order provider is the
form for a provider that must fix its inner choice locally, and it defaults its inner parameter to
[`UseContext`](/docs/reference/providers/use_context) so the context's wiring is the fallback. The
wired type here is a `ScaledRectangle` carrying `width`, `height`, and `scale_factor`, a **value
context**: the type being measured also carries the wiring.

### A capability is a context field; the `with` block is the context

Mandry's `with arena: &BasicArena` clause declares a value the function needs from its environment.
CGP declares the same thing as an [implicit argument](/docs/concepts/implicit-arguments), which
reads a field of the context every provider receives as `self`:

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

The `with` block that supplies the capability becomes a context type carrying the field, and the
binding happens when a value of that type is constructed:

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

`App` is the context, and constructing it is the `with` binding. Two properties of the proposal carry
over exactly: the binding costs nothing at runtime, since a field read compiles to a load, and it is
checked statically, since a missing field is a compile error that
[`check_components!`](/docs/reference/macros/check_components) names at the wiring site. `App` here
is an environmental context with one field, and `GreetHello` is [self-targeted](/docs/reference/glossary#self-targeted-component).

### The context is the dictionary

The dictionary-passing account and CGP meet in one observation: the context is the top-level
dictionary. In Nadrieril's elaboration every dictionary is a value passed alongside the data. In CGP
every provider receives the context, and the context's wiring table is a type-level record of
dictionaries, one `DelegateComponent` entry per component. Lowering a CGP program to
dictionary-passing form turns the context into a struct whose fields are the other dictionaries, with
a near one-to-one correspondence between a `delegate_components!` entry and a field of that struct.
The arrangement has a practical consequence the proposals have to solve separately: because every
dictionary reaches the others through the context, a dependency graph with cycles resolves without
an instantiation order, since the context is one stable root from which all of them are reachable.
The [ML modules](./ml-modules.md) page contrasts this with manual functor application.

## What each approach costs

The proposals are language changes that have not shipped, and their costs are the ones their authors
name. Specialization has an accepted RFC, a soundness hole open since 2016, and no path to
stabilization ([tracking issue #31844](https://github.com/rust-lang/rust/issues/31844)). The
named-impl and dictionary-passing designs are blog posts and design sketches, and their authors list
syntax, ergonomics, migration of the existing ecosystem, and the loss of the guarantees coherence
gives for free as the open problems ([Boxy](https://www.boxyuwu.blog/posts/an-incoherent-rust/);
[Nadrieril](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html)). Contexts
and capabilities is a research direction from 2021 with open questions about thread boundaries and
scope ([Mandry](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/)). Cairo's
scope-based inference means a caller must import the whole chain of impls a generic impl needs, and
two call sites in one project can resolve the same type differently.

CGP's costs are the ordinary ones. The provider trait has to be defined through
[`#[cgp_component]`](/docs/reference/macros/cgp_component), so CGP cannot retrofit an existing
foreign trait such as `serde::Serialize` without a parallel component, where a language change would
apply to every trait. The wiring is code somebody writes and reads, where a language feature would
infer it. The compile-time work is real. And the raw diagnostics are trait-solver output over
generated types: [`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes
it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page is where these costs are weighed
against the alternatives in full.

## Where a proposal, or a smaller tool, is the better choice

Where a program needs exactly a specialized fast path for one type under a blanket impl,
`min_specialization` on nightly, or a manual dispatch trick on stable, is the smaller tool, and CGP
would add wiring specialization would infer. Where a program needs one globally consistent
implementation, such as the `Ord` a map's keys rely on, coherence as it stands is the right
arrangement and CGP's per-context choice is not an improvement on it. And where a program can wait,
a language feature that infers the impl from scope or nests bindings dynamically would express these
patterns with less ceremony than a library can. Where the need is several equally valid impls chosen
per application, impls for types the program does not own, or environment values reaching deep code
without parameters, CGP is the one option available on stable Rust today, and its explicitness is the
price of that.

## What to expect that differs

**CGP does not infer an impl from scope.** Cairo and the named-impl sketch let the compiler pick an
impl from those visible at a call site. CGP has no such search: a context names every provider it
uses in its wiring table. This is deliberate, and it keeps two call sites in one program from
resolving the same type differently by accident.

**Bindings are flat.** Mandry's `with` blocks nest, and an inner scope can shadow an outer binding. In
CGP every binding lives at the one place the concrete context is defined, so an inner scope cannot
shadow a provider without defining a new context type. All bindings for a context are readable in one
place, and that is the trade.

**A provider receives `&self`.** A capability that must be `&mut` or owned needs interior mutability
or cloning, rather than the ownership-aware design the proposals discuss.

**CGP is not specialization for stable Rust.** It does not pick a more specific impl, it does not
touch existing traits, and it asks for wiring specialization would infer. It addresses the two
problems specialization leaves open, equally general overlap and orphans, by naming implementations
and choosing them per context.

## Where to go next

- [Bypassing coherence](/docs/concepts/coherence): the coherence rules and the provider move, without
  reference to the proposals.
- [Higher-order providers](/docs/concepts/higher-order-providers) and
  [implicit arguments](/docs/concepts/implicit-arguments): the two constructs this page maps onto
  passed impls and capabilities.
- [Type classes](./type-classes.md): the same trade seen from Haskell's side, including the
  `INCOHERENT` and overlapping-instance extensions.
- [RustLab 2025 talk](/blog/rustlab-2025-coherence): why coherence exists and why specialization does
  not remove the overlap problem, argued on a hash-table example.

## Sources

The Rust snippets from the proposals are quoted from their authors' posts and the Cairo book. The
specialization snippet was compiled against a stable toolchain to confirm it is rejected with
`E0658`; the CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!`
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
