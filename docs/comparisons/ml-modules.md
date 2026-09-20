---
sidebar_label: 'ML modules'
sidebar_position: 4
description: 'CGP read against OCaml and Standard ML signatures, structures, functors, and modular implicits.'
---

# ML modules and modular implicits

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who knows
the signature, structure, and functor system of OCaml or Standard ML, and the *modular implicits*
extension that adds type-directed resolution over it. CGP's components, providers, and higher-order
providers line up with signatures, structures, and functors, and CGP replaces the manual, ordered
functor application that assembling a large ML program requires with a declarative wiring table. The
page covers that correspondence, the one thing ML does that CGP does not, and what an ML reader
should expect to differ.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. In most CGP code it is a type you define to stand for an application, and it is where the
functor applications an ML program writes by hand are recorded instead.

| In ML | In CGP |
| --- | --- |
| A signature | A **component**: one trait with many possible implementations |
| A structure ascribing to it | A **provider**: a named implementation |
| A functor | A **higher-order provider**, parameterized by another provider |
| A signature's abstract `type t` | An abstract type, chosen by the context through `#[cgp_type]` |
| The sequence of functor applications | The **wiring table**, written with `delegate_components!` |
| Checking an argument against a functor's parameter signature | `check_components!` at the wiring site |
| Modular implicits' type-directed search | Nothing: the table names each choice |

## The idea, briefly

ML modules build large programs from interchangeable, separately specified parts with real type
abstraction. A *signature* states what a component provides without fixing representations, a
*structure* implements it, and a *functor* is a module parameterized by another module. The module
system is widely regarded as one of the most expressive in any language
([Dreyer, *Understanding and Evolving the ML Module System*](https://people.mpi-sws.org/~dreyer/thesis/main.pdf)).
The snippets below are OCaml, compiled with OCaml 5.5.0, except the modular-implicits one, which
follows the proposal paper because that extension has never shipped in mainline OCaml.

### Signatures, structures, and abstract types

A signature specifying an abstract type `t` and a `compare` over it, and a structure implementing it:

```ocaml
module type OrderedType = sig
  type t
  val compare : t -> t -> int
end

module StringCompare = struct
  type t = string
  let compare = String.compare
end
```

The `type t` left without a definition is the core of ML data abstraction. When a structure is
*sealed* with a signature that keeps `t` abstract, clients cannot see the representation, and the
type system enforces representation independence
([*Modules and Data Abstraction in OCaml*](https://cs.wellesley.edu/~cs251/s12/handouts/modules.pdf)).
Sealing hides a *known* type's representation rather than abstracting over an unknown one.

### Functors

A functor takes a module of some signature and produces a module built on top of it:

```ocaml
module Make (O : OrderedType) = struct
  (* ... a set implementation using O.compare ... *)
end

module StringSet = Make (StringCompare)
```

The application is explicit and by name, and OCaml's standard library uses the shape throughout, as
in `Set.Make(String)` ([OCaml, *Functors*](https://ocaml.org/docs/functors)). Two subtleties make
functors heavier than the shape suggests. When a functor result's abstract type must be known to
equal another type, the programmer writes a *sharing constraint* with `with type`, a recurring source
of type errors. And OCaml functors are *applicative* by default while SML's are *generative*, so
whether two applications yield equal abstract types depends on the language and on a trailing `()`
argument ([OCaml manual](https://ocaml.org/manual/5.5/firstclassmodules.html)).

### Assembling a program: manual functor plumbing

Building a whole application from functors means applying each one by hand, in dependency order.
The MirageOS project makes the cost concrete: the unikernel running its website "contains more than
70 modules and a functor application depth of up to 10", and the community built a DSL, Functoria,
to organize the applications, because "OCaml's module language is much less flexible than its
expression language" ([*Functor Driven Development*](https://arxiv.org/pdf/1905.02529)). A mature
ecosystem writing a configuration DSL to escape manual functor application is the clearest evidence
of the limitation CGP's wiring addresses.

### Modular implicits

Modular implicits extends OCaml so that module arguments can be marked implicit and resolved by type.
A function takes an implicit module parameter in braces, and the compiler searches the implicit
modules in scope for one of the required signature
([White, Bour & Yallop, *Modular implicits*](https://arxiv.org/abs/1512.01895)):

```ocaml
module type Show = sig
  type t
  val show : t -> string
end

let show {S : Show} x = S.show x

implicit module Show_int = struct
  type t = int
  let show x = string_of_int x
end

implicit module Show_list {S : Show} = struct
  type t = S.t list
  let show x = "[" ^ String.concat ", " (List.map S.show x) ^ "]"
end

let () =
  print_endline (show 5);           (* resolves Show_int *)
  print_endline (show [1; 2; 3])    (* resolves Show_list(Show_int) *)
```

`Show_list` is an *implicit functor*, applied during resolution, which is the module-system
counterpart of Haskell resolving `Show [Int]` from `Show Int`. The proposal remains an
[experimental fork](https://github.com/ocamllabs/ocaml-modular-implicits).

### Classes as signatures

The mapping this page rests on is a known result. Dreyer, Harper, and Chakravarty's *Modular Type
Classes* treats classes as signatures and instances as structures and functors
([*Modular Type Classes*](https://people.mpi-sws.org/~dreyer/papers/mtc/main-long.pdf)). The
[type classes](./type-classes.md) page develops the result and the canonicity-versus-modularity
tension it exposes.

## How CGP expresses it

CGP reproduces the signature, structure, and functor triad with its consumer and provider machinery,
then replaces manual functor application with a declarative table.

### Components are signatures; providers are structures

Declaring a component states the interface, and a provider supplies the implementation:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}
```

`CanCalculateArea` is the signature and `RectangleArea` is a structure ascribing to it. One piece
has no ML counterpart: CGP splits the trait into a consumer side callers use and a provider side
implementers target, whereas an ML signature is one interface used from both sides. The split exists
because Rust has coherence and ML does not. ML holds many structures of one signature by *naming
them*, so it never needs the maneuver CGP uses to escape the one-implementation-per-type rule; the
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) page explains it. A CGP
component is also typically a small signature with one operation, where an ML signature bundles many
values and types; a rich ML signature corresponds to several components combined through `#[uses]`.
The wired type here is a `Rectangle` carrying `width` and `height`, a **value context**: the type
being measured also carries the wiring.

### Abstract-type components are a signature's abstract types

An abstract-type component is CGP's version of a signature's abstract `type t`, defined with
[`#[cgp_type]`](/docs/reference/macros/cgp_type). CGP's own error type is the standing example:

```rust
#[cgp_type]
pub trait HasErrorType {
    type Error: Debug;
}
```

`HasErrorType` declares an abstract `Error` the way `OrderedType` declares an abstract `t`, and
generic code imports it with `#[use_type(HasErrorType.Error)]` and names it as the bare `Error`,
as a functor body refers to `O.t`. The abstraction is the parametric kind, generic code polymorphic
over the type it is given. What CGP does not reproduce is *sealing*: once a context wires `Error` to
a concrete type, no type-system boundary hides that representation from other code. CGP leaves
representation hiding to Rust's module privacy. The [Abstract types](/docs/concepts/abstract-types)
page develops the construct.

### Higher-order providers are functors

A [higher-order provider](/docs/concepts/higher-order-providers) is a provider parameterized by
another provider, as a functor is a module parameterized by another module:

```rust
#[cgp_impl(new ScaledArea<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        InnerCalculator::area(self) * scale_factor * scale_factor
    }
}
```

`ScaledArea<RectangleArea>` is `Make (StringCompare)` in CGP form. Two differences favour CGP's
ergonomics. A higher-order provider can default its inner parameter to
[`UseContext`](/docs/reference/providers/use_context), so an unparameterized `ScaledArea` falls back
to whatever the context wires for that component, an open, recursive application a plain functor
cannot express without recursive modules. And CGP sidesteps the sharing-constraint problem: its
abstract types are supplied by the shared context, so every provider in a context agrees on `Error`
with no `with type` annotation to write.

### `delegate_components!` replaces manual functor application

The decisive difference is that CGP wires a program declaratively where ML applies functors by hand
in order. A context lists its choices in any order, and the trait system resolves each provider's
dependencies through the context:

```rust
use cgp::core::error::ErrorTypeProviderComponent;

delegate_components! {
    App {
        EmailSenderComponent: SendViaSmtp,
        ErrorTypeProviderComponent: UseType<anyhow::Error>,
        // ... further components, in any order
    }
}
```

This is CGP's built-in Functoria. A provider states what it needs as
[impl-side dependencies](/docs/concepts/impl-side-dependencies), and the compiler threads each
requirement to whatever provider the context wires for it, with no application order to get right.
The counterpart to ML's check that a functor argument satisfies its parameter signature is
[`check_components!`](/docs/reference/macros/check_components), which verifies at compile time that a
context supplies every trait its providers transitively need. A reusable bundle of wirings is an
[aggregate provider](/docs/concepts/aggregate-providers) or a [namespace](/docs/concepts/namespaces),
CGP's way of packaging a sub-assembly. `App` here is an **environmental context**, a type standing
for an application, with an `smtp_server` field the email provider reads.

### Modular implicits and CGP wiring: the same problem, opposite selection

Modular implicits and `delegate_components!` both answer the limitation that assembling a program from
functors is manual, and they select implementations by opposite means. Modular implicits resolves by
*implicit search*, which shares type classes' coherence-leaning character: a search can be ambiguous,
and canonicity fights abstraction. CGP resolves by *explicit table*: a context names one provider per
component, so there is no search and no ambiguity, and each context may choose differently. The
mechanism the two share is *recursive composition*. An implicit functor such as `Show_list {S : Show}`
builds `Show` of a list from `Show` of its element, as a higher-order provider builds its behavior
from an inner provider resolved through the context.

## What each approach costs

ML modules are valued for genuine data abstraction through sealing, for functors as reusable
components against an interface, for separate compilation, and for first-class modules where an
implementation must be chosen at runtime
([OCaml manual, *First-class modules*](https://ocaml.org/manual/5.5/firstclassmodules.html)). Their
costs, as their users state them, cluster around weight and plumbing. Assembling a large application
means applying functors by hand in dependency order, painful enough that MirageOS built Functoria,
and rooted in the *stratification* of ML: the module language is a second, weaker language above the
expression language ([*Functor Driven Development*](https://arxiv.org/pdf/1905.02529);
[Rossberg, *1ML*](https://www.cambridge.org/core/journals/journal-of-functional-programming/article/1ml-core-and-modules-united/47B10882829E4B32F98FBA93B28CEF30)).
Sharing constraints and the applicative-versus-generative distinction are recurring sources of
confusion. Base ML has no ad-hoc polymorphism, the gap that motivated modular implicits, whose paper
opens on OCaml's separate `+` and `+.` ([White, Bour & Yallop, 2015](https://arxiv.org/abs/1512.01895)),
and that extension remains an experimental fork.

CGP gives up ML's sealing: its abstract types defer and configure a type but do not hide a
representation behind a type-system boundary. Its wiring is trait resolution, which compiles to
direct calls but produces verbose, generated-type errors: [`cargo cgp check`](/docs/cargo-cgp/check)
leads with the root cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not
yet reshape every class. It resolves at compile time and monomorphizes, so a runtime-chosen
implementation needs a different tool. And its table-driven resolution is more automatic but less
locally obvious than ML's explicit by-name linking. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where ML modules are the better choice

When a program needs representation hiding enforced by the type system, separately compiled modules,
runtime selection through first-class modules, or the explicit control of by-name linking, ML modules
are the better tool, and reaching for CGP would forgo abstraction guarantees ML provides natively. CGP
is the better tool when a program needs many interchangeable implementations chosen per deployment, a
dependency graph the compiler wires rather than the programmer, abstract types unified into the same
selection mechanism, and all of it in one language with no module stratum, and can accept Rust as the
platform.

## What to expect that differs

**There is no application order.** An ML reader expects to apply functors in dependency order. The
CGP table is unordered, and the compiler resolves each provider's dependencies through the context.
This is Functoria's job done by the type system.

**Abstract types are not sealed.** A CGP abstract type defers and configures a type but does not
hide a known one. Representation hiding is Rust's module privacy, separate from the abstract-type
mechanism, and a reader who assumes sealing will look for a guarantee CGP does not make through this
feature.

**Selection is a table, not a search.** Unlike modular implicits, CGP never resolves a provider from
what is in scope. Each context names its choice, and that lets two contexts choose differently
with no ambiguity.

**Every construct is ordinary Rust.** Providers and contexts are types and traits, wired in Rust, so
there is no separate module language. CGP is a library on stable Rust and a consumer trait can still
be implemented directly, without wiring.

## Where to go next

- [Higher-order providers](/docs/concepts/higher-order-providers): the functor correspondence in
  full, including the `UseContext` default.
- [Abstract types](/docs/concepts/abstract-types): what an abstract-type component does and does not
  hide.
- [Namespaces](/docs/concepts/namespaces): packaging a reusable sub-assembly of wirings.
- [Type classes](./type-classes.md): the modular-type-classes result and where CGP sits on its
  spectrum.
- [Dependency injection](./dependency-injection.md): the same wiring seen from the container side.

## Sources

The OCaml snippets were compiled with OCaml 5.5.0; the modular-implicits snippet follows the proposal
paper, since the extension has not shipped. The CGP snippets were compiled against `cgp`
`0.8.0-alpha` with a `check_components!` assertion per wired context.

- [OCaml, *Functors*](https://ocaml.org/docs/functors) and [OCaml manual, *First-class modules* (5.5)](https://ocaml.org/manual/5.5/firstclassmodules.html): signatures, structures, functors, sharing constraints, and the applicative/generative distinction.
- [*Modules and Data Abstraction in OCaml* (Wellesley CS251)](https://cs.wellesley.edu/~cs251/s12/handouts/modules.pdf): sealing for data abstraction.
- [Dreyer, *Understanding and Evolving the ML Module System*](https://people.mpi-sws.org/~dreyer/thesis/main.pdf) and [Rossberg, *1ML*](https://www.cambridge.org/core/journals/journal-of-functional-programming/article/1ml-core-and-modules-united/47B10882829E4B32F98FBA93B28CEF30): the depth of the module system and the critique of its stratification.
- [Dreyer, Harper & Chakravarty, *Modular Type Classes* (POPL 2007)](https://people.mpi-sws.org/~dreyer/papers/mtc/main-long.pdf): classes as signatures, instances as structures and functors.
- [White, Bour & Yallop, *Modular implicits* (2014)](https://arxiv.org/abs/1512.01895) and the [experimental fork](https://github.com/ocamllabs/ocaml-modular-implicits): implicit module parameters, implicit functors, and the `Show` example.
- [*Programming Unikernels in the Large via Functor Driven Development*](https://arxiv.org/pdf/1905.02529): functor plumbing at scale in MirageOS and the Functoria DSL.
- [Applicative vs. generative functors (OCaml discussion)](https://discuss.ocaml.org/t/practical-example-of-applicative-vs-generative-functors/13777): the practical distinction.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
