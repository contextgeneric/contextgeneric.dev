---
sidebar_label: 'ML modules'
sidebar_position: 4
description: 'CGP read against OCaml and Standard ML signatures, structures, functors, and modular implicits.'
---

# ML modules and modular implicits

CGP lets Rust programs assemble named implementations through a context's wiring table. For an
ML programmer, its components, providers, and [higher-order providers](/docs/reference/glossary#higher-order-provider) play roles similar to
signatures, structures, and functors. The correspondence explains how to compose implementations;
ML's sealing and module type identities need a separate account.

CGP is a language extension for Rust, with pluggable trait implementations at compile-time. It is
implemented as a library on stable Rust, and its consumer traits are ordinary Rust traits. The
[Introduction](/docs/) covers the basics. This page assumes familiarity with OCaml or Standard ML
modules and also compares the proposed OCaml modular-implicits extension.

## In your terms

A **context** is the type on which CGP's consumer methods operate. It supplies runtime data and
selects providers for its components. An [application context](/docs/reference/glossary#application-context) can gather the choices that an ML
program records through module bindings and functor applications.

The vocabulary maps by role, with limits explained below:

| In ML | In CGP |
| --- | --- |
| A signature | A **component** declaring an interface |
| A structure satisfying a signature | A **provider** implementing a declared provider trait |
| A functor | A **higher-order provider** parameterized by providers |
| An abstract `type t` in a signature | An associated type supplied through an abstract-type component |
| Module bindings and functor applications assembling a program | Provider choices in `delegate_components!` |
| Checking a functor argument against its signature | Provider trait bounds, with `check_components!` checking a concrete context's dependencies |
| Modular implicits' type-directed selection | Explicit provider selection per context and component |

## The idea, briefly

ML modules separate an implementation from the interface its clients can use. Signatures describe
types and operations, structures supply them, and functors build modules from other modules. This
separation supports reusable implementations and representation hiding.

### Signatures, structures, and abstract types

A signature can require operations on a type without exposing the type's representation. Here,
`OrderedType` requires a type `t` and a comparison operation, while `StringCompare` supplies both:

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

`StringCompare.t` remains visibly equal to `string` in this example. Ascribing the structure to a
signature that leaves `t` abstract would hide that equality from clients. This is the distinction
between providing a type and sealing its representation behind an interface.

### Functors

A functor makes a module depend on another module through a signature. The following skeleton
shows the application shape; OCaml's `Set.Make` supplies the complete set implementation:

```ocaml
module Make (O : OrderedType) = struct
  (* ... a set implementation using O.compare ... *)
end

module StringSet = Make (StringCompare)
```

`Make (StringCompare)` explicitly chooses the implementation of the dependency. When clients need
to know that a result type equals an argument type, a signature can expose that equality with a
constraint such as `with type elt = O.t`. These equalities let independently described modules
exchange values of the same type. [OCaml's functor guide](https://ocaml.org/docs/functors) develops
the set example and dependency injection through modules.

Functor application also determines which [abstract types](/docs/reference/glossary#abstract-type) are equal. OCaml's applicative functors
can preserve result type equality across applications to the same module path. Generative functors
can introduce fresh type identities; OCaml provides unit-parameter functors for this purpose.
Standard ML uses generative semantics, but an explicitly shared or manifest type need not become
fresh at every application. The distinction concerns type identity, not merely the syntax for
calling a functor. See the [OCaml manual on generative functors](https://ocaml.org/manual/5.5/generativefunctors.html).

### Assembling a program

A large graph of functor applications can require substantial configuration code. MirageOS uses
Functoria to describe and assemble such graphs, including configuration choices. Its authors'
[*Functor Driven Development*](https://arxiv.org/pdf/1905.02529) paper explains the application
structure that motivated the tool. This is a useful comparison for CGP's wiring, though Functoria
also handles concerns beyond selecting an implementation for an interface.

### Modular implicits

Modular implicits proposes automatic selection of module arguments from implicit modules in scope.
The function declares an implicit module parameter, and resolution uses the signature and type
constraints to find an argument:

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

`Show_list` composes an implementation for lists from an implementation for their elements. In this
example, resolution applies it to `Show_int`. The proposal allows different implementations in
different scopes; it does not require a single global instance for each type. Resolution must
still reject ambiguous choices. See [White, Bour, and Yallop's proposal](https://arxiv.org/abs/1512.01895).

This syntax belongs to the modular-implicits proposal and its
[experimental implementation](https://github.com/ocamllabs/ocaml-modular-implicits). It is not a
standard OCaml example. The other OCaml snippets retain the verification record in Sources.

### Modules and type classes

The relationship between module interfaces and type-class interfaces has a formal precedent.
Dreyer, Harper, and Chakravarty's
[*Modular Type Classes*](https://people.mpi-sws.org/~dreyer/papers/mtc/main-long.pdf) presents classes
as signatures and instances through structures and functors. That work helps explain the analogy
here; it is not a formal equivalence between ML modules and CGP. The
[type classes](./type-classes.md) comparison develops the related selection tradeoffs.

## How CGP expresses it

CGP uses Rust traits to declare interfaces and provider types to name implementations. A context's
wiring selects those providers, while provider bounds state which other interfaces they require.

### Components are signatures; providers are structures

A component and its providers separate the interface from its named implementations:

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

`CanCalculateArea` is the interface callers use, and `RectangleArea` implements its generated
provider trait, `AreaCalculator`. The provider reads `width` and `height` from the context through
[implicit arguments](/docs/concepts/implicit-arguments). A `Rectangle` wired to this provider is a
**[value context](/docs/reference/glossary#value-context)**: the value being measured also determines the implementation.

The consumer/provider split is specific to CGP's use of Rust traits. Distinct provider types can
implement the same provider trait for a context while respecting Rust's [coherence](/docs/reference/glossary#coherence) rules; wiring
then selects one for consumer calls. ML already names different structures satisfying a signature.
The [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) page explains this
split in detail.

Components can group multiple methods, associated types, and constants. This example uses one
operation because it is independently replaceable. A larger ML signature may correspond to one
component or several, depending on which choices should vary together.

### Abstract-type components are a signature's abstract types

An abstract-type component lets generic code use a type that the context supplies:

```rust
#[cgp_type]
pub trait HasErrorType {
    type Error: Debug;
}
```

`HasErrorType` declares an `Error` with a `Debug` bound. Generic code can import it through
`#[use_type(HasErrorType.Error)]` and use the name `Error`, much as a functor uses `O.t` without
knowing its representation. Wiring chooses the associated type for a concrete context.

This mechanism does not itself seal the selected type. If wiring exposes an equality with a
concrete public type, clients can use that equality. Rust's module privacy and types with private
representations remain available for data abstraction. The
[Abstract types](/docs/concepts/abstract-types) page distinguishes configuring a type from hiding
its representation.

### Higher-order providers are functors

A higher-order provider composes behavior from another provider supplied as a type parameter:

```rust
#[cgp_impl(new ScaledArea<InnerCalculator>)]
#[use_provider(InnerCalculator: AreaCalculator)]
impl<InnerCalculator> AreaCalculator {
    fn area(&self, #[implicit] scale_factor: f64) -> f64 {
        InnerCalculator::area(self) * scale_factor * scale_factor
    }
}
```

`ScaledArea<RectangleArea>` chooses `RectangleArea` as the inner implementation and uses the same
context for both calls. Its `AreaCalculator` bound plays the role of a functor's parameter
signature. The context supplies the rectangle dimensions and `scale_factor`.

The example requires an explicit `InnerCalculator` argument. A provider can separately declare a
default parameter of [`UseContext`](/docs/reference/providers/use_context), which forwards that
dependency through the context's wiring. The macro's `new` form above does not add such a default.
Forwarding also needs a dependency path that terminates: wiring an area wrapper back to itself
through `UseContext` would not select a different inner area calculator.

A shared context can reduce the need to repeat type equalities between providers. Providers that
use the same context's `HasErrorType` refer to the same associated `Error`. Other relationships
between associated types may still require equality bounds. This convenience does not replace
ML's full account of sharing constraints or abstract type identity.

### `delegate_components!` records the assembly choices

A wiring table names the provider for each component without listing those entries in dependency
order. Providers obtain their declared dependencies through the same context:

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

`App` is an **[environmental context](/docs/reference/glossary#environmental-context)** representing the application. Its `smtp_server` field supplies
the runtime data used by `SendViaSmtp`, while the table selects behavior and the error type. The
entry order has no execution meaning: the table does not construct objects or schedule startup.
This is the part of functor assembly that the comparison covers, rather than all of Functoria's
configuration and build functions.

Concrete wiring needs a dependency check as well as a provider selection. Rust checks each provider
body against its declared bounds; [`check_components!`](/docs/reference/macros/check_components)
asserts that a specified context satisfies the selected components' transitive requirements.
Writing a table alone does not force every entry to be usable. An
[aggregate provider](/docs/concepts/aggregate-providers) or
[namespace](/docs/concepts/namespaces) can package choices for reuse.

### Modular implicits select by scope; CGP selects through the context

Modular implicits and CGP both support composing implementations from other implementations. Their
selection mechanisms differ: modular implicits searches the implicit modules in scope under type
constraints, while CGP's wiring names a provider for a context and component. Both allow alternative
implementations; they place the choice in different parts of the program.

Explicit wiring makes CGP's selected provider visible, but Rust still resolves trait bounds and
checks coherence. Wiring does not eliminate every possible ambiguity or trait-resolution failure.
Likewise, an implicit functor's recursive resolution is only an analogy for provider composition;
it is not CGP's implementation mechanism.

## What each approach costs

ML modules make interfaces, type abstraction, and module application explicit. That gives clients
control over which type equalities and implementations are visible. Larger assemblies can require
more module bindings and sharing constraints, and the distinction between applicative and generative
functors adds another type-level concern. Tools such as Functoria organize that assembly work.

CGP moves many assembly choices into a context, at the cost of generated traits and indirect
dependency resolution. A reader may need to follow the wiring to find an implementation, and a
failed bound can produce a long diagnostic. [`cargo cgp check`](/docs/cargo-cgp/check) helps explain
the error classes it recognizes. [Monomorphization](/docs/reference/glossary#monomorphization) can also increase compile time and generated code;
static provider selection permits direct calls but does not guarantee that every call is inlined.

CGP wiring supplies neither ML sealing nor runtime module selection. Rust's ordinary abstraction
and runtime-polymorphism tools can coexist with CGP, but they address those needs separately. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) helps decide when the wiring machinery
is worth introducing.

## Where ML modules are the better choice

ML modules fit programs whose design relies on signature sealing, precise control of module type
equalities, or explicit module application. In OCaml, first-class modules also package an
implementation for use as a runtime value. Those are native module-system features, and CGP does
not reproduce them as a bundle.

CGP is useful in Rust when many providers share a context and applications need to vary the
implementation choices independently. For a smaller Rust interface, ordinary traits and generic
parameters may already express the needed dependency. The choice depends on the assembly problem
as well as the language's abstraction facilities.

## What to expect that differs

**Wiring order does not determine execution order.** Component entries select providers. Runtime
initialization and execution remain ordinary Rust code.

**Type selection and representation hiding are separate.** An abstract-type component supplies an
associated type; Rust privacy determines which representation details clients can access.

**Alternative implementations exist in both systems.** ML names structures, modular implicits uses
scope and type-directed resolution, and CGP records choices on a context.

**Consumer traits remain ordinary Rust traits.** Code can implement them directly when provider
wiring would add no useful flexibility.

## Where to go next

These pages expand the constructs and comparisons used here:

- [Higher-order providers](/docs/concepts/higher-order-providers): explicit provider parameters and
  forwarding through `UseContext`.
- [Abstract types](/docs/concepts/abstract-types): selecting types through a context.
- [Namespaces](/docs/concepts/namespaces): reusable groups of wiring choices.
- [Type classes](./type-classes.md): interfaces, instances, and implementation selection.
- [Dependency injection](./dependency-injection.md): wiring viewed through dependency declarations.

## Sources

The OCaml snippets were compiled with OCaml 5.5.0; the modular-implicits snippet follows the proposal
paper, since the extension has not shipped. The CGP snippets were compiled against `cgp`
`0.8.0-alpha` with a `check_components!` assertion per wired context.

These references support the module semantics and comparison:

- [OCaml manual, *Generative functors*](https://ocaml.org/manual/5.5/generativefunctors.html): unit-parameter functors and fresh result type identities.
- [OCaml, *Functors*](https://ocaml.org/docs/functors) and [OCaml manual, *First-class modules* (5.5)](https://ocaml.org/manual/5.5/firstclassmodules.html): signatures, structures, functors, sharing constraints, and the applicative/generative distinction.
- [*Modules and Data Abstraction in OCaml* (Wellesley CS251)](https://cs.wellesley.edu/~cs251/s12/handouts/modules.pdf): sealing for data abstraction.
- [Dreyer, *Understanding and Evolving the ML Module System*](https://people.mpi-sws.org/~dreyer/thesis/main.pdf) and [Rossberg, *1ML*](https://www.cambridge.org/core/journals/journal-of-functional-programming/article/1ml-core-and-modules-united/47B10882829E4B32F98FBA93B28CEF30): the depth of the module system and the critique of its stratification.
- [Dreyer, Harper & Chakravarty, *Modular Type Classes* (POPL 2007)](https://people.mpi-sws.org/~dreyer/papers/mtc/main-long.pdf): classes as signatures, instances as structures and functors.
- [White, Bour & Yallop, *Modular implicits* (ML/OCaml 2014; published 2015)](https://arxiv.org/abs/1512.01895) and the [experimental fork](https://github.com/ocamllabs/ocaml-modular-implicits): implicit module parameters, implicit functors, and the `Show` example.
- [*Programming Unikernels in the Large via Functor Driven Development*](https://arxiv.org/pdf/1905.02529): functor plumbing at scale in MirageOS and the Functoria DSL.
- [Applicative vs. generative functors (OCaml discussion)](https://discuss.ocaml.org/t/practical-example-of-applicative-vs-generative-functors/13777): the practical distinction.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
