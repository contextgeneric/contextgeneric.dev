---
sidebar_label: 'Type classes'
sidebar_position: 3
description: 'CGP read against Haskell, Agda, and Lean type classes, coherence, and the overlapping and incoherent instance extensions.'
---

# Type classes

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who knows type
classes from Haskell, Agda, or Lean. Rust traits are Rust's type classes, so CGP already lives inside
a type-class system, and its one change is to the *coherence* rule: it makes instances first-class
values selected explicitly per context, which is the freedom overlapping and incoherent instances
reach for without making it safe. The page covers the class and instance correspondence, the
coherence trade, where coherent type classes remain the better tool, and what a type-class reader
should expect to differ.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. In the dictionary-passing account it is the dictionary that carries every other dictionary.

| In a type-class language | In CGP |
| --- | --- |
| A class | A **component**: one trait with many possible implementations |
| An instance, anonymous and canonical | A **provider**: a named, selectable implementation |
| Instance resolution by the compiler | **Wiring**, written by hand in a `delegate_components!` table |
| A class constraint on a function | An **impl-side dependency**, declared with `#[uses]` |
| The dictionary passed as a hidden argument | The context |
| A class's associated type | An abstract type, chosen by the context |

## The idea, briefly

Type classes make overloading principled. A class declares an interface, an instance implements it
for a type, and a constrained function works for every type with an instance, with the compiler
finding the instance. Wadler and Blott introduced them to "make ad-hoc polymorphism less ad hoc"
([Wadler & Blott, 1989](https://dl.acm.org/doi/10.1145/75277.75283)):

```haskell
class Show a where
  show :: a -> String

instance Show Bool where
  show True  = "True"
  show False = "False"

describe :: Show a => a -> String
describe x = "value: " ++ show x
```

Underneath, the class is a *dictionary*, a record of the class methods; an instance is a dictionary
value; and `describe` elaborates to a function that takes the dictionary as an extra hidden argument.
Dictionary passing is the implementation model behind every system on this page, and behind CGP. It
is the same idea as the evidence passing on the [algebraic effects](./algebraic-effects.md) page and
the qualified-type constraints on the [row polymorphism](./row-polymorphism.md) page.

### Coherence: one instance per type, globally

*Coherence* makes silent, automatic resolution safe. For a given class and type there is one
instance, and every resolution anywhere in the program finds the same one. The property decomposes
into *confluence*, *coherence*, and *global uniqueness*; GHC guarantees the first two within a
compilation and does not enforce the third across a whole program
([Yang, *Type classes: confluence, coherence and global uniqueness*](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)).
The canonical benefit is a `Set` of an ordered element type: with one `Ord` for that type, values
inserted under one ordering and read under another can never disagree.

Rust makes the same choice and *enforces* it where Haskell advises. Rust traits are type classes with
a hard orphan rule, so an impl is allowed only when the crate owns the trait or the type
([RFC 2451](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html)). That strictness
is the rule CGP is built to work within, and the [Rust proposals](./rust-language-proposals.md) page
covers what Rust itself has considered doing about it.

### Overlapping instances

The first extension lets instances overlap when one is strictly more specific. GHC exposes it through
per-instance pragmas:

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

Agda offers class-style overloading without a class construct and without coherence. A record plays
the class, and a double-braced argument is resolved from the call-site scope
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

Resolution succeeds when exactly one instance in scope matches, and an ambiguity is a local error at
that use site rather than a program-wide guarantee.

### Type classes in Lean: search, priorities, and the diamond problem

Lean resolves classes by a backtracking tabled search with per-instance priorities:

```lean
class Show (α : Type) where
  display : α → String

instance : Show Bool where
  display b := if b then "true" else "false"

#eval Show.display true   -- "true"
```

Because Lean allows multiple instances, the same constraint can be satisfiable in more than one way,
and when two resolution paths disagree, a *diamond*, inference can pick the wrong one or diverge
([Selsam, Ullrich & de Moura, *Tabled Typeclass Resolution*](https://arxiv.org/pdf/2001.04301)). In
a proof assistant two instances that are not definitionally equal break proofs that assume they
coincide, so the Mathlib community keeps overlapping instances definitionally equal as a standing
discipline ([Baanen, 2022](https://arxiv.org/pdf/2202.01629)).

### Modular type classes

Dreyer, Harper, and Chakravarty showed that type classes are ML modules plus a resolution layer:
**classes as signatures and instances as structures and functors**, with certain instance modules
designated *canonical* within a scope so the compiler can resolve them implicitly
([*Modular Type Classes*](https://people.mpi-sws.org/~dreyer/papers/mtc/main-long.pdf)). The paper
names the tension every design on this page negotiates: **canonicity fights modularity.** A canonical
instance makes implicit resolution safe, and it is a non-modular property, since two modules can each
supply a different one. The design space is a spectrum. At one end sits Haskell: fully
implicit resolution, global coherence, one instance per type. In the middle sit modular type classes,
OCaml's modular implicits, Agda instance arguments, and Scala implicits: implicit resolution with
canonicity scoped or dropped, paid for in ambiguity. CGP sits at the far end: no canonicity, no
search, and explicit per-context selection. The [ML modules](./ml-modules.md) page develops the
module side.

## How CGP expresses it

CGP is a type-class system with the coherence rule removed and explicit per-context selection in its
place, built by splitting each class into a consumer trait callers use and a provider trait
implementations target. The consumer trait is an ordinary Rust type class. The provider trait and the
wiring are what make instances first-class values.

### A component is a class; a provider is a first-class instance

Declaring a component is declaring a class:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

`CanCalculateArea` is the class interface, as `class Show a` is. The two systems differ in what an
instance may be. A Haskell instance is anonymous and canonical: there is one `Show Bool`, chosen by the
compiler. A CGP provider is a named marker type carrying a provider-trait impl, a *first-class
dictionary* you can name and choose among:

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

Moving the implementation off the context and onto a provider marker is the single device that lifts
the one-instance-per-type limit: because a provider implements the provider trait for *its own* type,
coherence never forbids a second one. The wiring line plays the role of resolution, explicitly, where
Haskell's compiler would search for the canonical instance. A provider's `#[uses]` imports are the
class constraints threaded by dictionary passing, and the context is the dictionary that carries them.
`Rectangle` here is a **value context**: the type being measured also carries the wiring, and the
component is self-targeted.

### Overlapping instances are the default, with no heuristic

Where Haskell needs pragmas and a most-specific heuristic to permit overlap, CGP permits unlimited
overlapping providers by construction and resolves them by naming rather than guessing. Two encoders
that both apply to `String` compile side by side:

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

As type-class instances two of these would be rejected, since `String` matches the first pair and
`Vec<u8>` matches the second, and even with `OVERLAPPING` the compiler would need one to be strictly
more specific. As providers all three compile, because each implements the provider trait for its
own marker. There is no most-specific rule and therefore no ambiguity. The fragility
GHC warns about, overlap silently producing incoherence, cannot arise, because the choice is never
inferred. Here the encoded value has moved from `Self` into the `Value` parameter, so the contexts
that wire these providers are **environmental contexts**, types standing for an application, and the
component is parameter-targeted.

### Incoherent instances made deterministic and local

CGP keeps the many-instances-for-one-type situation that type-class languages fear and makes it safe
by moving the choice from a global search to an explicit per-context table:

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

Each context selects one provider per component and per value type, so two contexts resolve the same
type differently *on purpose*, and within one context the choice is unambiguous and fixed. `AppA`
encoding a `Vec<u8>` as hexadecimal and `AppB` passing the bytes through are two coherent local
scopes, not a global incoherence. The different-choice-in-different-places that is a hazard in
Haskell and a diamond in Lean is a feature here because it is written down. In one line, CGP is
incoherent instances with the incoherence made deterministic and the selection made explicit. The
`open` statement and the `@`-path keys are documented on the
[`delegate_components!`](/docs/reference/macros/delegate_components) page.

### No orphan rule, no newtype

Two everyday frustrations disappear because a provider's `Self` is always a type its crate owns. The
orphan rule never applies, so a downstream crate can supply a provider for a component and a type it
did not define. And the newtype a type needs for a second instance, such as `Sum` and `Product`
wrapping `Int`, is unnecessary: a second behavior for a type is a second provider, named directly.
The [Bypassing coherence](/docs/concepts/coherence) page shows both on the encoder example.

## What each approach costs

Type classes are valued for principled, inferred overloading and for global uniqueness, and their
costs, as their users state them, follow from the same rules. The one-instance-per-type limit forces
the newtype workaround, which "breaks down when the type is embedded in another type"
([Yang, 2014](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)).
The orphan rule shapes module structure ([Queensland FP Lab](https://qfpl.io/posts/orphans-and-fundeps/)).
Overlapping instances are subtle and incoherent instances are, by GHC's own account, indeterministic
([GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)). Haskell
has no easy local instance, because local instances reintroduce a coherence problem. In Agda and Lean
the absence of enforced coherence surfaces as the diamond problem and the work of keeping overlapping
instances definitionally equal ([Baanen, 2022](https://arxiv.org/pdf/2202.01629)). Some practitioners
argue the coherence bargain is wrong and prefer explicit dictionaries
([Chiusano, *The trouble with typeclasses*](https://pchiusano.github.io/2018-02-13/typeclasses.html)),
and a recent survey compares how Swift, Rust, Scala, and Haskell each draw the line
([Racordon, Flesselle & Pham, 2025](https://arxiv.org/pdf/2502.20546)).

CGP pays with the wiring. There is no search, so the selection must be written down, once per context.
A component has to be declared with [`#[cgp_component]`](/docs/reference/macros/cgp_component)
before it can have providers, which adds declarations a plain trait does not need. The compile-time
work is real. And the raw diagnostics are trait-solver output over generated types:
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where coherent type classes are the better choice

When a program wants one canonical instance per type program-wide, one `Ord`, one `Show`, one
serialization, so that a `Set` cannot be corrupted and generic code needs no wiring, coherent type
classes are the right tool. Simulating that with CGP's per-context wiring reintroduces by hand the
uniqueness the compiler gives for free. The same holds in Rust: a trait with one implementation per
type is an ordinary trait, and CGP's [Modularity Hierarchy](/docs/concepts/modularity-hierarchy)
page says so first. CGP's explicit selection is the better tool when a program needs several
interchangeable instances, per-context choice, instances for types and traits it does not own, or an
escape from the diamond and orphan problems.

## What to expect that differs

**CGP does not find the instance by type.** The provider is named in a table. This is the consequence
of the property a type-class reader will find most striking: because CGP does not resolve by type, it
is free of coherence, so it hosts the overlapping and orphan instances Haskell forbids or makes
fragile.

**Uniqueness is per context, not program-wide.** A type-class reader may expect that once a type has
an instance it is the instance everywhere. CGP promises agreement within a context and lets two
contexts differ, and that scoping lets two applications encode one type two ways without conflict.

**Selection can be keyed on the context rather than the type.** Haskell keys `Show Bool` on `Bool`,
so there is one of it. CGP's decisive shape moves the type being operated on into a parameter and
keys the choice on the context, and that change of key makes per-context selection more than a rewording of
per-type instances. CGP can also key on the type in the Haskell manner, and then it inherits the same
one-per-type limit. The [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page works out
the difference between the two arrangements.

**The verification you gave up is replaced, not removed.** Where a type-class compiler proves an
instance exists, [`check_components!`](/docs/reference/macros/check_components) proves a context's
wiring is complete, at the wiring site, naming the missing dependency.

## Where to go next

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
