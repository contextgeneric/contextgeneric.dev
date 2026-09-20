---
sidebar_label: 'Row polymorphism'
sidebar_position: 7
description: "CGP's extensible records and variants read against PureScript rows, OCaml polymorphic variants, structural typing, and row theory."
---

# Row polymorphism, structural typing, and extensible data types

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who knows
row polymorphism, structural typing, or extensible data types: PureScript's rows, OCaml's polymorphic
variants, TypeScript's structural checking, or the row-theory literature. CGP brings field-and-variant
driven extensibility to Rust's nominal structs and enums, not through a built-in row kind but by
deriving a type-level view of a type's shape and matching names through the trait system. The page
covers the correspondence, the one place the analogy breaks, where a real row system is the better
tool, and what to expect that differs.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. On this page every context is a **[value context](/docs/reference/glossary#value-context)**: the record being built or the enum being
cast is the type that carries the shape and the wiring, and each component targets `Self`.

| In a row system | In CGP |
| --- | --- |
| A closed row | A struct's derived `Fields`, a `Product!` of `Field<Tag, Value>` entries |
| A row variable, "the rest of the fields" | The context type variable, constrained by trait bounds and never named as a row |
| Row containment (`ρ₁ ≲ ρ₂`) | A `HasField<Symbol!("name")>` bound, usually written as an `#[implicit]` argument |
| Record concatenation (`ρ₁ ⊙ ρ₂ ~ ρ₃`) | `CanBuildFrom` and the `ConcatProduct` type-level operation |
| A presence flag per label | The `IsPresent` and `IsNothing` markers on a partial record |
| A row-typed sum | An enum's derived `Fields`, a `Sum!` over `Either` and `Void` |
| Variant injection and branching | `CanUpcast`, and the extensible visitor's dispatch |

## The idea, briefly

Nominal type systems, Rust's included, force code to name the concrete type it works on. A function
that reads a `name` field must take a specific struct, and two structs with identical fields but
different names are incompatible. The features on this page let a type's structure, not its name,
decide what operations apply.

### Structural versus nominal typing

A structural type system decides compatibility by a type's members rather than its name
([Wikipedia, *Structural type system*](https://en.wikipedia.org/wiki/Structural_type_system)).
TypeScript is the mainstream face of it: an object satisfies an interface by having the required
members ([TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes-oop.html)),
though TypeScript approximates rather than has row polymorphism
([Bornea](https://medium.com/@gabriel-bornea/approximating-row-type-polymorphism-in-typescript-495ebd5a623d)).
Rust sits at the nominal end: its structs and enums are nominal, and row polymorphism "is notably
absent from Rust's struct system"
([Andreas, *Structural vs Nominal Typing in Rust*](https://felixandreas.me/blog/nominal-vs-structural-types/)).
That absence is the ground CGP works on.

### Row polymorphism proper, in PureScript

Row polymorphism is structural flexibility delivered by a *row variable* rather than a subtype
relation: a function accepts any record that has at least the fields it names, and a type variable
carries the others through unchanged ([Wikipedia, *Row polymorphism*](https://en.wikipedia.org/wiki/Row_polymorphism)).
In PureScript a *row* is "an unordered collection of named types", of kind `Row k`, and a record is a
row wrapped by `Record` ([PureScript language reference](https://github.com/purescript/documentation/blob/master/language/Types.md)).
An open row adds a tail variable after a pipe:

```purescript
-- accepts any record that has at least firstName and lastName;
-- `r` captures whatever other fields the caller's record carries
fullName :: forall r. { firstName :: String, lastName :: String | r } -> String
fullName p = p.firstName <> " " <> p.lastName

fullName { firstName: "Ada", lastName: "Lovelace" }              -- ok
fullName { firstName: "Ada", lastName: "Lovelace", age: 36 }     -- also ok; age flows through `r`
```

The `Prim.Row` type classes `Cons`, `Union`, `Nub`, and `Lacks` govern the row operations, resolved
during type checking ([Pursuit, *Prim.Row*](https://pursuit.purescript.org/builtins/docs/Prim.Row)).
PureScript's design descends from Wand's row variables (1987), Rémy's principal types for rows
(1989), and Leijen's scoped labels (2005), which PureScript adopts directly
([Leijen, *Extensible records with scoped labels*](https://www.microsoft.com/en-us/research/publication/extensible-records-with-scoped-labels/)).

### Rows as a kind versus rows as predicates

Two designs realize row polymorphism. PureScript makes rows a *built-in kind* with its own
unification. Gaster and Jones instead encode rows through *qualified types*: a function is polymorphic
over an ordinary type variable, and the *lacks* and *has* predicates it must satisfy are discharged by
the same machinery that resolves type-class instances, with no new kind
([Gaster & Jones](http://web.cecs.pdx.edu/~mpj/pubs/polyrec.html)). CGP most closely resembles this
second design. CGP has no row kind and no row variable, but it does have row *predicates*: a
`HasField<Symbol!("name"), Value = String>` bound is a has-predicate over a context's row, resolved
by the trait solver. Rémy's *presence polymorphism* adds a per-label flag recording presence or
absence, which CGP reproduces with the markers shown below.

### Extensible variants: the sum-side dual

Where a row-polymorphic record function accepts "at least these fields", a row-polymorphic variant
value promises "at most these cases". OCaml's *polymorphic variants* are the industrial home of the
dual, needing no central type declaration; OCaml 5.5 infers the type in the comment:

```ocaml
(* no type declaration needed; the variant type is inferred and reusable *)
let to_int = function
  | `On       -> 1
  | `Off      -> 0
  | `Number n -> n
(* inferred: [< `Number of int | `Off | `On ] -> int *)
```

Extensible records and variants are the classic answer to Wadler's *expression problem*: adding both
new cases to a datatype and new operations over it "without recompiling existing code, and while
retaining static type safety" ([Wadler, 1998](https://homepages.inf.ed.ac.uk/wadler/papers/expression/expression.txt)).
Haskell's *Data types à la carte* is the constraint-based route for sums: a coproduct of signature
functors with a `:<:` constraint whose evidence supplies an injection `inj` and a partial projection
`prj` ([Swierstra](https://www.cs.tufts.edu/~nr/cs257/archive/wouter-swierstra/DataTypesALaCarte.pdf)).

### The unifying theory

Morris and McKinna's *rows by any other name* generalizes the designs into *row theories*, "a monoidal
generalization of row types", with two predicates: **combination** `ρ₁ ⊙ ρ₂ ~ ρ₃` and **containment**
`ρ₁ ≲ ρ₂`. Records are built by combination and read by containment; variants are built by
containment and consumed by combination. Its deepest idea for the CGP comparison is that "evidence for
type qualifiers has computational content": the proof that a row contains a label *is* the projection
code ([Morris & McKinna, POPL 2019](https://dl.acm.org/doi/10.1145/3290325)). Koka applies the same
rows to effects ([Leijen, *Koka*](https://arxiv.org/pdf/1406.2061)), which the
[algebraic effects](./algebraic-effects.md) page covers.

## How CGP expresses it

CGP keeps Rust's nominal structs and enums and *derives* a type-level description of a type's shape,
then matches field and variant names through the trait system. The row predicates become trait
constraints the trait solver resolves, in the Gaster and Jones style, carried by Rust's traits.

### A struct is a closed row; `HasField` is row containment

`#[derive(HasFields)]` gives a struct the type-level equivalent of a closed row, one
`Field<Tag, Value>` entry per field, tagged by a type-level string:

```rust
#[derive(HasField, HasFields)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

// generated:
// type Fields = Product![
//     Field<Symbol!("first_name"), String>,
//     Field<Symbol!("last_name"), String>,
// ];
```

Where PureScript's `fullName` names its required fields with an open row, a CGP function names them
as [implicit arguments](/docs/concepts/implicit-arguments), and any context whose derived row includes
the fields satisfies it:

```rust
#[cgp_fn]
pub fn full_name(&self, #[implicit] first_name: &str, #[implicit] last_name: &str) -> String {
    format!("{first_name} {last_name}")
}

#[derive(HasField)]
pub struct Employee {
    pub first_name: String,
    pub last_name: String,
    pub badge: u32,
}

person.full_name();     // "Ada Lovelace"
employee.full_name();   // "Ada Lovelace"; `badge` plays the part of `r`
```

The bound each argument desugars to, `HasField<Symbol!("first_name"), Value = String>`, is the
containment predicate, and the context's other fields play the role of the tail variable. The
difference from PureScript is that the row variable is implicit and never named: CGP does not infer or
carry a residual row, it resolves the containment bound against whatever concrete context is used.

### Building a record is row combination

Assembling a struct field by field is CGP's record concatenation. The
[builder family](/docs/reference/traits/builder/has_builder) walks a *partial record* from empty to
complete, and [`CanBuildFrom`](/docs/reference/traits/casting/can_build_from) absorbs the shared
fields of one struct into another's builder in one step, the analogue of PureScript's `Record.union`:

```rust
let combined: FooBarBaz = FooBarBaz::builder()
    .build_from(FooBar { foo: 1, bar: "bar".into() })  // splice one row in
    .build_from(Baz { baz: true })                     // splice another
    .finalize_build();                                 // exists only when the row is complete
```

The underlying type-level operations are a row algebra: [`ConcatProduct`](/docs/reference/traits/type-level/concat_product)
splices two products, `AppendProduct` adds one field, and `MapFields` rewrites every entry, all
evaluated during type checking. That `finalize_build` type-checks only when every field is present is
the completeness guarantee a closed row gives, recovered for generic Rust. The
[Extensible records](/docs/concepts/extensible-records) page develops the pattern.

### Presence markers are presence polymorphism

CGP tracks field presence with the per-field flags Rémy's presence polymorphism introduced, as
[`MapType`](/docs/reference/traits/type-level/map_type) markers on a partial record: `IsPresent`
stores the value, `IsNothing` stores nothing, and the
[optional-field extensions](/docs/reference/traits/optional/) add `IsOptional`. A builder starts at
all-`IsNothing`, each step flips one marker, and `FinalizeBuild` exists only at the all-`IsPresent`
configuration. The variant side uses `IsVoid` to mark a case ruled out during extraction.

### An enum is a row-typed sum; upcast and downcast are injection and branching

The variant side is the exact dual. The derive represents an enum as a `Sum!` of `Field` entries, and
the [structural casts](/docs/reference/traits/casting/can_upcast) implement the row-theory operations
directly. `CanUpcast` lifts a narrow enum into a wider one, which is variant *injection* and always
succeeds. `CanDowncast` narrows the other way, guarded by containment:

```rust
#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum Reading {
    Temperature(u64),
    Label(String),
}

#[derive(Debug, Eq, PartialEq, CgpData)]
pub enum ExtendedReading {
    Temperature(u64),
    Label(String),
    Flag(bool),
}

// upcast: variant injection, always succeeds
let wide = Reading::Temperature(21).upcast(PhantomData::<ExtendedReading>);

// downcast: succeeds only for variants the target still has
ExtendedReading::Flag(true).downcast(PhantomData::<Reading>).is_err();   // true
```

Branching, the combination predicate on the sum side, is CGP's
[extensible visitor](/docs/concepts/extensible-variants). The `MatchWithValueHandlers` dispatcher
derives one extract-and-handle step per variant from the enum's row and runs them as a pipeline, and
because each failed extraction rules out a variant at the type level, the final match is provably
exhaustive with no wildcard arm. In *Data types à la carte* terms, `CanUpcast` is `inj` and
`CanDowncast` is `prj`, but where à la carte's `prj` returns a `Maybe` with no exhaustiveness check,
CGP's branching consumes the whole sum and proves it covered every case.

### Providers are the theory's computational evidence

CGP lines up with the theory on one point more than with any implementing language: Morris and
McKinna's insight that a row predicate's evidence has computational content. In CGP that evidence is
visible. The `HasField` impl a derive generates *is* the projection code, the `CanBuildFrom` recursion
*is* the concatenation code, and a wired provider *is* the witness that a context can perform an
operation. Where a row-typed language elaborates the evidence invisibly, CGP surfaces it as ordinary
traits and impls a programmer can read.

## What each approach costs

Row polymorphism is valued because one function serves many record shapes, which "can greatly reduce
refactoring" ([Fowler, *Row Polymorphism without the Jargon*](https://jadon.io/blog/row-polymorphism/)),
and because rows drive codec-free JSON decoding and type-safe bindings
([Nguyen](https://hgiasac.github.io/posts/2018-11-18-Record-Row-Type-and-Row-Polymorphism.html)). Its
costs, as its users state them, cluster in three places. The loudest is error messages: row
unification produces large diagnostics, and one PureScript thread records outputs of around 152 KB
([PureScript Discourse](https://discourse.purescript.org/t/upcoming-changes-to-error-messages-for-large-records-rows/3696)).
The second is complexity: extensible records are widely seen as a specialist tool, and one user who
modeled a domain with them wished afterwards for separate functions per nominal type
([PureScript Discourse](https://discourse.purescript.org/t/when-to-use-extensible-types-when-modeling-a-domain/217)).
The third is specific to pure structural typing: a structurally valid call can be semantically wrong,
"like finding the `area` of a `Fish`" ([Fowler](https://jadon.io/blog/row-polymorphism/)), and OCaml's
manual warns that polymorphic variants "result in a weaker type discipline" and recommends core
variants for simple programs ([OCaml manual](https://ocaml.org/manual/5.5/polyvariant.html)). Two
languages retreated from rows for the first reason: PureScript replaced its row-typed effect monad
because "getting the effect rows to line up was sometimes quite tricky"
([PureScript-Resources](https://purescript-resources.readthedocs.io/en/latest/eff-to-effect.html)),
and Gleam removed row-typed records to become "less structural and more nominal in style"
([Gleam v0.4](https://gleam.run/news/gleam-v0.4-released/)).

CGP inherits the first two costs in its own idiom. A mis-wired or incomplete structural operation
surfaces as a long, generated-type trait error, and CGP asks for derives, type-level tags, and wiring
where a row system would infer everything. Its mitigations are
[`check_components!`](/docs/reference/macros/check_components), which names the missing field or
variant at the wiring site, and [`cargo cgp check`](/docs/cargo-cgp/check): `cargo cgp check` leads
with the root cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet
reshape every class. The diagnostics remain heavier than a nominal `match`. The third cost CGP largely avoids: because a
provider still binds to a context it was wired for, structure grants access without erasing identity.
The [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where a row system is the better choice

Where a program wants terse anonymous records with full inference, and can pay the error-message cost,
a real row system such as PureScript's is the better tool, and reaching for CGP's machinery to emulate
it would be over-engineering. Where a program is already in nominal Rust and wants structural
field-and-variant access at a few chosen points, a builder assembled from independent parts, a
visitor over an open set of variants, a cast between sibling enums, CGP delivers row polymorphism's
payoff without adopting a new kind system, and pays the complexity only where it is used.

## What to expect that differs

**CGP does not infer rows.** A reader expects `{ name :: String | r }` to be inferred and rows to be
unified. CGP has no row variable and no row inference: a provider names its required fields as
bounds, and a context's shape comes from a derive. Because CGP does not unify open rows, it also does
not produce the row-unification messages that drove PureScript off effect rows and Gleam off row-typed
records; its verbose errors have a different shape and a different mitigation.

**Structure grants access, not identity.** A row-trained reader may expect any shape-compatible value
to work anywhere. In CGP a provider reads a context's fields structurally, but which providers a
context has is a nominal, wired decision, so the "area of a `Fish`" call is not automatically
expressible.

**Shapes are opt-in.** Only a type that derives `HasFields` or `CgpData` exposes its row. A foreign
type without the derive has no shape CGP can see.

**The predicates are traits.** A `HasField` bound is a has-predicate, `CanUpcast` and `CanDowncast`
are `inj` and `prj`, and the trait solver plays the part of qualified-type inference. A reader from
the Gaster and Jones tradition, or from `row-types` in Haskell, will find the mechanism they already
reach for.

## Where to go next

- [Extensible records](/docs/concepts/extensible-records): the builder pattern in full.
- [Extensible variants](/docs/concepts/extensible-variants): the visitor pattern and the expression
  problem.
- [Reflection](./reflection.md): the same type-level shapes read against runtime and compile-time
  reflection.
- [Algebraic effects](./algebraic-effects.md): rows applied to effects, and how CGP's dependencies
  compare.
- [`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields) and
  [`CanBuildFrom`](/docs/reference/traits/casting/can_build_from): the constructs this page names.

## Sources

The PureScript snippet was compiled with PureScript 0.15.15 against the prelude and the OCaml snippet
with OCaml 5.5.0. The CGP snippets were compiled against `cgp` `0.8.0-alpha`.

- [Wikipedia, *Structural type system*](https://en.wikipedia.org/wiki/Structural_type_system), [*Nominal type system*](https://en.wikipedia.org/wiki/Nominal_type_system), and [*Row polymorphism*](https://en.wikipedia.org/wiki/Row_polymorphism): the definitions and their contrast.
- [PureScript language reference, *Types*](https://github.com/purescript/documentation/blob/master/language/Types.md), [Pursuit, *Prim*](https://pursuit.purescript.org/builtins/docs/Prim), and [*Prim.Row*](https://pursuit.purescript.org/builtins/docs/Prim.Row): rows, records, open-row syntax, and the row type classes.
- [Wand (1987)](https://www.ccs.neu.edu/home/wand/papers/wand-lics-87.pdf), [Rémy (1989)](https://dl.acm.org/doi/10.1145/75277.75284), and [Leijen, *Extensible records with scoped labels* (2005)](https://www.microsoft.com/en-us/research/publication/extensible-records-with-scoped-labels/): the row-polymorphism lineage and presence flags.
- [Gaster & Jones, *A Polymorphic Type System for Extensible Records and Variants* (1996)](http://web.cecs.pdx.edu/~mpj/pubs/polyrec.html): rows as qualified-type predicates, the design CGP's bounds resemble.
- [Swierstra, *Data types à la carte* (JFP 2008)](https://www.cs.tufts.edu/~nr/cs257/archive/wouter-swierstra/DataTypesALaCarte.pdf) and [Wadler, *The Expression Problem* (1998)](https://homepages.inf.ed.ac.uk/wadler/papers/expression/expression.txt): the constraint-based sum solution and the problem it answers.
- [Morris & McKinna, *Abstracting extensible data types: or, rows by any other name* (POPL 2019)](https://dl.acm.org/doi/10.1145/3290325): row theories, the combination and containment predicates, and evidence with computational content.
- [Leijen, *Koka: Programming with Row-Polymorphic Effect Types* (2014)](https://arxiv.org/pdf/1406.2061): rows applied to effects.
- [OCaml manual, *Polymorphic variants* (5.5)](https://ocaml.org/manual/5.5/polyvariant.html): open and closed variant bounds and the documented weaker discipline.
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes-oop.html), [Bornea](https://medium.com/@gabriel-bornea/approximating-row-type-polymorphism-in-typescript-495ebd5a623d), and [Andreas, *Structural vs Nominal Typing in Rust*](https://felixandreas.me/blog/nominal-vs-structural-types/): the wider structural landscape and Rust's nominal baseline.
- [PureScript-Resources, *Eff to Effect*](https://purescript-resources.readthedocs.io/en/latest/eff-to-effect.html) and [Gleam v0.4 release notes](https://gleam.run/news/gleam-v0.4-released/): two retreats from rows and their reasons.
- [Fowler, *Row Polymorphism without the Jargon*](https://jadon.io/blog/row-polymorphism/), [Nguyen, *Record Row Type and Row Polymorphism*](https://hgiasac.github.io/posts/2018-11-18-Record-Row-Type-and-Row-Polymorphism.html), and PureScript Discourse threads on [error messages](https://discourse.purescript.org/t/upcoming-changes-to-error-messages-for-large-records-rows/3696) and [when to use extensible types](https://discourse.purescript.org/t/when-to-use-extensible-types-when-modeling-a-domain/217): community sentiment.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
