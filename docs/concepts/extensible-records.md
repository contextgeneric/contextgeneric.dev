---
sidebar_label: 'Extensible records'
sidebar_position: 13
---

# Extensible records

Building and reading a struct through the names of its fields, without naming the struct.

This page answers *how does code assemble a struct it does not know the type of?* It shows what a
struct becomes when its shape is a type, how a value is built one field at a time with the compiler
tracking what is still missing, and the pattern that follows: independent pieces of a program each
contributing part of a whole. It closes on the opt-in requirement, which is the honest limit of all of
this.

## What a closed struct cannot do

A Rust struct is closed against incremental construction. A struct literal names the concrete type and
supplies every field at once, so building one is something a single place has to do, and that place
grows a line for every subsystem the program acquires:

```rust
let app = App {
    database: connect_to_database()?,
    http_client: build_http_client()?,
    metrics: start_metrics()?,
    // …and every future subsystem edits this same expression
};
```

Everything about that is fine until the subsystems want to be independent of each other and of `App`.
A database module that could build its own piece without knowing what it is a piece *of* would be
reusable across applications; written this way it cannot be, because the only thing that can name a
field of `App` is code that names `App`.

## A struct as a list of named fields

The move is to give the shape a type.
[`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) gives a struct a description of itself
that generic code can read:

```rust
#[derive(CgpData)]
pub struct DatabaseClient {
    pub url: String,
    pub pool_size: u32,
}
```

Its shape is now available as a type-level list, one entry per field, each pairing the field's name
with the field's type:

```rust
Product![Field<Symbol!("url"), String>, Field<Symbol!("pool_size"), u32>]
```

`Symbol!("url")` is the field name lifted into a type, so the trait system can match *by name*. Nothing
here is inspected at run time; the list exists during compilation and is gone afterwards. This is the
same field-name-as-type idea behind an [implicit argument](./implicit-arguments.md), scaled from one
field to a whole struct.

## Built one field at a time, checked all the way

With the shape available, a value can be assembled by parts that never meet. Building goes through a
**partial record**, a companion type carrying, per field, whether that field is present yet:

```rust
let app: App = App::builder()
    .build_from(database)
    .build_from(http)
    .finalize_build();
```

`builder()` produces the partial record with every field absent. Each `build_from` merges a smaller
struct's fields into it, matching them by name and flipping each from absent to present. The types
change at every step, tracking what has been filled in.

That tracking is the whole safety argument. `finalize_build` exists **only** for the configuration with
every field present, so finalizing early is not a runtime panic or a `None`. It does not compile:

```rust
// `http_timeout` has never been set, so there is no `finalize_build` to call.
let app: App = App::builder()
    .build_from(DatabaseConfig { database: "postgres://…".to_owned() })
    .finalize_build();
```

And because presence is tracked per field rather than by position, the order the pieces arrive in does
not matter.

## The pattern this is for

The machinery buys the **extensible builder**: each subsystem is a provider producing its own
small struct, knowing nothing about the target or about its siblings, and a dispatcher runs them all and
merges the outputs into the whole.

The list of contributors is then wiring, so swapping a subsystem is a line, and a second application
built from a different mix of the same providers is another table. The same thing as above guarantees
the mix is complete: if the providers between them do not supply every field, the build does not
finalize and the code does not compile.

The dispatcher is one of the [dispatch combinators](./dispatching.md), which is where the general form
of "run a handler per field" lives.

## What it costs

**Only types that opted in.** All of this works on a struct that derives the shape. A struct from
another crate that has not is invisible to it. That is the honest difference between this and runtime
reflection, and the thing to say first when someone reads "generic over any type's structure".

**Fields match by name and type, with nothing checking intent.** Two subsystems that both produce a
`timeout: u32` are producing the same field as far as merging is concerned. The decoupling depends on
exactly that, and it means a rename in one place silently stops matching in another.

**The error messages are the worst in CGP.** A missing field surfaces as an unsatisfiable bound over a
partial record whose type spells out every field and its presence marker. It is accurate, it is long,
and there is no version of it that reads like "you forgot `http_timeout`" without the toolchain's help.

**And it is a lot of machinery for a constructor.** An application assembled in one place, by code that
is allowed to know the type, should be assembled with a struct literal. This earns its keep when the
contributors must not know the whole: a plugin set, a framework building a user's type, or several
applications sharing subsystems.

## Where to go next

[Extensible variants](./extensible-variants.md) is the dual: an enum as a sum of named variants, with
the same tracking used for exhaustiveness instead of completeness. [Dispatching](./dispatching.md) is
what runs a handler per field and assembles the result, and is the page that makes the builder pattern
above concrete.

For the constructs, [`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) is the umbrella
derive and its slices are [`#[derive(HasField)]`](/docs/reference/derives/derive_has_field) for reading
one field, [`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields) for the whole shape, and
[`#[derive(BuildField)]`](/docs/reference/derives/derive_build_field) for the builder.
[`HasFields`](/docs/reference/traits/shape/has_fields) is the whole-shape view those produce,
[`HasBuilder`](/docs/reference/traits/builder/has_builder) is the builder family, and
[`Field`](/docs/reference/types/field) and [`Symbol!`](/docs/reference/macros/symbol) are the pieces a
shape is made of.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
