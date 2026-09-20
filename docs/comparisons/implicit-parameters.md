---
sidebar_label: 'Implicit parameters'
sidebar_position: 5
description: "CGP read against Scala's given and using, Haskell's ImplicitParams, and the type-class resolution both build on."
---

# Implicit parameters

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who knows
implicit parameters: Scala's `given` and `using`, Haskell's `ImplicitParams`, and the type-class
resolution both languages build on. CGP shares the goal of threading context through code without
explicit plumbing, and supplies the values from a context's fields and wiring rather than from a
compiler-driven search. The page covers the correspondence, the coherence trade underneath it, where
implicit resolution remains the better tool, and what to expect that differs.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. It is the implicit environment you already reason about, the set of `given`s in scope or
the `Reader` you thread, made a single explicit type that every provider receives.

| In Scala or Haskell | In CGP |
| --- | --- |
| A `using` parameter or a `?x` implicit parameter | An `#[implicit]` argument, read from a context field |
| A `given` value in scope | A field of the context |
| A type class | A **component**: one trait with many possible implementations |
| An instance | A **provider**: a named implementation |
| Instance resolution by the compiler | **Wiring**, written in a `delegate_components!` table |
| A functional dependency (`m -> e`) or associated type family | An abstract type, chosen by the context |

## The idea, briefly

Some values are needed everywhere and interesting nowhere: a configuration, a logger, a comparison
strategy, an error type. Passing them through every function that transitively needs them clutters
signatures and forces intermediate functions to forward parameters they only pass along. Implicit
parameters remove that: the caller omits the argument, the compiler fills it in from scope, and the
declaration still records the dependency in the type.

### Context parameters in Scala

Scala 3 marks a parameter list `using`, and the compiler supplies a matching `given` from scope:

```scala
case class Config(port: Int, baseUrl: String)

def renderWebsite(path: String)(using config: Config): String =
    "<html>" + renderWidget(List("cart")) + "</html>"    // config passed implicitly

def renderWidget(items: List[String])(using config: Config): String = ???

given Config = Config(8080, "docs.scala-lang.org")

renderWebsite("/home")     // the given Config is supplied automatically
```

The same mechanism serves type classes. A `given` can be defined for a type, and a method with a
`using` parameter of that type resolves the instance from the call site. Scala 3.6 changed the syntax
for a given with a body to a colon form; the older `given Comparator[Int] with` is still accepted
([Scala 3 Reference, *Given Instances*](https://docs.scala-lang.org/scala3/reference/contextual/givens.html)):

```scala
trait Comparator[A]:
  def compare(x: A, y: A): Int

given Comparator[Int]:
  def compare(x: Int, y: Int): Int = x - y

def max[A](x: A, y: A)(using c: Comparator[A]): A =
  if c.compare(x, y) > 0 then x else y

max(1, 2)     // Comparator[Int] resolved and passed implicitly
```

Scala 2 wrote all of this with the single `implicit` keyword, and Scala 3 split it into `given` and
`using` because the one keyword was overloaded
([Baeldung, *Scala 3 Implicit Redesign*](https://www.baeldung.com/scala/scala-3-implicit-redesign)).
Rust's own contexts-and-capabilities proposal is the nearest thing to a `using Config` for Rust; the
[Rust proposals](./rust-language-proposals.md) page compares it.

### Implicit parameters in Haskell

Haskell's `ImplicitParams` extension is the more literal form. A function names a dynamically bound
variable `?x`, which appears as a constraint on its signature and is filled from the binding in
scope. This version uses the real `Data.List.sortBy` and compiles with GHC 9.10:

```haskell
{-# LANGUAGE ImplicitParams #-}
import Data.List (sortBy)

sort :: (?cmp :: a -> a -> Ordering) => [a] -> [a]
sort = sortBy ?cmp

least :: (?cmp :: a -> a -> Ordering) => [a] -> a
least xs = head (sort xs)

min' :: Ord a => [a] -> a
min' = let ?cmp = compare in least
```

The constraint propagates to any caller that does not bind it, and a `let` binding discharges it.
The GHC User's Guide records the restrictions: the constraints leak into every signature, there is no
way to declare a default, an implicit parameter may not appear in a class or instance context, and the
monomorphism restriction applies
([GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/implicit_parameters.html)).
Haskell programmers reach for the `Reader` monad or for type classes instead.

### Type classes as implicit dictionary passing

Type classes are the mechanism both languages use for implicit resolution in practice. A `class`
declares an interface, an `instance` provides it for a type, and a constrained call receives the
instance as a hidden *dictionary* argument. The resolution is type-directed and automatic, and it is
governed by *coherence*: for a given type there is exactly one instance, so the compiler can inject
it silently without two pieces of code disagreeing. Its price is the one CGP was built to escape. A
program cannot have two legitimate orderings of `Int` as first-class instances without a `newtype`,
and a module cannot add an instance for a type and class it does not own. The
[type classes](./type-classes.md) page develops the mechanism and its extensions.

## How CGP expresses it

CGP threads context through code and lets deep code read what it needs, through the context that is
already the `self` of every provider rather than through a scope search. Two constructs map onto the
two forms of implicit parameter: an `#[implicit]` argument corresponds to an implicit *value*
parameter, and a component with its wiring corresponds to a type class.

### Implicit arguments are implicit value parameters

An `#[implicit]` argument is written as an ordinary function parameter but is supplied from the
context's fields rather than by the caller:

```rust
#[cgp_fn]
pub fn rectangle_area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
    width * height
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}
```

Where Scala's `renderWebsite` obtains its `Config` from a `given`, this function obtains `width` and
`height` from the context threaded through it as `self`. The parameter disappears from the public
signature and is bound from the surroundings before the body runs, which is why CGP calls the feature
by the same name. The difference is where "the surroundings" live. A `using` parameter searches the
implicit scope by type; an `#[implicit]` argument reads the context field of the matching name, so
resolution is by *field*, decided when the context is defined. `Rectangle` here is a **value
context**: the type carrying `width` and `height` is the rectangle itself. The
[Implicit arguments](/docs/concepts/implicit-arguments) page develops the construct.

### A shared context value is a threaded environment

When several pieces of code must agree on one value, Scala uses a single `given Config` and Haskell
the `Reader` monad. CGP has them all read the same field or the same abstract type from the shared
context. CGP's error type is the standing instance: every fallible provider imports the context's
error type with `#[use_type(HasErrorType.Error)]` and names it as the bare `Error`, so all of them
agree on one type the context supplies once. That is a threaded `Reader` environment at the type
level, resolved at compile time.

### Abstract types are implicit type parameters

The correspondence runs one level up, and a reader from these languages will recognize this half
fastest. An [abstract type](/docs/concepts/abstract-types) is a type the context determines rather
than one a caller supplies. Haskell's `mtl` achieves the same with a functional dependency: in
`class MonadReader r m | m -> r` the environment type is fixed *by* the monad, and
`MonadError e m | m -> e` says the same for the error type
([`mtl`, `Control.Monad.Error.Class`](https://hackage.haskell.org/package/mtl/docs/Control-Monad-Error-Class.html)).
An associated type family states it more directly still, and is the closest thing in either language
to a `#[cgp_type]` component:

```haskell
class Monad m => MonadError e m | m -> e where
  throwError :: e -> m a
```

```rust
#[cgp_type]
pub trait HasErrorType {
    type Error: Debug;
}
```

Both exist for the same reason. A plain type *parameter* propagates: a signature that mentions `e`
and `r` carries them through every intermediate function that only passes a value along, as a Rust
generic function's `where` clause does. Making the type determined by the context, whether by a
functional dependency, an associated type, or a CGP abstract type, stops it propagating, because a
determined type is an output rather than an input. Two differences follow the same axes as the value
case. On *selection*, an `mtl` instance for a given monad is unique program-wide, whereas CGP's is a
wiring entry and two contexts may bind the same abstract type differently. On *propagation*, CGP
hides more: a CGP consumer trait declares the abstract-type trait as a supertrait, so a caller
bounding on the consumer gets it implied without restating it.

### Components and wiring are type classes without coherence

A CGP component is a type class, a provider is an instance, and wiring is the resolution step. Because
a provider's `Self` is its own marker type, many providers for the same component coexist, which
instances cannot do. Two encoders that both apply to `Vec<u8>` are selected per context:

```rust
#[cgp_component(Encoder)]
pub trait CanEncode<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}

#[cgp_impl(new EncodeAsHex)]
impl<Value: AsRef<[u8]>> Encoder<Value> {
    fn encode(&self, value: &Value) -> Vec<u8> {
        value.as_ref().iter().flat_map(|byte| format!("{byte:02x}").into_bytes()).collect()
    }
}

#[cgp_impl(new EncodeBytes)]
impl<Value: AsRef<[u8]>> Encoder<Value> {
    fn encode(&self, value: &Value) -> Vec<u8> {
        value.as_ref().to_vec()
    }
}

delegate_components! {
    AppA {
        open EncoderComponent;
        @EncoderComponent.Vec<u8>: EncodeAsHex,
    }
}

delegate_components! {
    AppB {
        open EncoderComponent;
        @EncoderComponent.Vec<u8>: EncodeBytes,
    }
}
```

The `Comparator[Int]` that Scala can define once becomes any number of interchangeable providers,
each selected per context. `AppA` and `AppB` are **environmental contexts**, types standing for an
application, and the component is parameter-targeted: `Self` is the application and the encoded value
is the `Value` parameter. The freedom has the cost the type-class trade predicts: CGP will not
*find* the provider for you by type. A context names its choice in a table, where Scala and
Haskell would resolve the instance from the type.

## What each approach costs

Implicit parameters and the type classes built on them are valued for erasing boilerplate: a
`Config` or a comparator threads through a deep call graph without appearing at every call, and one
generic function works over any type with an instance. Their costs, as their users state them, come
from the same generality. The recurring Scala complaint is that a value appears from nowhere, and tracking
down which `given` was selected, or why an expected one was not, takes time; Scala 2's single
overloaded keyword made it worse, and Scala 3 split the keyword apart to address it
([Baeldung](https://www.baeldung.com/scala/scala-3-implicit-redesign)). Haskell's `ImplicitParams`
is little used for the concrete reasons its manual records: leaking constraints, no defaults, and the
monomorphism restriction
([GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/implicit_parameters.html)).
And coherence itself is a persistent source of friction: the orphan rule shapes module structure, and
the one-instance-per-type limit forces `newtype` wrappers whenever a second interpretation of a type
is wanted ([Yang, 2014](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)).

CGP pays with the wiring itself. There is no automatic search, so the choice must be written down,
once per context, and a component must be declared before it can have providers. The compile-time
work is real. And the raw diagnostics are trait-solver output over generated types:
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where implicit resolution is the better choice

When a program wants one canonical instance per type, one `Ord`, one `Show`, one serialization, and
values automatic resolution above all, type classes are the better tool, and fighting their coherence
with CGP-style wiring would be over-engineering. The same holds in Rust: a trait with one
implementation per type is an ordinary trait. CGP's explicit wiring is the better tool when a program
needs several interchangeable implementations, per-deployment or per-context choice, or must
implement a behavior for types and traits it does not own.

## What to expect that differs

**CGP does not find the provider by type.** A reader used to `given` resolution will expect the
compiler to locate the implementation. In CGP the provider is named in a table, once per context.
Because CGP does not resolve by type, it is free of coherence, and that freedom lets it host the
overlapping instances these languages forbid.

**Which implementation was chosen is a line you can read.** The value still arrives without being
threaded by hand, but the choice is a wiring entry rather than the outcome of a scope search, so there
are no priority rules and no ambiguity to debug.

**An abstract type is chosen per context, not per type.** An `mtl` instance fixes the environment
type once for a monad program-wide. A CGP abstract type is a wiring entry, and two contexts may bind
it differently.

**Resolution is by field name, not by type.** An `#[implicit]` argument reads the context field with
the matching name. Two fields of the same type are distinct arguments, where a `using` parameter of
that type would be ambiguous.

## Where to go next

- [Implicit arguments](/docs/concepts/implicit-arguments): the construct this page maps onto a
  `using` parameter, with its borrowing rules.
- [Abstract types](/docs/concepts/abstract-types): why a determined type propagates nowhere while a
  parameter propagates everywhere.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): how a provider's requirements
  stay out of the caller's signature.
- [Type classes](./type-classes.md): the coherence trade in full.
- [Rust's own proposals](./rust-language-proposals.md): the contexts-and-capabilities proposal as
  Rust's `using Config`.

## Sources

The Haskell snippets were compiled with GHC 9.10 and the Scala snippets with Scala 3.8.4. The CGP
snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!` assertion per wired
context.

- [Scala 3 Book, *Context Parameters*](https://docs.scala-lang.org/scala3/book/ca-context-parameters.html) and [Scala 3 Reference, *Given Instances*](https://docs.scala-lang.org/scala3/reference/contextual/givens.html): `using` clauses, `given` instances, and the 3.6 syntax change.
- [Baeldung, *Scala 3 Implicit Redesign*](https://www.baeldung.com/scala/scala-3-implicit-redesign): the rename from `implicit` to `given`/`using` and the reasons for it.
- [GHC User's Guide, *Implicit Parameters*](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/implicit_parameters.html): the `?x` syntax, `let` binding, propagation, and the documented restrictions.
- [Type class (Wikipedia)](https://en.wikipedia.org/wiki/Type_class) and [okmij.org, *Implementing, and Understanding Type Classes*](https://okmij.org/ftp/Computation/typeclass.html): type classes as dictionary-passing elaboration.
- [Yang, *Type classes: confluence, coherence and global uniqueness*](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/) and [Bottu et al., *Coherence of Type Class Resolution*](https://xnning.github.io/papers/coherence-class.pdf): the definition of coherence and why it constrains instances to one per type.
- [`mtl`, `Control.Monad.Error.Class`](https://hackage.haskell.org/package/mtl/docs/Control-Monad-Error-Class.html): the `MonadError e m | m -> e` functional dependency.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
