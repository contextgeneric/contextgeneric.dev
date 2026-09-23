---
sidebar_label: 'Reflection'
sidebar_position: 10
description: "How CGP compares with runtime reflection, Zig comptime, and Rust compile-time reflection."
---

# Reflection and compile-time introspection

CGP represents a struct's fields as types, letting generic code read them and select behavior at
compile time. It is a language extension built as a [stable Rust library](/docs/) , with pluggable
implementations behind ordinary Rust traits. This page compares that approach with Bevy's runtime
reflection, Zig's `comptime` , and Rust's experimental reflection API through a generic field
writer.

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
| Checking that an operation supports a field type | Trait bounds, verified when used or asserted with `check_components!` |

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

Bevy lets code inspect values through a common runtime interface. A type opts in with
`#[derive(Reflect)]` ; code can then access its fields by name or downcast it to a concrete type
([`bevy_reflect` documentation](https://docs.rs/bevy_reflect/latest/bevy_reflect/)). `TypeInfo`
describes the shape, and a
[`TypeRegistry`](https://docs.rs/bevy/latest/bevy/reflect/struct.TypeRegistry.html) associates types
with metadata and behavior. These facilities support scene serialization and editors that work
across registered types. A lookup for a missing field or an incorrect downcast can fail at runtime.

### Compile-time metaprogramming: Zig's `comptime`

Zig makes types available as compile-time values. `@typeInfo(T)` decomposes a type into a
`std.builtin.Type` describing its shape, `inline for` unrolls a loop over a compile-time collection,
and `@field` accesses a field by a name known at compile time ([Zig language
reference](https://ziglang.org/documentation/master/#typeInfo)). This Zig 0.16 example writes a
struct as a JSON-like object:

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

Zig resolves the loop and field accesses at compile time, without runtime lookup. A generic
`comptime` routine is checked for each concrete instantiation, so type-dependent errors appear when
it is used. Zig can also construct a new nominal type from a `std.builtin.Type` value through
`@Type` ; CGP does not offer that operation.

### Compile-time reflection comes to Rust

Rust libraries can generate either code or metadata from a type declaration. Serde's derives
generate serialization implementations that are specialized for each type and serializer.
[facet](https://fasterthanli.me/articles/introducing-facet-reflection-for-rust) generates a
`const SHAPE` descriptor that shared runtime code can inspect, aiming to reduce that duplication.

Rust's experimental `type_info` feature supplies metadata directly from the compiler. Under
`#![feature(type_info)]` , `TypeId::of::<T>()` gains query methods that reach a type's variants and
fields in source order. Each `FieldId` answers `name()` , `type_id()` , and `offset()` . The
source's own doc tests read:

```rust
#![feature(type_info)]
use std::any::TypeId;

struct Point { x: u32, y: u32 }

assert_eq!(const { TypeId::of::<Point>().fields(0) }, 2);
assert_eq!(const { TypeId::of::<Point>().field(0, 0).name() }, "x");
assert_eq!(const { TypeId::of::<Point>().field(0, 0).type_id() }, TypeId::of::<u32>());
```

The API shown here restricts these queries to compile time. Its `#[rustc_comptime]` marker avoids
requiring a runtime table of type descriptions. The
[Reflection and comptime project goal](https://rust-lang.github.io/rust-project-goals/2026/reflection-and-comptime.html)
aims to let libraries such as Bevy inspect types without requiring derives, while leaving full
Zig-style `comptime` outside its immediate scope. The feature remains experimental; consult the
[tracking issue](https://github.com/rust-lang/rust/issues/146922) before relying on its API.

## How CGP expresses it

CGP encodes a type's structure as types and processes it with trait resolution. The example
fragments form a field writer modeled on [`cgp-serde`](https://github.com/contextgeneric/cgp-serde)
's `SerializeFields` provider, which does the same over serde's `Serializer` . Its context is an
[environmental context](/docs/reference/glossary#environmental-context) : `App` stands for an
application and chooses how each field type is written. The written `Value` is a type parameter, so
the component is [parameter-targeted](/docs/reference/glossary#parameter-targeted-component) .

### Representing a struct as a field list {#a-types-shape-becomes-a-type-not-a-descriptor}

`#[derive(HasFields)]` represents a struct's structure as a type-level list. The list plays the role
of the `TypeInfo` , `Shape` , and `FieldId` descriptors above:

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

### Writing fields through trait recursion {#serializing-through-the-trait-system}

The `WriteFields` provider writes any type whose field list implements `FieldsWriter` . That bound
requires readable fields and a writer for each field type:

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

The `FieldsWriter` trait takes the place of `inline for` . It recurses over the
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

To complete these fragments, `Config` also needs `#[derive(HasField)]` , `App` needs a declaration,
and `config` needs a value. The `WriteWithDebug` provider formats a value with `Debug` ; this
illustrates traversal and dispatch rather than a complete JSON serializer. `cgp-serde` 's
[`SerializeFields`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/fields.rs)
performs the same recursion over a `serde::Serializer` . Its deserialization counterpart reads a
field name as a runtime string and compares it with each field's compile-time `Tag::VALUE` .

### Against facet and the reflection MVP

These approaches differ in how they represent and process a type's structure:

| Approach | What the type's shape becomes | What processes it |
| --- | --- | --- |
| serde | A generated serialization impl per struct | The impl calls a chosen serializer |
| facet | Data: a `const SHAPE` generated by a derive | A shared routine, at runtime |
| Rust MVP | Data provided by the compiler, without a derive | `const`-evaluated code, at compile time |
| CGP | Type-level data generated by a derive | A shared trait recursion, monomorphized to direct code |

CGP keeps each field's type available to the trait solver. facet's `Shape` and the MVP's
`FieldId::type_id()` describe it through an opaque value. CGP's `Field<Tag, FieldValue>` instead
carries `FieldValue` as a type parameter, letting `WriteFields` require a statically checked writer
for that field type. The context's wiring can also select different writers for the same type in
different applications. That [per-context choice](/docs/concepts/coherence) comes from CGP wiring;
structural metadata alone does not provide it.

CGP still [monomorphizes](/docs/reference/glossary#monomorphization) the recursion for each field
list. It avoids writing a serializer per type and adds per-context choice, but it does not provide
facet's shared runtime code. CGP also requires a `HasFields` implementation, usually supplied by a
derive; it cannot inspect an arbitrary foreign type without that implementation.

### Checking generic code and concrete wiring {#checked-where-the-code-is-written-not-where-it-is-instantiated}

Rust checks CGP's generic implementations against their declared trait bounds. For example, it
checks `FieldsWriter` without knowing which concrete struct it will process. Zig's `comptime`
routines are checked at instantiation, and Rust's reflection experiment can also produce errors
during evaluation for a concrete type. Rust still checks a `const fn` 's body against its declared
types and bounds.

CGP checks concrete dependencies when an operation is used or explicitly asserted. Wiring is
[lazy](/docs/concepts/check-traits) , so a delegation entry alone does not prove that its provider
can run. [`check_components!`](/docs/reference/macros/check_components) forces that check for the
listed components and parameters, reporting missing dependencies beside the wiring. The
[type classes](./type-classes.md) page explains the trait-system connection.

### Selecting behavior and types through the same wiring

CGP combines structural access with implementation selection. The
`Context: CanWriteValue<FieldValue>` bound lets the same traversal use different field writers in
different contexts. [`#[cgp_type]`](/docs/reference/macros/cgp_type) also lets a context select an
abstract type through wiring. Reflection frameworks can associate behavior with metadata too; Bevy,
for example, stores `TypeData` in its runtime registry. CGP resolves its choices through traits at
compile time.

## What each approach costs

Reflection reduces the need to write inspection code for each user type. Runtime descriptors can
also support shared code, as facet does, while compile-time introspection can generate direct
accesses. Neither approach guarantees a particular compile time or binary size.

Runtime reflection moves some checks into execution. A field lookup by string can fail after a
rename, and dynamic dispatch or registry lookup can add runtime work. Compile-time reflection moves
those operations into compilation, but type-dependent failures may surface only for a concrete
instantiation. Rust's [tracking issue](https://github.com/rust-lang/rust/issues/146922) explicitly
raises this concern.

CGP requires exposed field shapes, explicit wiring, and specialized code for each field list.
Learning the type-level representation and tracing recursive trait errors are additional costs.
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) compares these costs with simpler
approaches.

## Where reflection is the better choice

Runtime reflection fits programs that inspect types while running: an editor over live values, a
heterogeneous component registry, or deserialization into a type chosen from configuration. Some
language reflection systems can also inspect foreign types without a CGP derive. Constructing a new
nominal type from a computed shape calls for a facility such as Zig's `@Type` or C++26's splicers.
CGP fits generic operations over known shapes when static field types and per-context behavior
matter.

CGP and Rust's emerging compile-time reflection complement each other more than they compete. The
MVP produces reflection as *const values*, which libraries that reflect over values can consume
directly. CGP consumes reflection as *types*, which the MVP does not supply. For CGP to drop its
derive, the compiler would need to expose a type's shape as types. Until then, CGP's derive stands
in for that compiler-provided view.

## What to expect that differs

CGP's structural interface differs from a reflection API in several ways:

- **Shapes are types.** A recursive trait implementation processes a field list; there is no
  descriptor value to iterate over imperatively.
- **Field types remain available.** A `Field<Tag, Value>` carries `Value` as a type parameter, so
  trait bounds can require an operation for that exact type.
- **Shapes must be exposed.** Derives such as `HasFields` and `CgpData` provide the required traits.
  CGP cannot inspect an arbitrary foreign type that lacks those implementations.
- **Nominal types must be declared.** CGP can compose products and sums, but it cannot generate a
  new nominal struct or enum from a computed shape as Zig's `@Type` can.

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

The Zig snippet was compiled with Zig 0.16; the Rust snippet is the nightly source's own doc test
and is unstable. The CGP snippets were compiled against `cgp` `0.8.0-alpha` with a
`check_components!` assertion on the wired context.

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
