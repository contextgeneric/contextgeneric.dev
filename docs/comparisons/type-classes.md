---
sidebar_label: 'Type classes'
sidebar_position: 3
description: 'CGP read against Haskell, Agda, and Lean type classes, coherence, and the overlapping and incoherent instance extensions.'
---

# Type classes

CGP lets one Rust trait have several named implementations and lets each context choose among them,
which relaxes the one-instance-per-type limit that type-class coherence imposes. It is a language
extension for Rust, with pluggable trait implementations at compile-time, implemented as a library
on stable Rust whose consumer traits are ordinary Rust traits; the [Introduction](/docs/) covers the
basics. Rust traits are Rust's type classes, so CGP works inside a type-class system and changes only
how an implementation is chosen. For readers who know type classes from Haskell, Agda, or Lean, this
page maps classes and instances onto CGP, compares CGP with overlapping and incoherent instances,
and states where coherent type classes remain the better tool.

## In your terms

A **context** is the type a CGP method runs on, supplying its data through fields and its
implementations through wiring. In the dictionary-passing account, the context is the dictionary
that carries every other dictionary.

The type-class vocabulary maps onto CGP as follows:

| In a type-class language | In CGP |
| --- | --- |
| A class | A **component**: one trait with many possible implementations |
| An instance, anonymous and canonical | A **provider**: a named, selectable implementation |
| Instance resolution by the compiler | **Wiring**, written by hand in a `delegate_components!` table |
| A class constraint on a function | An **[impl-side dependency](/docs/reference/glossary#impl-side-dependency)**, declared with `#[uses]` |
| The dictionary passed as a hidden argument | The context |
| A class's associated type | An [abstract type](/docs/reference/glossary#abstract-type), chosen by the context |

## The idea, briefly

Type classes give overloading a principled basis. A class declares an interface, an instance
implements it for a type, and a constrained function works for every type with an instance, with the
compiler finding the instance. Wadler and Blott introduced them to "make ad-hoc polymorphism less ad
hoc" ([Wadler & Blott, 1989](https://dl.acm.org/doi/10.1145/75277.75283)):

```haskell
class Show a where
  show :: a -> String

instance Show Bool where
  show True  = "True"
  show False = "False"

describe :: Show a => a -> String
describe x = "value: " ++ show x
```

Underneath, the class is a *dictionary*, a record of the class methods. An instance is a dictionary
value, and `describe` elaborates to a function that takes the dictionary as an extra hidden argument.
Dictionary passing is the implementation model behind every system on this page, including CGP. It is
the same idea as the evidence passing on the [algebraic effects](./algebraic-effects.md) page and the
qualified-type constraints on the [row polymorphism](./row-polymorphism.md) page.

### Coherence: one instance per type, globally

*Coherence* makes automatic resolution safe. For a given class and type there is one instance, and
every resolution anywhere in the program finds the same one. The property divides into
*confluence*, *coherence*, and *global uniqueness*. GHC guarantees the first two within a compilation
and does not enforce the third across a whole program
([Yang, *Type classes: confluence, coherence and global uniqueness*](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)).
The standard example of the benefit is a `Set` of an ordered element type. With one `Ord` for that
type, values inserted under one ordering can never be read back under another.

Rust makes the same choice and enforces it more strictly. Rust traits are type classes with a hard
[orphan rule](/docs/reference/glossary#orphan-rule): an impl is allowed only when the crate owns the
trait or the type ([RFC 2451](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html)),
while GHC permits orphan instances and can warn about them. CGP is built to work within Rust's
rule, and the [Rust proposals](./rust-language-proposals.md) page covers what Rust itself has
considered doing about it.

### Overlapping instances

GHC's first extension to the rule lets instances overlap when one is strictly more specific. It is
enabled through per-instance pragmas:

```haskell
instance {-# OVERLAPPABLE #-} Show a => Show [a] where   -- lists in general
  show xs = "[" ++ intercalate "," (map show xs) ++ "]"

instance {-# OVERLAPPING #-} Show [Char] where           -- but strings specially
  show s = s
```

Resolution commits to an instance only when it is strictly more specific than every other match.
GHC's own manual warns that "overlapping instances must be used with care as they can give rise to
incoherence (different instance choices are made in different parts of the program)"
([GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)).

### Incoherent instances

The second extension lets the compiler commit to an instance even when the choice is not unique. An
instance marked `{-# INCOHERENT #-}` may be selected where a more specific one could later apply, so
different parts of a program can resolve the same constraint to different dictionaries. The manual
states the danger: GHC's optimiser "assumes that type-classes are coherent, and hence it may replace
any type-class dictionary argument with another dictionary of the same type", so incoherence "may
cause unexpected results" ([GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)).

### Instance arguments in Agda

Agda provides class-style overloading without a class construct and without coherence. A record
plays the role of the class, and a double-braced argument is resolved from the scope at the call site
([Devriese & Piessens, 2011](https://dl.acm.org/doi/10.1145/2034574.2034796)):

```agda
record Show (A : Set) : Set where
  field show : A → String

instance
  showBool : Show Bool
  showBool = record { show = λ b → if b then "true" else "false" }

print : {A : Set} → {{Show A}} → A → String
print {{s}} x = Show.show s x
```

Resolution succeeds when exactly one instance in scope matches. An ambiguity is an error at that use
site rather than a violation of a program-wide guarantee.

### Type classes in Lean: search, priorities, and the diamond problem

Lean resolves classes with a backtracking, tabled search that respects per-instance priorities:

```lean
class Show (α : Type) where
  display : α → String

instance : Show Bool where
  display b := if b then "true" else "false"

#eval Show.display true   -- "true"
```

Lean allows multiple instances, so the same constraint can be satisfied in more than one way. When
two resolution paths disagree, which is called a *diamond*, inference can pick the wrong instance or
diverge ([Selsam, Ullrich & de Moura, *Tabled Typeclass Resolution*](https://arxiv.org/pdf/2001.04301)).
In a proof assistant, two instances that are not definitionally equal break proofs that assume they
coincide. The Mathlib community therefore keeps overlapping instances definitionally equal as a
standing practice ([Baanen, 2022](https://arxiv.org/pdf/2202.01629)).

### Modular type classes

Dreyer, Harper, and Chakravarty showed that type classes can be expressed as ML modules with a
resolution layer added: **classes as signatures, and instances as structures and functors**. Certain
instance modules are designated *canonical* within a scope, so the compiler can resolve them
implicitly ([*Modular Type Classes*](https://people.mpi-sws.org/~dreyer/papers/mtc/main-long.pdf)).

The paper identifies a tension that every design on this page must resolve: **canonicity conflicts
with modularity.** A canonical instance makes implicit resolution safe, but canonicity is not a
modular property, since two modules can each supply a different instance. The designs fall along a
range:

- **Haskell:** Fully implicit resolution, global coherence, and one instance per type.
- **Modular type classes, OCaml's modular implicits, Agda instance arguments, and Scala implicits:**
  Implicit resolution with canonicity scoped or dropped, at the cost of possible ambiguity.
- **CGP:** No canonicity and no search. Each context selects its implementations explicitly.

The [ML modules](./ml-modules.md) page develops the module side.

## How CGP expresses it

CGP keeps Rust's type classes and replaces the automatic choice of an implementation with explicit
selection per context. It splits each class into a consumer trait that callers use and a provider
trait that implementations target. The consumer trait is an ordinary Rust trait. The provider trait
and the wiring let implementations be named and chosen.

### A component is a class; a provider is a first-class instance

Declaring a component declares a class:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

`CanCalculateArea` is the class interface, as `class Show a` is. The two systems differ in what an
instance can be. A Haskell instance is anonymous and canonical: there is one `Show Bool`, and the
compiler chooses it. A CGP provider is a named marker type that carries a provider-trait impl, a
*first-class dictionary* that code can name and choose among:

```rust
#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! { Rectangle { AreaCalculatorComponent: RectangleArea } }
```

Implementing the provider trait on a provider type, rather than on the context, removes the
one-instance-per-type limit. Each provider implements the provider trait for its *own* type, so
coherence does not forbid a second provider. The wiring line takes the place of resolution: it states
the choice that Haskell's compiler would find by searching for the canonical instance. A provider's
`#[uses]` imports correspond to the class constraints that dictionary passing threads through, and
the context carries them. `Rectangle` here is a
**[value context](/docs/reference/glossary#value-context)**: the type being measured also carries
the wiring, and the component is [self-targeted](/docs/reference/glossary#self-targeted-component).

### Overlapping providers need no specificity rule

Haskell needs pragmas and a most-specific rule to permit overlap. CGP allows any number of
overlapping providers, and a context chooses among them by name rather than by specificity. Three
encoders that apply to overlapping sets of types compile side by side:

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

#[cgp_impl(new EncodeAsHex)]
impl<Value: AsRef<[u8]>> Encoder<Value> {
    fn encode(&self, value: &Value) -> Vec<u8> {
        value.as_ref().iter().flat_map(|byte| format!("{byte:02x}").into_bytes()).collect()
    }
}
```

As type-class instances these would conflict: `String` matches the first pair, and `Vec<u8>` matches
the second. Even with `OVERLAPPING`, the compiler would need one instance to be strictly more
specific. As providers, all three compile, because each implements the provider trait for its own
marker type. CGP has no most-specific rule, so the overlap creates no ambiguity. The problem GHC
warns about, overlap that silently produces incoherence, does not arise here, because CGP never
infers the choice.

The encoded value is the `Value` parameter rather than `Self`, so the contexts that wire these
providers are **[environmental contexts](/docs/reference/glossary#environmental-context)**, types that
stand for an application. The component is
[parameter-targeted](/docs/reference/glossary#parameter-targeted-component).

### Incoherent choices made explicit and local

CGP allows several implementations for one type, the situation that type-class languages guard
against, and makes it safe by moving the choice from a global search to an explicit table in each
context:

```rust
pub struct AppA;
pub struct AppB;

delegate_components! {
    AppA {
        open EncoderComponent;
        @EncoderComponent.String: EncodeWithDisplay,
        @EncoderComponent.Vec<u8>: EncodeAsHex,
    }
}

delegate_components! {
    AppB {
        open EncoderComponent;
        @EncoderComponent.String: EncodeBytes,
        @EncoderComponent.Vec<u8>: EncodeBytes,
    }
}
```

Each context selects one provider per component and per value type. Two contexts can therefore
resolve the same type differently *on purpose*, while the choice within one context is unambiguous
and fixed. `AppA` encodes a `Vec<u8>` as hexadecimal and `AppB` passes the bytes through, and each
context is consistent on its own terms. In Haskell, different choices in different parts of a program
are a hazard, and in Lean they produce a diamond. In CGP each choice is written in a context's table,
so a reader can see which one applies. The `open` statement and the `@`-path keys are documented on
the [`delegate_components!`](/docs/reference/macros/delegate_components) page.

### Providers avoid orphan conflicts and newtypes

A provider's `Self` is always a type that its crate owns, and this removes two common workarounds.
The orphan rule is always satisfied, so a downstream crate can supply a provider for a component and
a type it did not define. A second behavior for a type is a second provider, named directly, so the
type does not need a newtype such as the `Sum` and `Product` wrappers around `Int`. The
[Bypassing coherence](/docs/concepts/coherence) page shows both on the encoder example.

## What each approach costs

Type classes are valued for principled, inferred overloading and for global uniqueness. The costs
their users describe follow from the same rules. The one-instance-per-type limit requires the
newtype workaround, which "breaks down when the type is embedded in another type"
([Yang, 2014](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)).
The orphan rule shapes module structure ([Queensland FP Lab](https://qfpl.io/posts/orphans-and-fundeps/)).
Overlapping instances are subtle, and incoherent instances, by GHC's own account, "may cause
unexpected results" ([GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)).
Haskell has no simple form of local instance, because local instances reintroduce a coherence
problem.

In Agda and Lean, which do not enforce coherence, the costs appear as the diamond problem and the work
of keeping overlapping instances definitionally equal ([Baanen, 2022](https://arxiv.org/pdf/2202.01629)).
Some practitioners argue that the coherence bargain is the wrong one and prefer explicit dictionaries
([Chiusano, *The trouble with typeclasses*](https://pchiusano.github.io/2018-02-13/typeclasses.html)).
A recent survey compares where Swift, Rust, Scala, and Haskell each draw the line
([Racordon, Flesselle & Pham, 2025](https://arxiv.org/pdf/2502.20546)).

CGP's main cost is the wiring. CGP does not search for an implementation, so each context must write
its selection down. A component must be declared with
[`#[cgp_component]`](/docs/reference/macros/cgp_component) before it can have providers, which adds
declarations that a plain trait does not need. Trait resolution over the wiring adds compile-time
work. The raw diagnostics are trait-solver output over generated types:
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where coherent type classes are the better choice

Coherent type classes are the right tool when a program wants one canonical instance per type across
the whole program: one `Ord`, one `Show`, one serialization. They guarantee that a `Set` cannot be
corrupted by a second ordering, and generic code needs no wiring. Reproducing that uniqueness with
CGP's per-context wiring would mean maintaining by hand what the compiler already guarantees. The
same holds in Rust: a trait with one implementation per type is an ordinary trait, and the
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page starts from that case.

CGP's explicit selection fits a program that needs several interchangeable instances, a different
choice per context, instances for types and traits it does not own, or a way around the diamond and
orphan problems.

## What to expect that differs

**CGP does not find the instance by type.** A context names the provider in its table. Because CGP
does not search for an instance, it does not depend on each type having a unique one, so it can hold
the overlapping and orphan implementations that Haskell forbids or treats as fragile.

**Uniqueness is per context, not program-wide.** A type-class reader may expect that once a type has
an instance, it is the instance everywhere. CGP guarantees one choice within a context and lets two
contexts differ, so two applications can encode one type in two ways without conflict.

**Selection can be keyed on the context rather than the type.** Haskell keys `Show Bool` on `Bool`,
so there is one such instance. The encoder example moves the type being operated on into a
parameter and keys the choice on the context. Changing the key makes per-context selection different
from per-type instances, rather than a new name for them. CGP can also key the choice on the type, as
the `Rectangle` example does, and then it has the same one-choice-per-type limit. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page works out the difference between the
two arrangements.

**Wiring checks take the place of instance resolution's checks.** A type-class compiler proves that
an instance exists at each use. [`check_components!`](/docs/reference/macros/check_components)
proves that a context's wiring is complete, and it names any missing dependency at the wiring site.

## Where to go next

These pages develop the constructs and the neighbouring comparisons:

- [Bypassing coherence](/docs/concepts/coherence): the coherence rules and the provider move, on the
  encoder example.
- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): the two traits one
  class becomes, and how a call finds its provider.
- [ML modules](./ml-modules.md): the module side of the modular-type-classes correspondence.
- [Implicit parameters](./implicit-parameters.md): the same trade seen from Scala's `given` and
  Haskell's `ImplicitParams`.
- [Rust's own proposals](./rust-language-proposals.md): what Rust has considered doing about its own
  coherence rules.

## Sources

The Haskell snippets were compiled with GHC 9.10, the Agda snippet type-checked with Agda 2.8.0, and
the Lean snippet evaluated with Lean 4.34.0. The CGP snippets were compiled against `cgp`
`0.8.0-alpha` with a `check_components!` assertion per wired context.

- [Wadler & Blott, *How to make ad-hoc polymorphism less ad hoc* (POPL 1989)](https://dl.acm.org/doi/10.1145/75277.75283): the origin of type classes and the dictionary-passing translation.
- [GHC User's Guide, *Instance declarations and resolution*](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html): the one-instance rule, the orphan-instance rule, the overlap pragmas, and GHC's own warnings about incoherence.
- [Yang, *Type classes: confluence, coherence and global uniqueness*](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/): the decomposition of coherence, the `Set`/`Ord` argument, and the newtype workaround's limits.
- [Rust RFC 2451, *re-rebalancing coherence*](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html): Rust traits as type classes with an enforced orphan rule.
- [Devriese & Piessens, *On the Bright Side of Type Classes: Instance Arguments in Agda* (ICFP 2011)](https://dl.acm.org/doi/10.1145/2034574.2034796): scoped, type-directed resolution without a coherence guarantee.
- [Selsam, Ullrich & de Moura, *Tabled Typeclass Resolution*](https://arxiv.org/pdf/2001.04301) and [Baanen, *Use and abuse of instance parameters in the Lean mathematical library*](https://arxiv.org/pdf/2202.01629): Lean's resolution, the diamond problem, and Mathlib's definitional-equality discipline.
- [Dreyer, Harper & Chakravarty, *Modular Type Classes* (POPL 2007)](https://people.mpi-sws.org/~dreyer/papers/mtc/main-long.pdf): classes as signatures, instances as structures and functors, and the canonicity-versus-modularity tension.
- [Chiusano, *The trouble with typeclasses*](https://pchiusano.github.io/2018-02-13/typeclasses.html), [Racordon, Flesselle & Pham, *On the State of Coherence in the Land of Type Classes* (2025)](https://arxiv.org/pdf/2502.20546), and [Queensland FP Lab, *Multi-Parameter Type Classes and their Orphan Rules*](https://qfpl.io/posts/orphans-and-fundeps/): community sentiment on coherence, the case for explicit dictionaries, and the orphan-instance pain.
- [Kmett, *Typeclasses vs the World* (2015)](https://www.youtube.com/watch?v=hIZxTQP1ifo): what coherence buys and costs, and the alternatives that trade it for first-class instances. CGP's author names this talk as a core inspiration in the [RustLab 2025 talk](/blog/rustlab-2025-coherence).

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
