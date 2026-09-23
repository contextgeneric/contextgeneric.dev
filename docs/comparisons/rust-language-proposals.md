---
sidebar_label: "Rust's own proposals"
sidebar_position: 1
description: 'CGP read against specialization, named impls, dictionary passing, and contexts and capabilities, for a reader who follows the coherence debate.'
---

# Rust's own proposals

CGP overlaps with Rust proposals for trait selection and values supplied by a caller's environment.
It is a language extension built as a [stable Rust library](/docs/), with pluggable trait
implementations at compile time and ordinary Rust consumer traits. This page compares CGP with
specialization, dictionary passing, named impls, and contexts and capabilities. It shows where the
designs meet and what CGP cannot express.

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
Rust's coherence rules set the starting point. The
*[orphan rule](/docs/reference/glossary#orphan-rule)* says an `impl<P1..=Pn> Trait<T1..=Tn> for T0`
is valid only if `Trait` is a local trait, or at least one of the types `T0..=Tn` is a local type and
no uncovered type parameter appears before it. The *overlap rule* rejects two implementations that
"can be instantiated with the same type"
([Rust Reference](https://doc.rust-lang.org/reference/items/implementations.html);
[RFC 2451](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html)).

Together the rules give each trait lookup one impl. Generic code can therefore use `T: Hash`
consistently, including inside a `HashMap<K, V>`. The
[type classes](./type-classes.md) page covers why coherence exists in the wider type-class
tradition; this page takes it as given.

### Specialization

RFC 1210 proposes letting two impls overlap when one is *strictly more specific*, with the compiler
choosing the more specific one. Its own example refines a blanket `AddAssign`:

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
*unsound*", because dispatch information about lifetimes is erased before code generation. The
`min_specialization` subset, which the standard library uses internally, is the usable part
([tracking issue #31844](https://github.com/rust-lang/rust/issues/31844)). On a stable toolchain the
snippet above fails with `E0658`.

Specialization also does not address the case CGP starts from. Two impls of equal generality, such
as `Hash for T: Display` and `Hash for T: AsRef<[u8]>`, are not ordered by specificity, so
specialization rejects them as coherence does. In addition, generic code can select a blanket impl
through a bound that never mentions the trait. By the time the concrete type is known, that code can
no longer see the more specific impl. The [RustLab 2025 talk](/blog/rustlab-2025-coherence) develops
this argument with a hash-table example.

### Dictionary passing

Another line of work treats the trait system as a shorthand for *dictionary passing*. A trait
becomes a struct of function pointers, an impl becomes a value of that struct, and a trait bound
becomes an argument the caller passes. Nadrieril works this out for Rust:

```rust
struct Clone<Self> {
    clone: fn(&Self) -> Self,
}

const CLONE_U32: Clone<u32> = Clone { clone: |x: &u32| *x };
```

In this account, trait solving is the process that supplies these values. Coherence becomes a source
of friction, because a global impl constrains a caller in ways the caller cannot see
([Nadrieril, *Dictionary-passing style*](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html)).
Once impls are values, choosing a different impl means passing a different value, and coherence
forbids exactly that choice.

### Named and incoherent impls

Boxy's *An Incoherent Rust* proposes replacing the coherence rules with explicit impl selection. The
motivation is *ecosystem evolution*. The post argues that once a foundational crate such as `serde`
establishes the trait impls for common types, an alternative cannot gain adoption, because
downstream crates cannot implement the competing trait for types they do not own. The sketch names an
impl and passes it where a bound is required:

```rust
impl Name<T> = Trait<T> for T { /* ... */ }

function::<T + TraitImpl<T> + OtherTraitImpl<T>>(/* ... */)
```

The post answers the two classic justifications for coherence directly. It solves the `HashMap`
problem by moving the `Hash` and `Eq` bounds onto the type definition, so every operation on one map
uses the same impl. It keeps associated types sound by making the impl part of the type. It names
the remaining challenges as substantial: syntax, ergonomics, migration, and whether the added
complexity is worthwhile ([Boxy, *An Incoherent Rust*](https://www.boxyuwu.blog/posts/an-incoherent-rust/)).

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
given capability is in scope. The mechanism compiles to ordinary function arguments, so it adds no
runtime cost. Its open questions concern thread boundaries and scope validity
([Mandry, *Contexts and capabilities in Rust*](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/)).
The proposal borrows the word "capability" from a security model whose other properties it does not
claim; the [capabilities](./capabilities.md) page separates the senses.

Nadrieril's follow-up extends dictionary passing so that a trait bound can carry a runtime value. It
names the difficulties that follow: ownership modes for the carried value, the same type satisfying a
trait differently in different scopes, and methods that become closures
([*What If Traits Carried Values*](https://nadrieril.github.io/blog/2026/03/22/what-if-traits-carried-values.html)).

### Cairo: a Rust-like language that shipped named impls

Cairo borrows Rust's surface syntax and gives every impl a name, which makes it the closest shipped
counterpart to Incoherent Rust. A generic function can take an impl as an explicit parameter, named
or anonymous:

```rust
// Cairo
fn largest_list<T, impl TDrop: Drop<T>>(l1: Array<T>, l2: Array<T>) -> Array<T> { /* ... */ }

fn smallest_element<T, +PartialOrd<T>, +Copy<T>, +Drop<T>>(list: @Array<T>) -> T { /* ... */ }
```

A caller does not pass the impl. The compiler infers it from the impls visible at the call site
([Cairo Book, *Traits*](https://www.starknet.io/cairo-book/ch08-02-traits-in-cairo.html);
[*Generic Data Types*](https://www.starknet.io/cairo-book/ch08-01-generic-data-types.html)). This
inference matters for the comparison: because resolution uses what is in scope, a call site's
imports decide which implementation it uses.

## How CGP expresses it

CGP records provider choices on a context. Named-impl designs can instead select an impl at a call
site, and capability designs can bind a value within a scope. The examples below show the practical
effect of that difference.

### A named impl is a provider

A CGP provider corresponds to an impl named with `impl Name<T> = Trait<T> for T` in Boxy's proposal.
Each implementation has its own zero-sized type. Overlapping implementations of one trait can
therefore coexist, because each implements the provider trait for its own type. The encoder pair
from the [Introduction](/docs/) shows this. Rust rejects two blanket impls of one trait for
`T: Display` and `T: AsRef<[u8]>`, since `String` satisfies both. Written as providers, both
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

`EncodeWithDisplay` and `EncodeBytes` are named impls in the proposal's sense. They overlap on
`String`, and a context selects one by name. A crate that owns neither `Display` nor `String` can
define more of them, because each provider trait is implemented for a provider type the defining
crate owns. Specialization cannot provide this relief from the overlap and orphan rules, because the
two providers are equally general.

The encoded value is the `Value` parameter rather than `Self`, so `Self` can stand for an
application. The contexts below are
**[environmental contexts](/docs/reference/glossary#environmental-context)**, and the component is
[parameter-targeted](/docs/reference/glossary#parameter-targeted-component).

### An incoherent bound resolves through the context, not at each call site

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

CGP makes the choice on the context, so every call through `ApiServer` uses the same provider.
Boxy's design selects explicitly at a call, while Cairo infers from the impls in scope. A context
selects one provider per component, keeping code that shares that context consistent. The proposal
addresses the `HashMap` concern by putting `Hash` on the type definition. The
[type classes](./type-classes.md) page develops the comparison with Haskell's incoherent instances.

### Passing an impl explicitly is a higher-order provider

Where the proposals and Cairo let a caller pass an impl as a parameter, CGP uses
[higher-order providers](/docs/concepts/higher-order-providers). A provider takes another provider
as a type parameter and binds it with `#[use_provider]`, which corresponds to Cairo's
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

A context wires `AreaCalculatorComponent: ScaledArea<RectangleArea>`, naming the inner impl that a
Cairo compiler would infer. CGP can assemble providers at each call site, but doing so is verbose,
and idiomatic CGP leaves the assembly to the context. A higher-order provider suits a provider that
must fix its inner choice locally. Such providers often default the inner parameter to
[`UseContext`](/docs/reference/providers/use_context), so that the context's wiring is the fallback.
The wired type here is a `ScaledRectangle` with `width`, `height`, and `scale_factor` fields. It is a
**value context**: the type being measured also carries the wiring.

### A `with` clause becomes a context field

Mandry's `with arena: &BasicArena` clause declares a value the function needs from its environment.
CGP declares the same need as an [implicit argument](/docs/concepts/implicit-arguments), which reads
a field of the context that every provider receives as `self`:

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

`App` is the context, and constructing it plays the role of the `with` block. Two properties of the
proposal carry over. The binding adds no runtime cost, since a field read compiles to a load. It is
also checked statically: a missing field is a compile error, which
[`check_components!`](/docs/reference/macros/check_components) reports at the wiring site. `App` here
is an environmental context with one field, and the greeter component is
[self-targeted](/docs/reference/glossary#self-targeted-component).

### The context is the dictionary

The dictionary-passing account and CGP meet in one observation: the context is the top-level
dictionary. In Nadrieril's elaboration, every dictionary is a value passed alongside the data. In
CGP, every provider receives the context, and the context's wiring table is a type-level record of
dictionaries, with one `DelegateComponent` entry per component. Lowering a CGP program to
dictionary-passing form would turn the context into a struct whose fields are the other
dictionaries, with close to one field per `delegate_components!` entry.

This arrangement also handles a problem the proposals must solve separately. Every dictionary
reaches the others through the context, so components that depend on each other resolve without an
instantiation order: the context is one stable root from which all of them are reachable. The
[ML modules](./ml-modules.md) page contrasts this with manual functor application.

## What each approach costs

These proposals have different levels of maturity and different open questions. Specialization has
an accepted RFC but remains unstable because of a soundness problem
([tracking issue #31844](https://github.com/rust-lang/rust/issues/31844)). Named impls and dictionary
passing remain design sketches whose authors discuss syntax, ergonomics, migration, and coherence
([Boxy](https://www.boxyuwu.blog/posts/an-incoherent-rust/);
[Nadrieril](https://nadrieril.github.io/blog/2026/03/20/dictionary-passing-style.html)). Contexts and
capabilities has open questions about thread boundaries and scope
([Mandry](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/)). Cairo's
scope-based resolution requires the caller to import the needed impls; two call sites can then
resolve the same type differently.

CGP requires a component definition and explicit wiring. A provider trait created by
[`#[cgp_component]`](/docs/reference/macros/cgp_component) cannot retrofit a foreign trait such as
`serde::Serialize` without a parallel component. Wiring adds compile-time work and can produce long
trait errors over generated types. [`cargo cgp check`](/docs/cargo-cgp/check) identifies the root
cause for errors it recognizes. The [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page
weighs these costs against simpler approaches.

## Where a proposal, or a smaller tool, is the better choice

A smaller tool fits a single specialized fast path under a blanket impl. `min_specialization` on
nightly, or manual dispatch on stable Rust, handles that case without CGP wiring. Ordinary coherence
fits a program that needs one global implementation, such as the `Ord` used by map keys. A future
language feature may express scoped or inferred choices with less code than a library can.

CGP fits programs that need several valid implementations chosen per application, providers for
foreign types, or environment values available in deep code. It offers these choices on stable Rust
through explicit wiring.

## What to expect that differs

**CGP does not infer an impl from scope.** Cairo and the named-impl sketch let the compiler choose
among the impls visible at a call site. CGP has no such search: a context names every provider it
uses in its wiring table. This design prevents two call sites in one program from resolving the same
type differently by accident.

**Bindings are flat.** Mandry's `with` blocks nest, and an inner scope can shadow an outer binding.
In CGP every binding lives where the concrete context is defined, so an inner scope cannot replace a
provider without defining a new context type. In return, all bindings for a context can be read in
one place.

**Providers share one context value.** A provider method can take `&self` or `&mut self`, but a
`&mut self` method can borrow only one field mutably through its implicit arguments. A value that
several providers must mutate or own independently needs interior mutability or cloning; the
proposals discuss designs that track ownership modes directly.

**CGP is not specialization for stable Rust.** It does not choose a more specific impl, it does not
change existing traits, and it asks for wiring that specialization would infer. It addresses the two
problems specialization leaves open, overlap between equally general impls and the orphan rule, by
naming implementations and choosing them per context.

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
