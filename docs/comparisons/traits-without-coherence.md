---
sidebar_label: 'Without coherence'
sidebar_position: 1.5
description: 'How Cairo, Scala, OCaml modular implicits, Haskell, and an incoherent Rust choose among several impls of one trait, compared with Context-Generic Programming.'
---

# Traits without coherence

Some trait systems let one type have several implementations of a trait, and each then has to
decide which implementation a given use gets. [Context-Generic Programming](/docs/) is one answer:
CGP enables pluggable trait implementations in Rust, as a library on the stable toolchain whose
consumer traits are ordinary traits. Each context chooses among the implementations, while every impl
that Rust checks still obeys its coherence rules. This page compares that answer with Cairo's named
impls, Scala's `given` instances, OCaml's modular implicits, Haskell's
overlapping and incoherent instances, and Boxy's sketch of an incoherent Rust. It covers one
question only, how each system makes the choice, and links each system's own comparison for the
rest.

## In your terms

A **context** is the type a CGP method runs on. It chooses implementations through its wiring and can
supply values through its fields. Often it is a type defined to stand for one application.

Each system resolves the choice in its own place, and CGP has a construct that plays the same part:

| System | How it chooses an implementation | Where the choice lives | In CGP |
| --- | --- | --- | --- |
| Haskell | One instance per type, with opt-in overlap | The whole program | A provider, one of several, chosen per context |
| Cairo | Inferred from the named impls a call site imports | The caller's imports | A provider named in a `delegate_components!` table |
| Scala 3 | Ranked among the `given` instances in scope | The enclosing scopes | The context's wiring, with no ranking |
| Modular implicits | Found among the implicit modules in scope, ambiguity rejected | The enclosing scopes | The context's wiring |
| Boxy's sketch | Named at the call, or fixed on a type such as `HashMap` | The call, or the type | The **context**, which fixes every choice for the code that uses it |

## The idea

Once a type can have several implementations, three problems follow. Two call sites can resolve the
same type differently. Changing an import can change which implementation a call uses. And a
structure such as a hash table, which needs every operation on one value to agree on one hashing
implementation, loses that agreement unless something restores it. The systems below fall into three
families by how they answer. Haskell keeps one answer for the whole program and relaxes it only by
exception. Cairo, Scala, and modular implicits search what is in scope at the call. Boxy's sketch
names the choice at the call, or fixes it on a type.

### Haskell

Haskell resolves instances across the whole program and expects one instance per class and type, so
its departures from coherence are extensions that a programmer opts into for each instance. An
instance marked `OVERLAPPABLE` or `OVERLAPPING` may sit beside a more or less specific one, and
resolution commits only to a match that is strictly more specific than every other:

```haskell
{-# LANGUAGE FlexibleInstances #-}
import Prelude hiding (Show, show)
import Data.List (intercalate)

class Show a where
  show :: a -> String

instance {-# OVERLAPPABLE #-} Show a => Show [a] where   -- lists in general
  show xs = "[" ++ intercalate "," (map show xs) ++ "]"

instance {-# OVERLAPPING #-} Show [Char] where           -- but strings specially
  show s = s
```

An instance marked `INCOHERENT` goes further, because resolution may commit to it even where a more
specific instance could later apply. GHC's manual warns that overlap alone "can give rise to
incoherence (different instance choices are made in different parts of the program)", and that its
optimiser "assumes that type-classes are coherent"
([GHC User's Guide](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)). The
[type classes](./type-classes.md#overlapping-instances) page covers both extensions in full.

### Cairo

Cairo, the smart-contract language of Starknet, borrows Rust's syntax but gives every impl a name and
drops the requirement that an impl be unique. In the words of its design discussion, it has "no
orphan rules nor uniqueness demand" ([discussion #2744](https://github.com/starkware-libs/cairo/discussions/2744)). A
trait takes every type as an explicit parameter, an impl is written `impl Name of Trait<…>`, and a
generic impl takes the impls it depends on as parameters, named as `impl Inner: Describe<T>` or
anonymous as `+Drop<T>` ([Cairo Book](https://www.starknet.io/cairo-book/ch08-02-traits-in-cairo.html)).
Here two impls of one trait for `u8` coexist in separate modules:

```rust
// Cairo
trait Describe<T> {
    fn describe(self: @T) -> felt252;
}

mod decimal {
    pub impl U8Decimal of super::Describe<u8> {
        fn describe(self: @u8) -> felt252 { 10 }
    }
}

mod hex {
    pub impl U8Hex of super::Describe<u8> {
        fn describe(self: @u8) -> felt252 { 16 }
    }
}

impl ArrayDescribe<T, impl Inner: Describe<T>, +Drop<T>> of Describe<Array<T>> {
    fn describe(self: @Array<T>) -> felt252 {
        let mut total = 0;
        for x in self.span() {
            total += Inner::describe(x);
        };
        total
    }
}

mod uses_decimal {
    use super::Describe;
    use super::decimal::U8Decimal;
    pub fn run() -> felt252 {
        let xs: Array<u8> = array![1, 2];
        xs.describe()       // ArrayDescribe with U8Decimal: 20
    }
}

mod uses_hex {
    use super::Describe;
    use super::hex::U8Hex;
    pub fn run() -> felt252 {
        let x: u8 = 1;
        x.describe()        // U8Hex: 16
    }
}
```

The caller never passes an impl. Instead, the compiler infers each one from the impls the call site
imports, including the `Inner` impl that `ArrayDescribe` needs. That is why the two modules resolve
`Describe<u8>` differently: each imports a different impl. A call that can see both fails with
`E2313`, "Trait `Describe::<u8>` has multiple implementations", and one that can see neither fails
with "Trait has no implementation in context". A caller must also import each impl in the chain that
a generic impl needs, such as `U8Decimal` for `ArrayDescribe`. A glob import shortens that list, but
it brings in every impl in the module, so a module that holds both of them leads to the same
`E2313`.

### Scala

Scala 3 fills a `using` parameter from the `given` instances in scope:

```scala
trait Comparator[A]:
  def compare(x: A, y: A): Int

given Comparator[Int]:
  def compare(x: Int, y: Int): Int = x - y

def max[A](x: A, y: A)(using c: Comparator[A]): A =
  if c.compare(x, y) > 0 then x else y

max(1, 2)     // Comparator[Int] resolved and passed implicitly
```

Where several givens fit, Scala ranks them instead of requiring one. A given nested more deeply
wins, a case Scala 2 reported as ambiguous. An ambiguity found partway through the search is still an
error, and it is reported to the caller
([*Changes in Implicit Resolution*](https://docs.scala-lang.org/scala3/reference/changed-features/implicit-resolution.html)).
Scala 3 also keeps givens out of ordinary wildcard imports. `import A.*` brings in everything except
givens, and `import A.given` brings in the givens, so that "it is made clearer where givens in scope
are coming from" ([*Importing Givens*](https://docs.scala-lang.org/scala3/reference/contextual/given-imports.html)).
The [implicit parameters](./implicit-parameters.md#context-parameters-in-scala) page covers `given` and
`using` in full.

### OCaml modular implicits

The modular-implicits proposal lets an OCaml function take a module parameter that the compiler
fills from the modules declared `implicit` in scope. An implicit functor builds an instance for lists
from one for their elements:

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

Different scopes can hold different implicit modules for one type. Where two would fit, resolution
rejects the call rather than ranking them, and the programmer passes the module explicitly
([White, Bour & Yallop](https://arxiv.org/abs/1512.01895)). The snippet follows the proposal paper,
because the extension exists only as an
[experimental fork](https://github.com/ocamllabs/ocaml-modular-implicits) and does not run on released
OCaml. The [ML modules](./ml-modules.md#modular-implicits) page covers it with the module system it
extends.

### Boxy's sketch

Boxy's *An Incoherent Rust* drops Rust's coherence rules for traits declared `incoherent trait`,
names each impl, and passes the impl wherever a bound needs one:

```rust
impl Name<T> = Trait<T> for T { /* ... */ }

function::<T + TraitImpl<T>>(/* ... */)
```

The syntax is a sketch, not a proposal, and does not compile. Naming the impl at each call means an
import can never change a choice, though every call site has to state it. For the hash-table problem,
the post moves the bounds onto the type, as `struct HashMap<K: Hash + Eq, V> { ... }`, so that "which
impl is used is part of the HashMap type itself"
([Boxy, *An Incoherent Rust*](https://www.boxyuwu.blog/posts/an-incoherent-rust/)). The sketch
therefore uses two answers: naming at the call in general, and fixing the choice on a type wherever
every operation must agree. [Rust's own proposals](./rust-language-proposals.md#named-and-incoherent-impls)
places it among the other designs Rust has considered.

## How CGP expresses it

CGP belongs to the family that fixes the choice on a type, and the type it uses is the context. A
**provider** is a named impl, implemented for a marker type its own crate owns, so two providers of
one component for one type coexist and Rust's overlap check accepts both. The context's
`delegate_components!` table then picks one provider for each component and value type, and every use
through that context gets the same one. The examples wire an
**[environmental context](/docs/reference/glossary#environmental-context)**, `App`, for a
[parameter-targeted](/docs/reference/glossary#parameter-targeted-component) serializer component.

### Assembling the chain at the call

Cairo's `ArrayDescribe`, a generic impl that takes its element's impl as a parameter, can be written
almost directly in CGP. Here the same shape serializes a list: the element's provider becomes a type
parameter of a [higher-order provider](/docs/concepts/higher-order-providers), bound with
`#[use_provider]`, and the caller assembles the chain that Cairo would infer:

```rust
#[cgp_component(ValueSerializer)]
pub trait CanSerializeValue<T> {
    fn serialize(&self, value: &T, output: &mut Vec<u64>);
}

#[cgp_impl(new VecSerdeWith<Inner>)]
#[use_provider(Inner: ValueSerializer<T>)]
impl<T, Inner> ValueSerializer<Vec<T>> {
    fn serialize(&self, value: &Vec<T>, output: &mut Vec<u64>) {
        output.push(value.len() as u64);
        for item in value {
            Inner::serialize(self, item, output);
        }
    }
}

// The caller names every provider in the chain.
<VecSerdeWith<PairSerdeWith<U8Serde, BoolSerde>> as ValueSerializer<App, Vec<(u8, bool)>>>::serialize(
    &App, &value, &mut output,
);
```

The example omits the `PairSerdeWith`, `U8Serde`, and `BoolSerde` providers, which follow the same
pattern. Written this way, every caller repeats the assembly that Cairo's compiler infers from imports.

### Fixing the choice on the context

Idiomatic CGP moves the choice to the context. The provider asks the context for the element's
serializer through `#[uses]`, and the context's table answers once, for every caller:

```rust
#[cgp_impl(new VecSerde)]
#[uses(CanSerializeValue<T>)]
impl<T> ValueSerializer<Vec<T>> {
    fn serialize(&self, value: &Vec<T>, output: &mut Vec<u64>) {
        output.push(value.len() as u64);
        for item in value {
            self.serialize(item, output);
        }
    }
}

pub struct App;

delegate_components! {
    App {
        open ValueSerializerComponent;

        @ValueSerializerComponent.<T> Vec<T>: VecSerde,
        @ValueSerializerComponent.<A, B> (A, B): PairSerde,
        @ValueSerializerComponent.u8: U8Serde,
        @ValueSerializerComponent.bool: BoolSerde,
    }
}

check_components! {
    App {
        ValueSerializerComponent: Vec<(u8, bool)>,
    }
}
```

Code that serializes through `App` imports only the consumer trait, `CanSerializeValue`, and calls
`app.serialize(&value, &mut output)`. The `open` statement dispatches the component by value type,
and the table is the one place a reader looks to learn which provider each type gets. This answers
all three problems. Every call through `App` agrees, and no import can change a choice. The
agreement a hash table needs also follows from the context, much as it follows from the type in
Boxy's `HashMap`, but for every component the context wires rather than for one type. A second
context with a different provider for `u8` makes a different choice from the same providers.

CGP does not relax Rust's coherence to do this. Each provider implements the provider trait for its
own type, and each table entry is an ordinary impl on the context, so a second entry for the same
context and value type fails with `E0119`, like any overlapping impl.
[Bypassing coherence](/docs/concepts/coherence) explains the arrangement in full.

## What each approach costs

Each system's costs are the ones its own documentation and authors name:

- **Haskell:** the overlap extensions trade away determinism, and GHC's manual warns that they can
  make different parts of a program choose different instances.
- **Cairo:** a caller imports every impl in the chain, and imports decide the choice, so two call
  sites in one project can use different impls for one type.
- **Scala 3:** the ranking rules decide among givens in scope, and they changed between Scala 2 and
  Scala 3; the redesign also renamed Scala 2's overloaded `implicit` keyword to `given` and `using`.
- **Modular implicits:** the proposal has not shipped in OCaml, so it has no users to report on.
- **Boxy's sketch:** its post leaves syntax, ergonomics for deriving incoherent traits, migration of
  the ecosystem, and whether the complexity is worth it as open questions.

CGP's costs are its ordinary ones. The component and its providers need declarations, and the wiring
is code someone writes and reads. Wiring also adds compile-time work, and its raw errors are trait-solver
output over generated types. [`cargo cgp check`](/docs/tooling/cargo-cgp/check) leads with the root
cause for the classes it recognizes, and the tool does not yet reshape every class. CGP reaches only
the traits defined as components, so it cannot give `serde::Serialize` a second impl without a
parallel component. And it cannot nest choices: a block inside a program cannot override the
context's wiring for the code it encloses, as Scala's nesting rule allows. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against simpler
approaches.

## Where another system is the better choice

Each of the other systems fits a setting better than CGP does:

- **Haskell's global instances** give the same answer everywhere with no table, which suits code
  that needs one meaning per type, as Rust's own coherence does for ordinary Rust code.
- **Cairo's and Scala's inference** keeps a small program free of wiring when it needs only one
  choice per type and its imports are easy to follow.
- **A language feature,** such as an incoherent Rust, would reach existing traits like
  `serde::Serialize`, which a library cannot.

CGP fits a program that needs several valid implementations of a trait and wants each application
to make its choice once, in a table it can read, on stable Rust today.

## What to expect that differs

A reader coming from these systems may expect behavior that CGP does not have:

- **CGP does not search scope.** A context names every provider it uses, so importing a provider
  never changes a choice. The cost is that the choice must be written down.
- **CGP keeps Rust's coherence rules.** The provider traits allow several implementations, but every
  impl Rust checks is coherent, so the compiler never picks between two candidates on its own, as it
  may with Haskell's `INCOHERENT` instances.
- **Choices belong to types, not blocks.** A different choice needs another context type rather than
  an inner scope.
- **Generic code names only the consumer trait.** It never names a provider, so the same function
  works with any context that wires the component.

## Where to go next

These pages develop the ideas this comparison rests on:

- [Bypassing coherence](/docs/concepts/coherence): the coherence rules and how separate provider types
  work within them.
- [Higher-order providers](/docs/concepts/higher-order-providers): providers that take other providers
  as parameters, as the Cairo-style chain does.
- [Rust's own proposals](./rust-language-proposals.md): specialization, dictionary passing, named
  impls, and contexts and capabilities.
- [Type classes](./type-classes.md), [Implicit parameters](./implicit-parameters.md), and
  [ML modules](./ml-modules.md): the full comparisons with Haskell, Scala, and OCaml.

## Sources

The Cairo snippet was compiled and run with Cairo 2.21.0 (`cairo-run`), and the errors it quotes come
from that release. The Haskell snippet was compiled with GHC 9.10 and the Scala snippet with Scala
3.8.4, as on their own comparison pages. The modular-implicits snippet follows the proposal paper and
cannot run on released OCaml, and Boxy's syntax is a sketch that does not compile. The CGP snippets
were compiled against `cgp` `0.8.0` with a `check_components!` assertion on `App`.

- [Cairo Book, *Traits in Cairo*](https://www.starknet.io/cairo-book/ch08-02-traits-in-cairo.html) and [*Generic Data Types*](https://www.starknet.io/cairo-book/ch08-01-generic-data-types.html): the `impl Name of Trait` form, named and anonymous impl parameters, and inferred impls.
- [Cairo design discussion #2744](https://github.com/starkware-libs/cairo/discussions/2744): Cairo's lack of orphan rules and of a uniqueness requirement.
- [GHC User's Guide, *Instance declarations and resolution*](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html): the overlap and incoherence pragmas and the manual's warnings.
- [Scala 3 Reference, *Changes in Implicit Resolution*](https://docs.scala-lang.org/scala3/reference/changed-features/implicit-resolution.html) and [*Importing Givens*](https://docs.scala-lang.org/scala3/reference/contextual/given-imports.html): the nesting rule, propagated ambiguity, and given imports.
- [White, Bour & Yallop, *Modular implicits*](https://arxiv.org/abs/1512.01895) and the [experimental fork](https://github.com/ocamllabs/ocaml-modular-implicits): implicit module parameters, implicit functors, and rejected ambiguity.
- [Boxy, *An Incoherent Rust*](https://www.boxyuwu.blog/posts/an-incoherent-rust/): named impls, impls passed in bounds, and the `HashMap` bounds moved onto the type.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
