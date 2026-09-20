---
sidebar_label: 'Reflection'
sidebar_position: 10
description: "CGP's type-level shapes read against Bevy's runtime reflection, Zig's comptime, and Rust's compile-time reflection work."
---

# Reflection and compile-time introspection

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who thinks in
`TypeInfo`, `@typeInfo`, or `TypeId::field`: Bevy's runtime reflection, Zig's `comptime`, Go or
Java reflection, or Rust's own compile-time reflection effort. CGP reaches the same payoff as
compile-time reflection, one generic routine that works over any type's fields, but takes the
structure not as data a routine inspects but as *types* the trait system resolves against. The page
covers the spectrum from runtime to compile-time reflection, where CGP sits on it, what the type-level
encoding buys and costs, and what to expect that differs.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. On this page the context is an application type that wires how each field type is written,
and the reflected struct is a value the context operates on.

| In a reflection system | In CGP |
| --- | --- |
| A type descriptor: `TypeInfo`, `Shape`, `std.builtin.Type`, `FieldId` | A `#[derive(HasFields)]` type's `Fields`, a type rather than a value |
| A field's name, as a runtime or `comptime` string | `Tag::VALUE`, a `&'static str` recovered from a type-level string |
| A field's type, as an opaque `TypeId` or `Shape` | The field's type, carried as a real type parameter |
| `inline for` over `@typeInfo`, or a walk over a descriptor | A trait recursion over the `Cons`/`Nil` field list |
| A registry mapping types to behavior | The **wiring table**, written with `delegate_components!` |
| An access that fails at runtime | A `check_components!` failure at compile time |

## The idea, briefly

Reflection lets one piece of code work over the *shape* of a type it was not written for. Serializers,
DI containers, ORMs, editors, and debuggers all need to walk an arbitrary type's fields by name and
type, and reflection supplies that by making a type's structure available as something the program can
read. Systems span a spectrum. At the runtime end a type carries metadata the program inspects while it
runs (Java, Go, C#, Python, Bevy). In the middle a derive generates a static descriptor that runtime
code walks (Rust's facet). At the compile-time end the compiler evaluates the introspection and emits
only the specialized result (Zig's `comptime`, D, C++26's static reflection
([P2996](https://isocpp.org/files/papers/P2996R13.html)), and Rust's nightly effort). CGP sits just
past the compile-time end, where the structure is not even a compile-time *value* but a *type*.

### Runtime reflection: Bevy

`bevy_reflect` is the prominent Rust runtime-reflection system. A type opts in with
`#[derive(Reflect)]`, after which a value can be handled as a `Box<dyn Reflect>`, accessed by string
field name through the `Struct` trait, and downcast back to its concrete type at runtime
([`bevy_reflect` documentation](https://docs.rs/bevy_reflect/latest/bevy_reflect/)). Behind the
dynamic access is `TypeInfo`, a description of a type's shape available without an instance, and a
runtime `TypeRegistry` that maps types to their `TypeInfo` and to associated behavior, so Bevy can
serialize an arbitrary registered component into a scene or drive an editor over types it was never
specialized for ([`TypeRegistry`](https://docs.rs/bevy/latest/bevy/reflect/struct.TypeRegistry.html)).
Every reflective access goes through dynamic dispatch, a downcast, or a registry lookup, and a
mismatch surfaces at run time as a `None` or a panic.

### Compile-time metaprogramming: Zig's `comptime`

Zig's `comptime` is the reference design for compile-time reflection. Types are ordinary compile-time
values, `@typeInfo(T)` decomposes a type into a `std.builtin.Type` describing its shape, `inline for`
unrolls a loop over a compile-time collection, and `@field` accesses a field by a compile-time-known
name ([Zig language reference](https://ziglang.org/documentation/master/#typeInfo)). Since Zig 0.14
the type tags are lowercase, so a struct is matched as `.@"struct"`. This writes any struct as a
JSON-like object and compiles with Zig 0.16:

```zig
const std = @import("std");

fn jsonStringify(value: anytype, writer: anytype) !void {
    try writer.writeAll("{");
    inline for (std.meta.fields(@TypeOf(value)), 0..) |field, i| {
        if (i > 0) try writer.writeAll(", ");
        try writer.print("\"{s}\": {any}", .{ field.name, @field(value, field.name) });
    }
    try writer.writeAll("}");
}
```

Because `inline for` unrolls at compile time and `@field` resolves statically, it compiles to the
code a hand-written serializer would. `@Type` runs the introspection in reverse and constructs a type
from a `std.builtin.Type` value, which CGP cannot do. The defining trade of `comptime` is that it is
zero-cost but checked late: a `comptime` routine is fully type-checked only when applied to a concrete
type, so an error surfaces at the use site, per instantiation, as with C++ templates.

### Compile-time reflection comes to Rust

Rust filled the absence of reflection with derive macros that generate specialized code per type. The
[facet](https://fasterthanli.me/articles/introducing-facet-reflection-for-rust) crate objects to the
cost: serde's derives monomorphize the serializer anew for every type, so facet's derive generates
*data* instead, a `const SHAPE` descriptor that shared runtime reflection code walks once.

The nightly compiler effort is the more consequential development, and it is unstable and will
change. Under `#![feature(type_info)]`, `TypeId::of::<T>()` gains query methods that reach a type's
variants and fields by source order, each `FieldId` answering `name()`, `type_id()`, and `offset()`.
The source's own doc tests read:

```rust
#![feature(type_info)]
use std::any::TypeId;

struct Point { x: u32, y: u32 }

assert_eq!(const { TypeId::of::<Point>().fields(0) }, 2);
assert_eq!(const { TypeId::of::<Point>().field(0, 0).name() }, "x");
assert_eq!(const { TypeId::of::<Point>().field(0, 0).type_id() }, TypeId::of::<u32>());
```

The single most important design decision is that this reflection is callable *only at compile time*:
every query carries a `#[rustc_comptime]` marker, because supporting runtime calls would require "some
global table somewhere that maps all `TypeId`s to their repr"
([Rust project goal, *Reflection and comptime*](https://rust-lang.github.io/rust-project-goals/2026/reflection-and-comptime.html)).
The goal names Zig as the model and declines full `comptime` for now, estimating the architectural
change at more than five years away. The [tracking issue](https://github.com/rust-lang/rust/issues/146922)
lists every stabilization step as open, and the plan runs from 2026 to 2028, with the ambition that
crates like Bevy will work with arbitrary types instead of requiring a derive. Everything in this
subsection was read from the compiler source and tracking issues on the day of writing; check them
before relying on a detail.

## How CGP expresses it

CGP reaches compile-time reflection's payoff by a different route. It encodes a type's structure as
types and processes them with trait resolution, so the type system does the "reflection" rather than a
routine inspecting a descriptor. The worked example below is a self-contained field writer modeled on
[`cgp-serde`](https://github.com/contextgeneric/cgp-serde)'s `SerializeFields` provider, which does the
same over serde's `Serializer`. Its context is an **[environmental context](/docs/reference/glossary#environmental-context)**: the `App` that wires how
each field type is written stands for an application, and the written `Value` is a parameter.

### A type's shape becomes a type, not a descriptor

`#[derive(HasFields)]` lifts a struct's structure into the type system as a type-level list, CGP's form
of the `TypeInfo`, `Shape`, and `FieldId` descriptors the reflection systems build:

```rust
#[derive(HasFields)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

// generated:
// impl HasFields for Config {
//     type Fields = Product![
//         Field<Symbol!("host"), String>,
//         Field<Symbol!("port"), u16>,
//     ];
// }
```

This is the same information a descriptor carries, each field's name and type, but it is a *type*.
The Rust MVP's `FieldId` answers `type_id()` with an opaque `TypeId` value. CGP's `Field<Tag, Value>`
carries the field's actual type as a type parameter, and that difference lets CGP dispatch on
it. The [Extensible records](/docs/concepts/extensible-records) page develops the derive.

### Reflection-driven serialization in the trait system

One generic writer works over any `HasFields` type, the counterpart of the Zig `jsonStringify`
above. It delegates to a trait that recurses over the field list:

```rust
#[cgp_component(ValueWriter)]
pub trait CanWriteValue<Value> {
    fn write_value(&self, value: &Value) -> String;
}

#[cgp_impl(new WriteFields)]
impl<Value> ValueWriter<Value>
where
    Value: HasFields,
    Value::Fields: FieldsWriter<Self, Value>,
{
    fn write_value(&self, value: &Value) -> String {
        format!("{{{}}}", Value::Fields::write_fields(self, value))
    }
}
```

The `FieldsWriter` trait is the `inline for` of CGP: a recursion over the
[`Cons`/`Nil`](/docs/reference/types/cons) list, with a step for a non-empty list and a base case for
the empty one:

```rust
pub trait FieldsWriter<Context, Value> {
    fn write_fields(context: &Context, value: &Value) -> String;
}

impl<Context, Value, Tag, FieldValue, Rest> FieldsWriter<Context, Value>
    for Cons<Field<Tag, FieldValue>, Rest>
where
    Tag: StaticString,                        // the field name, as a const &'static str
    Value: HasField<Tag, Value = FieldValue>, // read this field from the value
    Context: CanWriteValue<FieldValue>,       // write the field through the context's wiring
    Rest: FieldsWriter<Context, Value>,       // recurse on the remaining fields
{
    fn write_fields(context: &Context, value: &Value) -> String {
        let field_value = value.get_field(PhantomData);
        let entry = format!("\"{}\": {}", Tag::VALUE, context.write_value(field_value));
        let rest = Rest::write_fields(context, value);
        if rest.is_empty() { entry } else { format!("{entry}, {rest}") }
    }
}

impl<Context, Value> FieldsWriter<Context, Value> for Nil {
    fn write_fields(_context: &Context, _value: &Value) -> String {
        String::new()
    }
}
```

Every reflection concept has a counterpart here. The recursion over `Cons`/`Nil` is the loop over the
field list. `Tag::VALUE`, recovered through the
[`StaticString`](/docs/reference/traits/formatting/static_string) trait, is the `field.name` a
reflection system reads, produced with no runtime metadata. `value.get_field(PhantomData)` is the
by-name access, resolved statically through [`HasField`](/docs/reference/traits/field-access/has_field).
And `Context: CanWriteValue<FieldValue>` is the recursive step: writing each field's value by
dispatching on the field's type, through the context's wiring:

```rust
delegate_components! {
    App {
        open ValueWriterComponent;

        @ValueWriterComponent.[String, u16]: WriteWithDebug,
        @ValueWriterComponent.Config: WriteFields,
    }
}

App.write_value(&config);   // {"host": "localhost", "port": 8080}
```

The struct also derives `HasField` so the recursion can read each field; `WriteWithDebug` formats a
value with `Debug`. `cgp-serde`'s [`SerializeFields`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/fields.rs)
is this same recursion over a `serde::Serializer`, and its deserialization dual reads a field name as
a runtime string and compares it against each field's compile-time `Tag::VALUE`.

### Against facet and the reflection MVP

Four approaches solve the same problem, writing serialization once rather than per type, and comparing
what each turns a type's shape into shows where CGP sits. serde's derive generates *code*, a bespoke
impl per struct. facet generates *data*, a `const SHAPE` walked by a shared runtime routine, which
avoids re-monomorphizing per type at the cost of runtime reflection work. The Rust MVP has the
compiler provide the data, consumed by `const`-evaluated code with no derive. CGP generates
*type-level data*, walked by a shared compile-time routine that monomorphizes to direct code.

Two differences make CGP's position distinct. First, the field's type is preserved as a type. Facet's
`Shape` and the MVP's `FieldId::type_id()` carry it as an opaque descriptor from which a generic
function cannot be instantiated directly, so recursion into a field's own type is mediated by stored
function pointers or is not yet expressible. CGP's `Field<Tag, FieldValue>` carries `FieldValue` as a
real type parameter, so `WriteFields` can require `Context: CanWriteValue<FieldValue>` and recurse
into a fully typed, statically checked writer for the field's type. Second, CGP dispatches each field
through the *context's* wiring, so the same type is written differently under different application
contexts, the [per-context choice](/docs/concepts/coherence) that facet, serde, and the MVP do not
have.

The costs are equally clear. Unlike facet, CGP does not solve the monomorphization problem: the
recursion instantiates per field list, so it produces specialized code per type as serde's output
does. What it saves is the authoring duplication, and what it gains is configurability, not binary
size. And unlike the Rust MVP, CGP requires the `#[derive(HasFields)]` opt-in and cannot see a foreign
type that did not derive it.

### Checked when the code is written, not when it is instantiated

Zig's `comptime` and the MVP's `const fn` reflection are checked at instantiation: a routine is fully
checked only when applied to a concrete type, so an error surfaces at the use site. CGP's generic code
is checked at its definition. The `where` clause on the `FieldsWriter` impl is verified once against
the bounds, and [`check_components!`](/docs/reference/macros/check_components) verifies that a context
supplies everything its wiring transitively needs. This is the modular checking traits give and
templates do not, and the [type classes](./type-classes.md) page draws it out against Haskell's
neighbours.

### Reflection that also selects behavior and configures types

CGP applies the same type-level idea to two things a structural reflection facility does not directly
address. Frameworks do select behavior by reflection, Spring by annotations and Bevy through
`TypeData` in its registry, but as a runtime, registry-mediated act layered on the introspection. CGP
folds behavior selection into the same compile-time table that carries its structural information, and
the [`#[cgp_type]`](/docs/reference/macros/cgp_type) machinery lets a context fix an abstract *type*
through that table too. The `Context: CanWriteValue<FieldValue>` bound above shows it: the reflection
that walks the shape and the wiring that interprets each field are one mechanism.

## What each approach costs

Reflection is relied upon because a framework written once works over every user type, and runtime
reflection additionally works on types the framework author never saw. Compile-time reflection is
valued for "the expressiveness of runtime reflection with the performance of hand-written code"
([*Compile-Time Reflection with @typeInfo*](https://hive.blog/hive-196387/@scipio/learn-zig-series-32-compile-time-reflection-with-typeinfo)),
and facet and the Rust effort are motivated by cutting derive codegen's compile-time and binary cost.
The costs, as their users state them, split by when the reflection runs. Runtime reflection pays in
performance, since Go's `encoding/json` re-inspects types on every call
([*The Hidden Cost of Reflection in Go*](https://dev.to/devflex-pro/the-hidden-cost-of-reflection-in-go-why-your-code-is-slower-than-you-think-41ee)),
in type safety, since a field named by a string tag fails at runtime
([Go reflection guide](https://medium.com/@mojimich2015/golang-reflection-the-guide-to-runtime-type-inspection-manipulation-and-best-practices-303087684576)),
and in tooling, since renaming a field silently breaks reflective access. Compile-time reflection
answers those and trades them for instantiation-time error messages, added compile-time work, and, as
the Rust tracking issue itself asks, the risk of "monomorphization-time errors" deep inside
instantiation.

CGP's costs are the ordinary ones plus two specific to this page. It introspects only types that opted
in by deriving its machinery, and only at compile time. It does not reduce the specialized code per
type the way facet does. Its wiring is code somebody writes. And its raw diagnostics are trait-solver
output over generated types, which for a deep field recursion can be long:
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where reflection is the better choice

When a program must inspect types at runtime, to deserialize into a type chosen from a config file,
serialize a heterogeneous registry of components, build an editor or a debugger over live values, or
reflect over a foreign type that cannot be made to derive anything, runtime reflection is the right
and only tool of the two, and `bevy_reflect` or facet is the tool to reach for. When a program needs to
construct a new nominal type from computed structure, Zig's `@Type` and C++26's splicers do what CGP's
type-level lists cannot. CGP is the better tool when a program wants generic-over-structure code that
costs nothing at runtime, is checked when written, recurses into field types with full type
information, and drives behavior and type selection as well as data walking.

CGP and Rust's emerging compile-time reflection are more complementary than competing. The MVP
produces reflection as *const values*, which value-reflection libraries consume natively. CGP consumes
reflection as *types*, a different projection the MVP does not supply, so for CGP to shed its derive
the compiler would need to expose a type's shape as types. Until then CGP's derive stands in for the
compiler-provided reflection Rust is still building.

## What to expect that differs

**There is no reflection API.** No `TypeInfo` to hold, no field list to iterate imperatively, no
`TypeId::of::<T>()` to call. The structure is a type that trait impls dispatch on, so reflecting over
it means writing a recursive impl.

**A field's type is a type, not a `TypeId`.** This is why CGP can recurse into a typed, checked
writer for each field and the raw MVP cannot yet.

**Shapes are opt-in.** A type exposes its structure only by deriving `HasFields` or `CgpData`. A
foreign type without the derive is invisible.

**No type synthesis.** CGP's `Product!` and `Sum!` are the `@typeInfo` result reified as types, and
its trait recursion is the `inline for`, but CGP cannot `@Type`-construct a new nominal type.

**CGP is not a reflection system.** It is compile-time structural reflection encoded as types,
resolved by the trait system, checked at the definition site, and erased before runtime.

## Where to go next

- [Extensible records](/docs/concepts/extensible-records) and
  [Extensible variants](/docs/concepts/extensible-variants): the derives and what they enable.
- [Row polymorphism](./row-polymorphism.md): the structural-typing theory beneath the type-level
  shapes.
- [Dynamic dispatch](./dynamic-dispatch.md): the runtime-mechanism counterpart to this page.
- [`#[derive(HasFields)]`](/docs/reference/derives/derive_has_fields),
  [`Field`](/docs/reference/types/field), and [`StaticString`](/docs/reference/traits/formatting/static_string):
  the constructs this page names.

## Sources

The Zig snippet was compiled with Zig 0.16; the Rust snippet is the nightly source's own doc test and
is unstable. The CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!`
assertion on the wired context.

- [`bevy_reflect` documentation](https://docs.rs/bevy_reflect/latest/bevy_reflect/), [`TypeInfo`](https://docs.rs/bevy/latest/bevy/reflect/enum.TypeInfo.html), and [`TypeRegistry`](https://docs.rs/bevy/latest/bevy/reflect/struct.TypeRegistry.html): the `Reflect` trait, the shape descriptor, and the runtime registry.
- [Zig language reference](https://ziglang.org/documentation/master/), [Comptime (zig.guide)](https://zig.guide/language-basics/comptime/), and [*Compile-Time Reflection with @typeInfo*](https://hive.blog/hive-196387/@scipio/learn-zig-series-32-compile-time-reflection-with-typeinfo): `comptime`, `@typeInfo`, `@Type`, `inline for`, and `@field`.
- [Rust tracking issue #146922](https://github.com/rust-lang/rust/issues/146922), the source at [`library/core/src/mem/type_info.rs`](https://github.com/rust-lang/rust/blob/master/library/core/src/mem/type_info.rs), and the [*Reflection and comptime* project goal](https://rust-lang.github.io/rust-project-goals/2026/reflection-and-comptime.html): the `type_info` API, the compile-time-only restriction, and the stabilization plan.
- [fasterthanli.me, *Introducing facet*](https://fasterthanli.me/articles/introducing-facet-reflection-for-rust): the derive-generates-data approach and its motivation.
- [`cgp-serde`, `SerializeFields`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/fields.rs): the production version of this page's field recursion, over serde.
- [P2996R13, *Reflection for C++26*](https://isocpp.org/files/papers/P2996R13.html): C++'s static reflection with `std::meta::info` and splicers.
- [*The Hidden Cost of Reflection in Go*](https://dev.to/devflex-pro/the-hidden-cost-of-reflection-in-go-why-your-code-is-slower-than-you-think-41ee) and [*Golang Reflection guide*](https://medium.com/@mojimich2015/golang-reflection-the-guide-to-runtime-type-inspection-manipulation-and-best-practices-303087684576): the runtime performance and type-safety costs of reflection.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
