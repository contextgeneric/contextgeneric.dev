---
sidebar_label: 'Row polymorphism'
sidebar_position: 7
description: "How CGP expresses field access, record building, and variant operations compared with row polymorphism."
---

# Row polymorphism, structural typing, and extensible data types

CGP gives Rust structs and enums selected forms of structural reuse through derived field lists and
trait bounds. It is a language extension built as a [stable Rust library](/docs/) , with pluggable
implementations behind ordinary Rust traits. This page compares its field access, builders, and
variant operations with PureScript rows and OCaml polymorphic variants.

## In your terms

A **context** is the type a CGP method runs on. The examples here operate directly on records and
enums, so their contexts are [value contexts](/docs/reference/glossary#value-context) . Field access
uses derived traits; component-based operations can also use wiring to select providers.

The row operations map onto CGP constructs as follows:

| In a row system | In CGP |
| --- | --- |
| A closed row | A struct's derived `Fields`, a `Product!` of `Field<Tag, Value>` entries |
| An open row with required fields | A context type variable with field-access bounds; the remaining fields are not represented by a row variable |
| Row containment (`ρ₁ ≲ ρ₂`) | A `HasField<Symbol!("name")>` bound, usually written as an `#[implicit]` argument |
| Record concatenation (`ρ₁ ⊙ ρ₂ ~ ρ₃`) | `CanBuildFrom` and the `ConcatProduct` type-level operation |
| A presence flag per label | The `IsPresent` and `IsNothing` markers on a [partial record](/docs/reference/glossary#partial-record) |
| A row-typed sum | An enum's derived `Fields`, a `Sum!` over `Either` and `Void` |
| Variant injection and branching | `CanUpcast`, and the extensible visitor's dispatch |

## The idea, briefly

Rust's nominal types are distinct even when they have the same fields. Code that reads a `name`
field normally names a particular struct. Row polymorphism lets code depend on the field instead;
CGP offers selected forms of that flexibility within Rust's nominal type system.

### Structural versus nominal typing

Structural typing makes types compatible when they have the required members. For example,
TypeScript accepts an object as an interface when its members satisfy that interface ([TypeScript
Handbook](https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes-oop.html)). Row
polymorphism adds a way to name and preserve the remaining fields. Rust structs and enums have
nominal identities even when their fields match; CGP adds structural operations through traits.

### Row polymorphism in PureScript

Row polymorphism uses a *row variable* to describe the fields a function does not name. The function
accepts any record with its required fields, while the variable carries the remaining fields
([Wikipedia, *Row polymorphism*](https://en.wikipedia.org/wiki/Row_polymorphism)). In PureScript a
*row* is "an unordered collection of named types", of kind `Row k` , and a record is a row wrapped
by `Record` ([PureScript language
reference](https://github.com/purescript/documentation/blob/master/language/Types.md)). An open row
adds a tail variable after a pipe:

```purescript
-- accepts any record that has at least firstName and lastName;
-- `r` captures whatever other fields the caller's record carries
fullName :: forall r. { firstName :: String, lastName :: String | r } -> String
fullName p = p.firstName <> " " <> p.lastName

fullName { firstName: "Ada", lastName: "Lovelace" }              -- ok
fullName { firstName: "Ada", lastName: "Lovelace", age: 36 }     -- also ok; age flows through `r`
```

PureScript checks row operations through the `Prim.Row` classes `Cons` , `Union` , `Nub` , and
`Lacks` ([Pursuit, *Prim.Row*](https://pursuit.purescript.org/builtins/docs/Prim.Row)). PureScript's
design descends from Wand's row variables (1987), Rémy's principal types for rows (1989), and
Leijen's scoped labels (2005), which PureScript adopts directly ([Leijen, *Extensible records with
scoped
labels*](https://www.microsoft.com/en-us/research/publication/extensible-records-with-scoped-labels/)).

### Rows as a kind or as predicates

Row systems can represent field requirements through built-in rows or through predicates. PureScript
gives rows their own kind and unification rules. Gaster and Jones use *qualified types*: ordinary
type variables constrained by predicates such as *has* and *lacks*, checked through instance
resolution ([Gaster & Jones](http://web.cecs.pdx.edu/~mpj/pubs/polyrec.html)).

CGP resembles the predicate design. A `HasField<Symbol!("name"), Value = String>` bound says that a
context has a field, and Rust's trait solver checks it. CGP uses ordinary Rust types throughout,
without a separate row kind or row variable.

### Extensible variants

Variants are the dual of records. A row-polymorphic record function accepts "at least these fields",
while a row-polymorphic variant value promises "at most these cases". OCaml's *polymorphic variants*
provide these variants in a production language, without a central type declaration. OCaml 5.5
infers the type shown in the comment:

```ocaml
(* no type declaration needed; the variant type is inferred and reusable *)
let to_int = function
  | `On       -> 1
  | `Off      -> 0
  | `Number n -> n
(* inferred: [< `Number of int | `Off | `On ] -> int *)
```

[Extensible records](/docs/reference/glossary#extensible-record) and variants are a standard answer
to Wadler's *expression problem*: adding both new cases to a datatype and new operations over it
"without recompiling existing code, and while retaining static type safety" ([Wadler,
1998](https://homepages.inf.ed.ac.uk/wadler/papers/expression/expression.txt)). Haskell's *Data
types à la carte* solves the sum side with constraints. It builds a coproduct of signature functors
with a `:<:` constraint whose evidence supplies an injection `inj` and a partial projection `prj`
([Swierstra](https://www.cs.tufts.edu/~nr/cs257/archive/wouter-swierstra/DataTypesALaCarte.pdf)).

### Combination, containment, and their implementations {#the-unifying-theory}

Row theory connects records and variants through combination and containment. In Morris and
McKinna's *rows by any other name*, **combination** (`ρ₁ ⊙ ρ₂ ~ ρ₃`) describes joining rows, and
**containment** (`ρ₁ ≲ ρ₂`) describes one row fitting inside another. Records use combination to
build values and containment to read them; variants reverse those roles ([Morris & McKinna, POPL
2019](https://dl.acm.org/doi/10.1145/3290325)).

The evidence for these relationships can also implement the operation. Evidence that a record
contains a field supplies the code that reads it. This connection matters for CGP because its trait
implementations both satisfy constraints and perform field access or conversion. Rows also describe
effect sets, as in [Koka](https://arxiv.org/pdf/1406.2061) ; the
[algebraic effects](./algebraic-effects.md) page covers that use.

## How CGP expresses it

CGP derives type-level shapes for Rust's nominal structs and enums. Trait bounds then express the
field and variant requirements that row predicates express in a row system.

### Field lists and field requirements {#a-struct-is-a-closed-row-hasfield-is-row-containment}

`#[derive(HasFields)]` gives a struct the type-level equivalent of a closed row, with one
`Field<Tag, Value>` entry per field, each tagged by a
[type-level string](/docs/reference/glossary#type-level-string) :

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

PureScript's `fullName` names its required fields with an open row. A CGP function names them as
[implicit arguments](/docs/concepts/implicit-arguments) , and any context with those fields
satisfies it:

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

Each implicit argument desugars to a bound such as `HasField<Symbol!("first_name"), Value = String>`
, which is the containment predicate. The context's other fields play the role of the tail variable.
Unlike PureScript, CGP never names the row variable. It does not infer or carry a residual row; it
resolves each containment bound against the concrete context in use.

### Combining fields in a builder {#building-a-record-is-row-combination}

Assembling a struct field by field is CGP's form of record concatenation. The
[builder family](/docs/reference/traits/builder/has_builder) takes a *partial record* from empty to
complete. [`CanBuildFrom`](/docs/reference/traits/casting/can_build_from) adds the shared fields of
one struct to another's builder in one step, much as PureScript's `Record.union` joins two records:

```rust
let combined: FooBarBaz = FooBarBaz::builder()
    .build_from(FooBar { foo: 1, bar: "bar".into() })  // splice one row in
    .build_from(Baz { baz: true })                     // splice another
    .finalize_build();                                 // exists only when the row is complete
```

Type-level operations describe how fields combine.
[`ConcatProduct`](/docs/reference/traits/type-level/concat_product) joins two products,
`AppendProduct` adds one field, and `MapFields` rewrites every entry. `finalize_build` type-checks
only when every field is present, which gives generic Rust the completeness guarantee of a closed
row. The [Extensible records](/docs/concepts/extensible-records) page develops the pattern.

### Tracking which fields are present {#presence-markers-are-presence-polymorphism}

CGP tracks field presence with a flag per field, as Rémy's presence polymorphism does. The flags are
[`MapType`](/docs/reference/traits/type-level/map_type) markers on a partial record: `IsPresent`
stores the value, `IsNothing` stores nothing, and the
[optional-field extensions](/docs/reference/traits/optional/) add `IsOptional` . A builder starts
with every field marked `IsNothing` , and each step changes one marker. `FinalizeBuild` is
implemented only when every marker is `IsPresent` . On the variant side, `IsVoid` marks a case that
extraction has ruled out.

### Widening, narrowing, and handling variants {#an-enum-is-a-row-typed-sum-upcast-and-downcast-are-injection-and-branching}

The variant side mirrors the record side. The derive represents an enum as a `Sum!` of `Field`
entries, and the [structural casts](/docs/reference/traits/casting/can_upcast) implement the
row-theory operations. `CanUpcast` converts a narrower enum into a wider one. This is variant
*injection*, and it always succeeds. `CanDowncast` converts the other way, and it succeeds only for
variants the target contains:

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

CGP's [extensible visitor](/docs/concepts/extensible-variants) combines handlers to cover every
variant, corresponding to branching in row theory. The `MatchWithValueHandlers` dispatcher derives
one extract-and-handle step per variant from the enum's shape and runs the steps as a pipeline. Each
failed extraction rules out a variant at the type level, so the final match is provably exhaustive
without a wildcard arm. `CanUpcast` and `CanDowncast` play roles similar to `inj` and `prj` in *Data
types à la carte*. A projection can fail; exhaustiveness comes from combining handlers that cover
the whole sum.

### Trait implementations supply the evidence {#providers-are-the-theorys-computational-evidence}

CGP's trait implementations connect constraints to executable code. A generated `HasField` impl both
proves that a field is available and reads it. `CanBuildFrom` both requires compatible fields and
copies or moves them into a builder. These are concrete examples of the computational evidence
described by row theory; the correspondence does not give CGP a full row type system.

## What each approach costs

Row polymorphism lets one function serve many record shapes. Users value the reduced refactoring
([Fowler, *Row Polymorphism without the Jargon*](https://jadon.io/blog/row-polymorphism/)) and uses
such as codec-free JSON decoding and type-safe bindings
([Nguyen](https://hgiasac.github.io/posts/2018-11-18-Record-Row-Type-and-Row-Polymorphism.html)).

Row systems can make errors and domain models harder to understand. Row unification can produce very
large diagnostics; a PureScript discussion reports outputs around 152 KB ([PureScript
Discourse](https://discourse.purescript.org/t/upcoming-changes-to-error-messages-for-large-records-rows/3696)).
One user who modeled a domain with extensible records later preferred separate functions for nominal
types ([PureScript
Discourse](https://discourse.purescript.org/t/when-to-use-extensible-types-when-modeling-a-domain/217)).
Structural compatibility can also admit a call that makes little sense for the domain
([Fowler](https://jadon.io/blog/row-polymorphism/)). OCaml's manual warns that polymorphic variants
"result in a weaker type discipline" and recommends core variants for simple programs ([OCaml
manual](https://ocaml.org/manual/5.5/polyvariant.html)).

Some languages have narrowed their use of rows. PureScript replaced its row-typed effect monad
because "getting the effect rows to line up was sometimes quite tricky"
([PureScript-Resources](https://purescript-resources.readthedocs.io/en/latest/eff-to-effect.html)),
and Gleam removed row-typed records to become "less structural and more nominal in style" ([Gleam
v0.4](https://gleam.run/news/gleam-v0.4-released/)).

CGP requires derives and type-level tags, plus wiring for operations supplied by components. Missing
dependencies can produce long trait errors.
[`check_components!`](/docs/reference/macros/check_components) checks the listed components beside
the wiring. [`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it
recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class. A direct `match`
can still be easier to understand and debug.

CGP separates a provider's field requirements from the context's choice to use it. Matching fields
can satisfy the requirements, but they do not automatically select the provider. A `#[cgp_fn]`
function such as `full_name` is different: its single blanket implementation applies to any context
with the named fields, as a row-polymorphic function does. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where a row system is the better choice

A row system such as PureScript's fits programs that rely on concise anonymous records and full row
inference. CGP requires more declarations to express that style.

CGP fits nominal Rust code that needs structural access at selected points: a builder assembled from
parts, a visitor over an open set of variants, or a cast between related enums. These operations use
trait bounds and derives without changing Rust's type system.

## What to expect that differs

CGP expresses structural requirements within Rust's existing type system:

- **Field requirements are trait bounds.** Rust checks each `HasField` bound without inferring a
  residual row. Errors therefore come from trait resolution, rather than row unification.
- **Component behavior requires selection.** Matching fields can satisfy a provider's requirements,
  but the context must select that provider. A `#[cgp_fn]` blanket implementation applies wherever
  its bounds hold and does not require wiring.
- **Shapes must be exposed.** `HasFields` and `CgpData` derives supply the structural traits. CGP
  cannot inspect an arbitrary foreign type without the required implementations.
- **Structural operations keep nominal identities.** Builders and casts operate between declared
  Rust types; they do not introduce anonymous record types or full row inference.

## Where to go next

These pages develop the patterns and the neighbouring comparisons:

- [Extensible records](/docs/concepts/extensible-records): the builder pattern in full.
- [Extensible variants](/docs/concepts/extensible-variants): the visitor pattern and the expression
  problem.
- [Reflection](./reflection.md): the same type-level shapes compared with runtime and compile-time
  reflection.
- [Algebraic effects](./algebraic-effects.md): rows applied to effects, and how CGP's dependencies
  compare.
- [`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields) and
  [`CanBuildFrom`](/docs/reference/traits/casting/can_build_from): the constructs this page names.

## Sources

The PureScript snippet was compiled with PureScript 0.15.15 against the prelude and the OCaml
snippet with OCaml 5.5.0. The CGP snippets were compiled against `cgp` `0.8.0-alpha` .

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
