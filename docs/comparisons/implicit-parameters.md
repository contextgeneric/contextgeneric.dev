---
sidebar_label: 'Implicit parameters'
sidebar_position: 5
description: 'Compare CGP context fields and wiring with Scala contextual parameters, Haskell implicit parameters, and type-class selection.'
---

# Implicit parameters

CGP supplies implicit arguments from a context's fields and selects implementations through wiring.
It is a language extension for Rust, with pluggable trait implementations at compile-time,
implemented as a library on stable Rust whose consumer traits are ordinary Rust traits; the
[Introduction](/docs/) explains the basics. For readers familiar with Scala's `given` and `using`,
Haskell's `ImplicitParams`, or `Reader`, this page compares how dependencies travel through a
program, how implementations are selected, and what each approach costs.

## In your terms

A **context** is the type a CGP method runs on, supplying data through fields and implementations
through wiring. It gives providers an explicit environment through `self`. This serves some of the
same purposes as contextual parameters or a `Reader` environment, with the available fields and
provider choices determined by the context's type.

| In Scala or Haskell | In CGP |
| --- | --- |
| A `using` parameter or a `?x` implicit parameter | An `#[implicit]` argument, read from a context field |
| An implicit environment value | A field of the context |
| A type-class interface | A consumer trait within a **component** |
| A type-class implementation | A **provider**: a named implementation |
| Instance selection | **Wiring**, written in a `delegate_components!` table |
| A functional dependency (`m -> e`) or associated type family | An [abstract type](/docs/reference/glossary#abstract-type) determined by the context |

## The idea, briefly

Implicit parameters let callers omit arguments that the compiler can supply from the surrounding
scope. Configuration, loggers, and comparison strategies can pass through a call chain without an
explicit argument at every call. Functions still declare the dependencies in their types, and the
language's resolution rules determine which values reach them.

### Context parameters in Scala

Scala 3 marks a contextual parameter list with `using`. When a caller omits that list, the compiler
searches for matching `given` values:

```scala
case class Config(port: Int, baseUrl: String)

def renderWebsite(path: String)(using config: Config): String =
    "<html>" + renderWidget(List("cart")) + "</html>"    // config passed implicitly

def renderWidget(items: List[String])(using config: Config): String = ???

given Config = Config(8080, "docs.scala-lang.org")

renderWebsite("/home")     // the given Config is supplied automatically
```

Scala uses contextual parameters for type classes as well as configuration values. A
`given Comparator[Int]` supplies the implementation required by a `using Comparator[A]` parameter.
Scala 3.6 introduced the colon form for a given with a body; the older
`given Comparator[Int] with` form is also accepted. See the
[Scala reference](https://docs.scala-lang.org/scala3/reference/contextual/givens.html).

```scala
trait Comparator[A]:
  def compare(x: A, y: A): Int

given Comparator[Int]:
  def compare(x: Int, y: Int): Int = x - y

def max[A](x: A, y: A)(using c: Comparator[A]): A =
  if c.compare(x, y) > 0 then x else y

max(1, 2)     // Comparator[Int] resolved and passed implicitly
```

Scala's selection depends on scope as well as type. Different scopes can supply different givens
for the same requested type, and a caller can pass a contextual argument explicitly. CGP's
per-context choice therefore differs in how it is recorded, rather than being a choice Scala cannot
express. Scala 2 uses `implicit` where Scala 3 distinguishes `given` definitions from `using`
parameters. Rust's proposed equivalent is discussed in [Rust's own proposals](./rust-language-proposals.md).

### Implicit parameters in Haskell

Haskell's `ImplicitParams` extension binds named parameters such as `?cmp` through constraints.
The following example passes a comparison function to `sort` and `least`, then supplies it in a
`let` binding:

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

The `?cmp` constraint propagates to callers that do not supply a binding. The extension does not
provide default bindings, and implicit parameters cannot appear in class or instance declaration
contexts. Type inference and the monomorphism restriction also affect which bindings remain
polymorphic. The [GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/implicit_parameters.html)
explains these restrictions. `Reader` and type classes offer other ways to pass dependencies in
Haskell, with different rules for binding and propagation.

### Type classes as implicit dictionary passing

Type classes pass implementations as implicit dictionaries of methods. A class declares an
interface, an instance implements it for a type, and a constrained function receives the selected
dictionary. This explains how generic code can invoke an operation without naming its implementation.

Haskell and Scala differ in how they select those dictionaries. Haskell ordinarily uses a global
instance for a class and its type arguments, with overlapping instances controlled by extensions.
Scala selects contextual values using scope and priority rules. [Coherence](/docs/reference/glossary#coherence) concerns whether valid
resolutions agree in meaning; global instance uniqueness is one way to support it, rather than a
rule shared unchanged by both languages. The [type classes](./type-classes.md) comparison develops
these distinctions.

## How CGP expresses it

CGP makes a context available to generic implementations through `self`. An `#[implicit]` argument
reads a value from that context, while component wiring selects an implementation. Both choices are
checked through Rust traits; neither requires searching the caller's lexical scope for a value.

### Implicit arguments read named context fields

An `#[implicit]` parameter names a field that the implementation reads from its context:

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

`rectangle_area()` obtains `width` and `height` from `self`, so callers omit both arguments.
`Rectangle` is a [value context](/docs/reference/glossary#value-context): the rectangle itself holds the data the method operates on.
The generated implementation requires field access with the matching names and types. Scala's
`using Config` instead asks the compiler to resolve a contextual value by type; Haskell's `?cmp`
refers to a named implicit binding. The
[Implicit arguments](/docs/concepts/implicit-arguments) page explains CGP's field access and borrowing.

### A shared context carries a common environment

Several providers can read the same runtime value from a shared context. For example, a context
can store configuration once and expose it to each implementation that declares the corresponding
field dependency. This serves the environment-passing role of `Reader` without requiring each
intermediate call to forward individual values.

A context can also determine types shared by its providers. Fallible implementations can import
`HasErrorType.Error` and use the alias `Error`, agreeing on the type selected for that context.
Runtime fields and associated types serve different roles, but both keep related choices attached
to one context.

### Abstract types are determined by the context

A CGP [abstract type](/docs/concepts/abstract-types) is an associated type supplied by the context.
Haskell's functional dependencies express a related relationship: `m -> e` in `MonadError` says
that the monad determines its error type. These declarations illustrate the two forms:

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

Both forms avoid making the error type an independent choice at every use. Haskell determines `e`
from `m`; Rust refers to the associated `Error` of the context. A CGP consumer trait can require
`HasErrorType` as a [supertrait](/docs/reference/glossary#supertrait), making that requirement available to generic callers through their
consumer-trait bound. The type dependency still exists; its relationship to the context makes it
unnecessary to carry a separate error-type parameter.

Different contexts can select different error types, just as different Haskell monads can determine
different error types. CGP additionally lets a context choose the provider for its abstract-type
component through the same wiring mechanism it uses for operations. See
[`mtl`'s `MonadError` documentation](https://hackage.haskell.org/package/mtl/docs/Control-Monad-Error-Class.html)
for the functional-dependency form.

### Components and wiring make instance selection explicit

CGP gives each interchangeable implementation its own provider type. Rust therefore treats these
as distinct impls, even when both providers support the same target. The following fragments define
two encoders and select one for each application:

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

`AppA` encodes `Vec<u8>` as hexadecimal text; `AppB` copies its bytes. Both are environmental
contexts representing applications. The component is [parameter-targeted](/docs/reference/glossary#parameter-targeted-component): `Value` is the data being
encoded, while `self` supplies the application's choice of implementation.

CGP records the provider choice in a wiring table and keeps Rust's coherence checks. Separate
provider types let alternatives coexist; they do not permit conflicting entries for the same key
on one context. Scala can express alternative implementations through scoped givens, while CGP
attaches the selection to the context type.

## What each approach costs

Implicit resolution reduces argument passing but requires readers to trace the selected binding.
Scala's givens can come from local scope, imports, or implicit scope, so understanding a call may
require following the search rules. The
[Scala reference](https://docs.scala-lang.org/scala3/reference/contextual/using-clauses.html)
describes how arguments are supplied. Haskell's `ImplicitParams` makes dependencies visible as
constraints, which must propagate until a binding supplies them; its inference restrictions can
also affect behavior. See the
[GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/implicit_parameters.html).

Global type-class instances favor a canonical interpretation of a type. In Haskell, an alternative
ordering or rendering often needs a `newtype` to distinguish it from the existing instance.
That trade differs from Scala's scoped selection. The
[coherence analysis by Yang](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)
separates global uniqueness from coherence and confluence.

CGP requires component declarations and provider selection through wiring or declared defaults.
This adds code to maintain and trait-resolution work during compilation. Tracing a selection can
also require following delegation through several tables. Raw diagnostics expose generated traits
and types: [`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it
recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) weighs these costs against simpler forms.

## Where implicit resolution is the better choice

Implicit resolution fits code that benefits from the surrounding language's instance conventions
or local contextual bindings. Haskell type classes work well for a canonical ordering or rendering;
Scala's givens support contextual choices without a separate CGP-style wiring table. Within Rust,
an ordinary trait is sufficient when one implementation per type expresses the intended behavior.

CGP helps when Rust code needs separately reusable implementations and explicit choices per context.
Those benefits must justify its additional declarations. It does not replace the scope rules or
inference mechanisms of Scala and Haskell.

## What to expect that differs

CGP selects providers through type-level wiring, including any declared defaults and delegation.
It does not search a caller's scope for an implicit value. Rust still rejects overlapping impls;
CGP separates implementations by giving them distinct provider types.

A provider choice can be traced from a context's wiring. A direct entry names it immediately;
forwarding entries and namespaces require following the route. Explicit wiring makes the route
inspectable without making every route short.

An abstract type is determined by the context type. Two contexts may choose different error types,
but two values of the same concrete context type share the same associated error type.

An implicit argument selects a field by name and checks its type. Two fields of the same type can
supply distinct arguments, such as `width` and `height`. Scala's contextual search instead resolves
a requested type using its scope and priority rules.

## Where to go next

These pages develop the mechanisms and related comparisons:

- [Implicit arguments](/docs/concepts/implicit-arguments): field selection and borrowing rules.
- [Abstract types](/docs/concepts/abstract-types): types determined by a context.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): keeping implementation
  requirements out of a consumer interface.
- [Type classes](./type-classes.md): dictionary passing and instance selection.
- [Rust's own proposals](./rust-language-proposals.md): the contexts-and-capabilities proposal.

## Sources

The Haskell snippets were compiled with GHC 9.10 and the Scala snippets with Scala 3.8.4. The CGP
snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!` assertion per wired
context.

- [Scala 3 Book, *Context Parameters*](https://docs.scala-lang.org/scala3/book/ca-context-parameters.html) and [Scala 3 Reference, *Given Instances*](https://docs.scala-lang.org/scala3/reference/contextual/givens.html): `using` clauses, `given` instances, and the 3.6 syntax change.
- [Baeldung, *Scala 3 Implicit Redesign*](https://www.baeldung.com/scala/scala-3-implicit-redesign): the rename from `implicit` to `given`/`using` and the reasons for it.
- [GHC User's Guide, *Implicit Parameters*](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/implicit_parameters.html): the `?x` syntax, `let` binding, propagation, and the documented restrictions.
- [Type class (Wikipedia)](https://en.wikipedia.org/wiki/Type_class) and [okmij.org, *Implementing, and Understanding Type Classes*](https://okmij.org/ftp/Computation/typeclass.html): type classes as dictionary-passing elaboration.
- [Yang, *Type classes: confluence, coherence and global uniqueness*](https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/) and [Bottu et al., *Coherence of Type Class Resolution*](https://xnning.github.io/papers/coherence-class.pdf): the distinctions between coherence, confluence, and global instance uniqueness.
- [`mtl`, `Control.Monad.Error.Class`](https://hackage.haskell.org/package/mtl/docs/Control-Monad-Error-Class.html): the `MonadError e m | m -> e` functional dependency.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
