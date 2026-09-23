---
sidebar_label: 'Reflection'
sidebar_position: 10
description: "CGP's type-level shapes read against Bevy's runtime reflection, Zig's comptime, and Rust's compile-time reflection work."
---

# Reflection and compile-time introspection

CGP lets you write a generic operation over a type's fields by representing those fields as types.
It is a language extension built as a [stable Rust library](/docs/), with pluggable trait
implementations at compile time and ordinary Rust consumer traits. This page
compares CGP with Bevy's runtime reflection, Zig's `comptime`, and Rust's emerging compile-time
reflection. It uses a field writer to show what CGP can express and where reflection is more useful.

## In your terms

A **context** is the type a CGP method runs on. It supplies data through fields and chooses
implementations through wiring. Here an application context chooses how to write each field type;
the struct being written is passed to it as a value.

The reflection vocabulary maps onto CGP by role:

| In a reflection system | In CGP |
| --- | --- |
| A type descriptor: `TypeInfo`, `Shape`, `std.builtin.Type`, `FieldId` | A `#[derive(HasFields)]` type's `Fields`, a type rather than a value |
| A field's name, as a runtime or `comptime` string | `Tag::VALUE`, a `&'static str` recovered from a [type-level string](/docs/reference/glossary#type-level-string) |
| A field's type, as an opaque `TypeId` or `Shape` | The field's type, carried as a real type parameter |
| `inline for` over `@typeInfo`, or a walk over a descriptor | A trait recursion over the `Cons`/`Nil` field list |
| A registry mapping types to behavior | The **wiring table**, written with `delegate_components!` |
| An access that fails at runtime | A `check_components!` failure at compile time |

## The idea, briefly

Reflection lets code inspect a type's shape without knowing that type in advance. Serializers,
editors, and debuggers use it to find fields by name and type. Systems differ in when they inspect
the shape:

- **At runtime:** A type carries metadata that the program inspects while it runs, as in Java, Go,
  C#, Python, and Bevy.
- **Through a generated descriptor:** A derive generates a static descriptor that runtime code
  walks, as in Rust's facet crate.
- **At compile time:** The compiler evaluates the introspection and emits only the specialized
  result, as in Zig's `comptime`, D, C++26's static reflection
  ([P2996](https://isocpp.org/files/papers/P2996R13.html)), and Rust's nightly work.

CGP also processes shapes at compile time. Its shape is a type, rather than a compile-time value.

### Runtime reflection: Bevy

`bevy_reflect` is a widely used runtime-reflection system in Rust. A type opts in with
`#[derive(Reflect)]`. A value can then be handled as a `Box<dyn Reflect>`, accessed by string field
name through the `Struct` trait, and downcast to its concrete type at runtime
([`bevy_reflect` documentation](https://docs.rs/bevy_reflect/latest/bevy_reflect/)). `TypeInfo`
describes a type's shape without an instance, and a runtime `TypeRegistry` maps types to their
`TypeInfo` and to associated behavior. With these, Bevy can serialize any registered component into
a scene or drive an editor over types it was not specialized for
([`TypeRegistry`](https://docs.rs/bevy/latest/bevy/reflect/struct.TypeRegistry.html)). Every
reflective access goes through dynamic dispatch, a downcast, or a registry lookup, so a mismatch
surfaces at runtime as a `None` or a panic.

### Compile-time metaprogramming: Zig's `comptime`

Zig's `comptime` is the usual reference design for compile-time reflection. Types are ordinary
compile-time values: `@typeInfo(T)` decomposes a type into a `std.builtin.Type` describing its
shape, `inline for` unrolls a loop over a compile-time collection, and `@field` accesses a field by a
name known at compile time ([Zig language reference](https://ziglang.org/documentation/master/#typeInfo)).
From Zig 0.14 the type tags are lowercase, so a struct is matched as `.@"struct"`. This function
writes any struct as a JSON-like object and compiles with Zig 0.16:

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

The loop unrolls and `@field` resolves at compile time. Zig's `@Type` also constructs a type from a
`std.builtin.Type` value; CGP cannot construct a new nominal type this way.

Compile-time introspection adds no runtime lookup, but its checks happen late. A `comptime` routine
is fully type-checked only when applied to a concrete type, so an error surfaces at the use site, once
per instantiation, as with C++ templates.

### Compile-time reflection comes to Rust

Rust programs have used derive macros that generate specialized code per type in place of
reflection. The [facet](https://fasterthanli.me/articles/introducing-facet-reflection-for-rust)
crate responds to the cost of that approach. serde's derives monomorphize the serializer anew for
every type, so facet's derive generates *data* instead: a `const SHAPE` descriptor that shared
runtime reflection code walks.

The nightly compiler work goes further, and it is unstable and subject to change. Under
`#![feature(type_info)]`, `TypeId::of::<T>()` gains query methods that reach a type's variants and
fields in source order. Each `FieldId` answers `name()`, `type_id()`, and `offset()`. The source's
own doc tests read:

```rust
#![feature(type_info)]
use std::any::TypeId;

struct Point { x: u32, y: u32 }

assert_eq!(const { TypeId::of::<Point>().fields(0) }, 2);
assert_eq!(const { TypeId::of::<Point>().field(0, 0).name() }, "x");
assert_eq!(const { TypeId::of::<Point>().field(0, 0).type_id() }, TypeId::of::<u32>());
```

These queries can be called only at compile time, and that restriction shapes the whole design.
Each query carries a `#[rustc_comptime]` marker, because supporting runtime calls would require
"some global table somewhere that maps all `TypeId`s to their repr"
([Rust project goal, *Reflection and comptime*](https://rust-lang.github.io/rust-project-goals/2026/reflection-and-comptime.html)).
The goal names Zig as its model but does not pursue full `comptime` for now, estimating that
architectural change as more than five years away. The
[tracking issue](https://github.com/rust-lang/rust/issues/146922) lists every stabilization step as
open. The plan runs from 2026 to 2028, with the aim that crates such as Bevy can work with arbitrary
types instead of requiring a derive. The details in this subsection come from the compiler source and
the tracking issue as they stood when this page was written, so check them before relying on one.

## How CGP expresses it

CGP encodes a type's structure as types and processes it with trait resolution. The example is a
self-contained field writer modeled on
[`cgp-serde`](https://github.com/contextgeneric/cgp-serde)'s `SerializeFields` provider, which does
the same over serde's `Serializer`. Its context is an
[environmental context](/docs/reference/glossary#environmental-context): `App` stands for an
application and chooses how each field type is written. The written `Value` is a type parameter, so
the component is [parameter-targeted](/docs/reference/glossary#parameter-targeted-component).

### A type's shape becomes a type, not a descriptor

`#[derive(HasFields)]` represents a struct's structure as a type-level list. The list plays the role
of the `TypeInfo`, `Shape`, and `FieldId` descriptors above:

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

The list records each field's name and type. Unlike the Rust MVP's opaque `TypeId` value,
`Field<Tag, Value>` carries the field's actual type as a type parameter. CGP can therefore dispatch
on that type. The [Extensible records](/docs/concepts/extensible-records) page develops the derive.

### Serializing through the trait system

One generic writer works over any `HasFields` type, as the Zig `jsonStringify` above works over any
struct. It delegates to a trait that recurses over the field list:

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

The `FieldsWriter` trait takes the place of `inline for`. It recurses over the
[`Cons`/`Nil`](/docs/reference/types/cons) list, with one impl for a non-empty list and a base case
for the empty one:

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

Each step of a reflective serializer has a counterpart in this code:

- **The loop over fields:** The recursion over `Cons` and `Nil`.
- **The field name:** `Tag::VALUE`, recovered through the
  [`StaticString`](/docs/reference/traits/formatting/static_string) trait without runtime metadata.
- **Access by name:** `value.get_field(PhantomData)`, resolved statically through
  [`HasField`](/docs/reference/traits/field-access/has_field).
- **Writing each field's value:** The `Context: CanWriteValue<FieldValue>` bound, which dispatches
  on the field's type through the context's wiring.

The context's wiring then chooses a writer for each type:

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

The struct also derives `HasField` so the recursion can read each field, and `WriteWithDebug`
formats a value with `Debug`. `cgp-serde`'s
[`SerializeFields`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/fields.rs)
performs the same recursion over a `serde::Serializer`. Its deserialization counterpart reads a
field name as a runtime string and compares it with each field's compile-time `Tag::VALUE`.

### Against facet and the reflection MVP

serde, facet, the Rust MVP, and CGP all let serialization be written once rather than per type.
They differ in what they turn a type's shape into and in what walks the result:

| Approach | What the type's shape becomes | What processes it |
| --- | --- | --- |
| serde | Code: a separate generated impl per struct | Nothing; the generated impl is the serializer |
| facet | Data: a `const SHAPE` generated by a derive | A shared routine, at runtime |
| Rust MVP | Data provided by the compiler, without a derive | `const`-evaluated code, at compile time |
| CGP | Type-level data generated by a derive | A shared trait recursion, monomorphized to direct code |

CGP keeps each field's type available to the trait solver. facet's `Shape` and the MVP's
`FieldId::type_id()` describe it through an opaque value. CGP's `Field<Tag, FieldValue>` instead
carries `FieldValue` as a type parameter, letting `WriteFields` require a statically checked writer
for that field type. The context's wiring can also select different writers for the same type in
different applications. facet, serde, and the MVP do not supply this
[per-context choice](/docs/concepts/coherence).

CGP still [monomorphizes](/docs/reference/glossary#monomorphization) the recursion for each field
list. It avoids writing a serializer per type and adds per-context choice, but it does not provide
facet's shared runtime code. CGP also needs `#[derive(HasFields)]`; it cannot inspect a foreign type
that lacks the derive.

### Checked where the code is written, not where it is instantiated

Zig's `comptime` and the MVP's `const fn` reflection are checked at instantiation: a routine is fully
checked only when it is applied to a concrete type, so an error surfaces at the use site. CGP's
generic code is checked where it is defined. The compiler checks the `FieldsWriter` impl once against
the bounds in its `where` clause, as it checks any generic Rust impl. Wiring is
[lazy](/docs/concepts/check-traits), so [`check_components!`](/docs/reference/macros/check_components)
verifies that a context supplies everything its wiring transitively needs and reports a missing
dependency at the wiring site. Trait bounds give this modular checking, and templates do not. The
[type classes](./type-classes.md) page compares CGP with the type-class systems that Rust's traits
come from.

### Selecting behavior and types through the same wiring

CGP uses the same type-level mechanism for two jobs that a structural reflection facility does not
handle directly. Frameworks do select behavior through reflection, Spring through annotations and
Bevy through `TypeData` in its registry, but they do so at runtime, through a registry built on top
of the introspection. CGP selects behavior at compile time through the context's wiring, which trait
resolution processes together with the structure. The [`#[cgp_type]`](/docs/reference/macros/cgp_type)
machinery also lets a context fix an abstract *type* through the same wiring. The
`Context: CanWriteValue<FieldValue>` bound above shows the combination: one mechanism walks the shape
and chooses how each field is written.

## What each approach costs

Reflection lets a framework inspect user types it was not written for. Compile-time reflection aims
for "the expressiveness of runtime reflection with the performance of hand-written code"
([*Compile-Time Reflection with @typeInfo*](https://hive.blog/hive-196387/@scipio/learn-zig-series-32-compile-time-reflection-with-typeinfo)).
facet and the Rust effort aim to reduce the compile-time and binary-size cost of derive-generated
code.

The costs depend on when reflection runs. Runtime lookup and dynamic dispatch can add work compared
with direct field access
([*The Hidden Cost of Reflection in Go*](https://dev.to/devflex-pro/the-hidden-cost-of-reflection-in-go-why-your-code-is-slower-than-you-think-41ee)).
A field named by a string tag can fail at runtime
([Go reflection guide](https://medium.com/@mojimich2015/golang-reflection-the-guide-to-runtime-type-inspection-manipulation-and-best-practices-303087684576)).
It also affects tooling, because renaming a field can silently break reflective access. Compile-time
reflection addresses these costs but has its own: errors appear at instantiation, compilation does
more work, and, as the Rust tracking issue itself asks, "monomorphization-time errors" can arise deep
inside an instantiation.

CGP requires derives and compile-time knowledge of the type, and it still generates specialized
code per type. Its wiring also needs maintenance. Deep field recursion can produce long trait
errors; [`cargo cgp check`](/docs/cargo-cgp/check) identifies the root cause for errors it
recognizes, but does not yet handle every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against simpler
approaches.

## Where reflection is the better choice

Runtime reflection fits programs that inspect types while running: an editor over live values, a
heterogeneous component registry, or deserialization into a type chosen from configuration. Some
language reflection systems can also inspect foreign types without a CGP derive. Constructing a new
nominal type from a computed shape calls for a facility such as Zig's `@Type` or C++26's splicers.
CGP fits generic operations over known shapes when static field types and per-context behavior matter.

CGP and Rust's emerging compile-time reflection complement each other more than they compete. The
MVP produces reflection as *const values*, which libraries that reflect over values can consume
directly. CGP consumes reflection as *types*, which the MVP does not supply. For CGP to drop its
derive, the compiler would need to expose a type's shape as types. Until then, CGP's derive stands in
for that compiler-provided view.

## What to expect that differs

**CGP has no reflection API.** It has no `TypeInfo` value to hold, no field list to iterate
imperatively, and no `TypeId::of::<T>()` to call. The structure is a type that trait impls dispatch
on, so reflecting over it means writing a recursive impl.

**A field's type is a type, not a `TypeId`.** This lets CGP recurse into a typed, checked writer for
each field, which the MVP's `TypeId`-based API cannot yet do.

**Shapes are opt-in.** A type exposes its structure only by deriving `HasFields` or `CgpData`, so CGP
cannot see the fields of a foreign type without the derive.

**CGP cannot synthesize types.** `Product!` and `Sum!` correspond to the result of `@typeInfo`, and
trait recursion corresponds to `inline for`, but CGP has no counterpart to `@Type` for constructing a
new nominal type.

**CGP is not a reflection system.** It encodes a type's structure as types, resolves operations
over it with the trait system, checks generic code where it is defined, and leaves no reflection
metadata in the compiled program.

## Where to go next

These pages develop the constructs and the neighbouring comparisons:

- [Extensible records](/docs/concepts/extensible-records) and
  [Extensible variants](/docs/concepts/extensible-variants): the derives and what they enable.
- [Row polymorphism](./row-polymorphism.md): the structural-typing theory behind the type-level
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
