---
sidebar_label: 'Type classes'
sidebar_position: 3
description: 'How CGP provider selection compares with type classes, coherence, and overlapping instances in Haskell, Agda, and Lean.'
---

# Type classes

CGP lets each context select named implementations at compile time. It is a language extension built
as a [stable Rust library](/docs/) , with ordinary Rust traits as its caller interfaces. This page
compares explicit wiring with type-class resolution in Haskell, Agda, and Lean, and explains when a
single canonical implementation is the simpler choice.

## In your terms

A **context** is the type a CGP method runs on. It supplies data through fields and chooses
implementations through wiring. In dictionary-passing terms, its wiring records which dictionaries
to use.

The type-class vocabulary maps onto CGP as follows:

| In a type-class language | In CGP |
| --- | --- |
| A class interface | A component's consumer trait, used by callers |
| A class instance | A **provider**: a named implementation of the provider trait |
| Instance resolution by the compiler | **Wiring**, written by hand in a `delegate_components!` table |
| A constraint needed by an implementation | An [impl-side dependency](/docs/reference/glossary#impl-side-dependency), often declared with `#[uses]` |
| The dictionary passed as a hidden argument | The context |
| A class's associated type | An [abstract type](/docs/reference/glossary#abstract-type), chosen by the context |

## The idea, briefly

Type classes let a compiler choose an implementation for a type. A class declares an interface, an
instance implements it for a type, and a constrained function works for every type with an instance.
Wadler and Blott introduced them to "make ad-hoc polymorphism less ad hoc" ([Wadler & Blott,
1989](https://dl.acm.org/doi/10.1145/75277.75283)):

```haskell
class Show a where
  show :: a -> String

instance Show Bool where
  show True  = "True"
  show False = "False"

describe :: Show a => a -> String
describe x = "value: " ++ show x
```

Dictionary passing explains how a class constraint supplies behavior. The class becomes a record of
methods, an instance supplies that record, and `describe` receives it as a hidden argument. CGP
organizes choices through a context and resolves calls statically, without runtime dictionaries. The
[algebraic effects](./algebraic-effects.md) and [row polymorphism](./row-polymorphism.md) pages
compare other forms of passed evidence.

### Coherence and canonical instances

Coherence keeps implementation choices consistent as generic code is compiled and composed. A `Set`
, for example, relies on insertion and lookup agreeing on its element ordering. Global uniqueness of
instances is one way to support that agreement. GHC's guarantees within a compilation are distinct
from enforcing uniqueness across every separately compiled part of a program ([Yang, *Type classes:
confluence, coherence and global
uniqueness*](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)).

Rust enforces coherence through overlap and [orphan rules](/docs/reference/glossary#orphan-rule) .
An impl needs a local trait or a qualifying local type, subject to restrictions on uncovered type
parameters ([RFC 2451](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html)). GHC
permits orphan instances and can warn about them. CGP works within Rust's rules; the
[Rust proposals](./rust-language-proposals.md) page explores proposed alternatives.

### Overlapping instances

GHC permits some overlapping instances when one is more specific than another. Per-instance pragmas
control which overlaps are allowed:

```haskell
instance {-# OVERLAPPABLE #-} Show a => Show [a] where   -- lists in general
  show xs = "[" ++ intercalate "," (map show xs) ++ "]"

instance {-# OVERLAPPING #-} Show [Char] where           -- but strings specially
  show s = s
```

GHC normally requires a single most-specific candidate and may defer resolution if a type variable
could allow a competing match. Its manual warns that overlapping instances can still produce
inconsistent choices across a program ([GHC User's
Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)).

### Incoherent instances

`INCOHERENT` relaxes resolution further, allowing a choice even when later type information could
favor another instance. Different parts of a program can then use different dictionaries for the
same constraint. GHC's optimizer assumes coherence and may substitute dictionaries of the same type,
so such choices can lead to unexpected results ([GHC User's
Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)).

### Instance arguments in Agda

Agda provides class-style overloading without a class construct and without coherence. A record
plays the role of the class, and a double-braced argument is resolved from the scope at the call
site ([Devriese & Piessens, 2011](https://dl.acm.org/doi/10.1145/2034574.2034796)):

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

Lean allows multiple instances, so the same constraint can be satisfied in more than one way.
Multiple resolution paths can form a *diamond*. If they produce instances that are not
definitionally equal, code expecting those instances to coincide can fail ([Selsam, Ullrich & de
Moura, *Tabled Typeclass Resolution*](https://arxiv.org/pdf/2001.04301)). Mathlib therefore keeps
overlapping instances definitionally equal as a standing practice ([Baanen,
2022](https://arxiv.org/pdf/2202.01629)).

### Modular type classes

Dreyer, Harper, and Chakravarty showed that type classes can be expressed as ML modules with a
resolution layer added: **classes as signatures, and instances as structures and functors**. Certain
instance modules are designated *canonical* within a scope, so the compiler can resolve them
implicitly ([*Modular Type Classes*](https://people.mpi-sws.org/~dreyer/papers/mtc/main-long.pdf)).

The paper identifies a tension between **canonicity and modularity**. A canonical instance makes
implicit resolution predictable, but two modules can each supply a different instance. The designs
make different choices:

- **Haskell:** Implicit resolution with a preference for a canonical instance and extensions that
  relax the rules.
- **Modular type classes, OCaml's modular implicits, Agda instance arguments, and Scala implicits:**
  Implicit resolution with canonicity scoped or dropped, at the cost of possible ambiguity.
- **CGP:** Each context selects providers explicitly, and Rust resolves the resulting trait bounds.

The [ML modules](./ml-modules.md) page develops the module side.

## How CGP expresses it

CGP makes implementation choice explicit per context. A component has a consumer trait for callers
and a provider trait for implementations. The consumer trait is an ordinary Rust trait; providers
and wiring make several implementations selectable.

### Components define interfaces; providers name implementations {#a-component-is-a-class-a-provider-is-a-first-class-instance}

A component declares the interface callers use and generates a provider trait for implementations:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
```

`CanCalculateArea` is the class interface, as `class Show a` is. A Haskell instance is anonymous,
and the compiler chooses it. A CGP provider is a named marker type with a provider-trait impl, so a
context can select it explicitly:

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

Separate provider types let alternative implementations coexist under Rust's coherence rules. The
wiring selects `RectangleArea` for `Rectangle` ; another provider could implement the same provider
trait on its own marker type.

This example still makes one choice for `Rectangle` . It is a
[value context](/docs/reference/glossary#value-context) : the measured type also carries the wiring,
and the component is [self-targeted](/docs/reference/glossary#self-targeted-component) . Independent
choices for the same value type require moving selection to separate contexts, as the next example
does.

### Providers can accept overlapping sets of types {#overlapping-providers-need-no-specificity-rule}

CGP gives each encoder its own provider type, so their accepted value types can overlap. These
providers illustrate distinct choices for display output, raw bytes, and hexadecimal output:

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

These providers accept overlapping inputs: `String` supports display and byte access, while
`Vec<u8>` supports both byte encoders. A Haskell-style instance system would need a way to
distinguish those choices; constraints alone do not make one instance more specific. CGP keeps the
choices separate through provider types and records the selection in wiring.

The encoded value is the `Value` parameter rather than `Self` , so the contexts that wire these
providers are **[environmental contexts](/docs/reference/glossary#environmental-context)**, types
that stand for an application. The component is
[parameter-targeted](/docs/reference/glossary#parameter-targeted-component) .

### Selecting a provider per context and value type {#incoherent-choices-made-explicit-and-local}

Separate application contexts can select different providers for the same value type:

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

`AppA` encodes `Vec<u8>` as hexadecimal, while `AppB` returns its bytes directly. Each choice is
fixed for the context, component, and value type. This makes the difference explicit without
relaxing Rust's coherence rules. The `open` statement and `@` paths are explained in
[`delegate_components!`](/docs/reference/macros/delegate_components) .

### Providers avoid orphan conflicts and newtypes

A downstream crate can define a provider for a foreign component and value type by owning the
provider's marker type. This satisfies the orphan rule without wrapping the value in a newtype.
Alternative behaviors get their own provider types, rather than wrappers such as Haskell's `Sum` and
`Product` . The [Bypassing coherence](/docs/concepts/coherence) page develops this pattern.

## What each approach costs

Canonical instances simplify calls but make alternative implementations harder to use. A newtype can
select another instance, though that workaround becomes awkward when the wrapped type appears inside
other types ([Yang,
2014](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)). The
orphan rule shapes module structure ([Queensland FP
Lab](https://qfpl.io/posts/orphans-and-fundeps/)). Overlapping instances are subtle, and incoherent
instances, by GHC's own account, "may cause unexpected results" ([GHC User's
Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)). Local instances
also require a way to preserve consistency when values cross scopes.

Systems that allow multiple instances must manage ambiguity or disagreement between resolution
paths. Mathlib addresses this by keeping overlapping instances definitionally equal ([Baanen,
2022](https://arxiv.org/pdf/2202.01629)). Some practitioners argue that the coherence bargain is the
wrong one and prefer explicit dictionaries ([Chiusano, *The trouble with
typeclasses*](https://pchiusano.github.io/2018-02-13/typeclasses.html)). A 2025 survey compares
where Swift, Rust, Scala, and Haskell each draw the line ([Racordon, Flesselle & Pham,
2025](https://arxiv.org/pdf/2502.20546)).

CGP requires explicit wiring to choose providers. Providers also need a component declared with
[`#[cgp_component]`](/docs/reference/macros/cgp_component), adding code beyond a plain trait. Trait
resolution adds compile-time work and can produce long errors over generated types.
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page compares these costs with simpler
approaches.

## Where coherent type classes are the better choice

Coherent type classes fit programs that need one canonical instance per type, such as the `Ord` used
by a `Set` . The compiler keeps generic code consistent without wiring. In Rust, an ordinary trait
already provides that guarantee. The [Modularity Hierarchy](/docs/concepts/modularity-hierarchy)
page starts from this simpler case.

CGP fits programs that need several interchangeable implementations or a different choice per
context. Providers can also cover types the program does not own without orphan-rule conflicts.

## What to expect that differs

CGP changes where implementation choices are recorded:

- **Wiring selects providers.** Rust still resolves trait bounds, but a context names the provider
  instead of relying on a globally canonical implementation for the value type.
- **Consistency is scoped to the context and key.** Two contexts can encode the same type
  differently. Code that needs shared behavior must use the same selection.
- **Context placement controls flexibility.** Wiring on `Rectangle` gives that type one choice.
  Wiring on `AppA` and `AppB`, with the value passed separately, permits independent choices. The
  [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) explains these arrangements.
- **Explicit checks validate dependencies early.** A delegation entry alone does not prove that
  its provider can be used. [`check_components!`](/docs/reference/macros/check_components) asserts
  the required components and parameters beside the wiring; ordinary calls also force checking.

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
