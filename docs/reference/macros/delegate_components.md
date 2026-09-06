---
sidebar_label: 'delegate_components!'
sidebar_position: 4
---

# `delegate_components!`

Build a context's wiring table, naming the provider that implements each of its components.

## Overview

[`#[cgp_component]`](./cgp_component.md) separates the trait callers use from the trait
implementations target, which leaves one question open: for a given type, *which* implementation
supplies the behavior? `delegate_components!` answers it. You give it a type and a list of entries,
the component name as the key and the chosen provider as the value, and it records that choice on the
type.

```rust
delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}
```

Think of the result as a **table**: a compact list saying which implementation supplies each
capability. But one point needs precision, because the analogy suggests a wrong conclusion. This looks
like an object's method table, but the compiler resolves it entirely at compile time: the keys and
values are types, the lookup happens during trait resolution, and the result monomorphizes to a direct
call. The compiled program contains neither a table nor dynamic dispatch, and the binary holds nothing
for a provider you did not choose.

The type that carries this table is a **context**: the type that owns the wiring. Often it is a type
standing for your whole application, whose job is to carry choices rather than data. `struct App;`
without fields is a perfectly good context. Sometimes, as above, it is the data type itself.

You need a macro here because each entry expands to two impls rather than one. Besides the impl
that stores the choice, the macro emits a second one that forwards the chosen provider's own
dependencies back through the table. So a missing transitive requirement produces a usable error
instead of one that does not name the cause. [Under the hood](#under-the-hood) shows both.

## Usage

The macro takes a target type and a brace-delimited body. The body holds any number of **statements**
followed by any number of **mappings**, and a mapping is a **key**, an **operator**, and a **value**,
chosen independently. This section covers the target forms first, then the operators, the key forms,
the value forms, and the statements.

**Every one of these forms combines with the others inside a single block.** A table routinely opens a
component for per-type dispatch, joins a namespace, and still maps plain component names to providers
alongside them. The only ordering rule is that statements come first.

The simplest table needs none of that:

```rust
delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}
```

### Defining the target at the same time

A leading `new` keyword makes the macro declare the target struct as well.

```rust
delegate_components! {
    new GeometryComponents {
        AreaCalculatorComponent: RectangleArea,
        PerimeterCalculatorComponent: RectanglePerimeter,
    }
}
```

Use this form to build an **aggregate provider**: a zero-sized provider whose only job is to hold a
table dispatching each component to a sub-provider, so that other contexts can delegate a whole group
of components to it as one unit. An aggregate provider is a *provider*, not a context. It forwards
each component's provider trait onward, and a call never resolves with the bundle in the context
position. So you must wire it with plain `delegate_components!` and never with
[`delegate_and_check_components!`](./delegate_and_check_components.md). That macro's check asks whether
the bundle can use each component *as* a context, so it proves nothing either way.

### One table for a family of types

A leading generic list makes the whole table generic, wiring every instantiation at once:

```rust
delegate_components! {
    <T> MyContext<T> {
        AreaCalculatorComponent: RectangleArea,
    }
}
```

### The operators

The operator between a key and its value decides what the entry resolves to. The parser accepts any
operator with any key form. The notes below say which pairings are worth writing.

| Operator | Reads as | The entry resolves to |
|---|---|---|
| `Key: Provider` | **Name a provider** | that provider, exactly as written |
| `Key -> Table` | **Delegate directly** | whatever `Table`'s own entry for `Key` holds |
| `Key => @path` | **Redirect** | wherever the path lands, resolved later |

Nearly all wiring uses `:`, and the rest of this page's examples assume it unless they say
otherwise.

**`->` is direct delegation.** Instead of naming a provider it forwards the lookup one hop into another
table, and the macro adds a bound requiring that the named table actually has an entry for that key.
This is how one table adopts a single choice from another rather than repeating it:

```rust
delegate_components! {
    new ScaledGeometryComponents {
        AreaCalculatorComponent:
            ScaledAreaCalculator<RectangleArea>,     // the provider *is* this scaled calculator
        PerimeterCalculatorComponent ->
            GeometryComponents,     // the provider is whatever GeometryComponents wires for perimeter
    }
}
```

Compare the two lines, because the difference is easy to miss. The `:` entry names
`ScaledAreaCalculator<RectangleArea>` outright, so `ScaledGeometryComponents` scales the area
calculation. The `->` entry reaches into `GeometryComponents`'s table and copies out whatever it holds
for `PerimeterCalculatorComponent`, so the perimeter stays exactly as `GeometryComponents` already
computes it, unscaled.

**`=>` is redirection.** It sends the lookup along a type-level [path](./path.md) rather than to a
provider, so the place where the path finally lands decides the provider. On a context, that path names
a slot in the context's own table, so you can point several components at one shared slot and answer
them from a single entry:

```rust
delegate_components! {
    App {
        [AreaCalculatorComponent, PerimeterCalculatorComponent] =>
            @shared,

        @shared: RectangleGeometry,
    }
}
```

The `open` statement below uses this same mechanism: `open AreaCalculatorComponent;` is
another spelling of `AreaCalculatorComponent => @AreaCalculatorComponent,` and generates the same impl.
The other use of `=>` is rerouting a whole path *prefix* at once, which is a
[namespace](./cgp_namespace.md) concern more than a per-context one.

### The key forms

**A single key** is one component name, and may carry generics of its own:
`<Shape> ShapeAreaCalculatorComponent<Shape>: SumAreas` adds a parameter to just that entry.

**A list key** is a bracketed list, expanding to one entry per name so several components share one
value:

```rust
delegate_components! {
    MyComponents {
        [
            AreaCalculatorComponent,
            PerimeterCalculatorComponent,
        ]: RectangleGeometry,
        ColorComponent: SolidColor,
    }
}
```

Each bracketed element is a full key, so an element may carry its own generics: in
`[WidthKey<T1>, <T2> HeightKey<T1, T2>]: RectangleValue<T1>` only the second key introduces `T2`. The
list is a key form rather than an operator, so `[A, B] -> SomeTable` and `[A, B] => @somewhere` are as
legal as `[A, B]: Provider`. The `->` form adopts several of another table's entries at once, and the
`=>` form points several components at one shared slot.

**A path key** is an `@`-prefixed route rather than a bare name, and it addresses a slot behind a
redirect: the per-value slots an `open` statement opens, or the prefixed routes a
[namespace](./cgp_namespace.md) registers. `@AreaCalculatorComponent.Rectangle: RectangleArea` stores
`RectangleArea` where the `AreaCalculatorComponent` redirect lands when the dispatch type is
`Rectangle`. Segments follow [`Path!`](./path.md)'s convention: a lowercase, non-primitive identifier
becomes a type-level string, and anything else names a type. A segment may also carry generics, as in
`@SomeComponent.<'a, T> &'a T: SomeProvider`.

Two grouping forms expand one path key into several, and **they are not interchangeable**:

- **`[…]` groups alternatives for one segment**, and the path may continue after it.
  `@app.[AreaCalculatorComponent, PerimeterCalculatorComponent].[u64, String]: RectangleGeometry`
  writes four entries: the cartesian product of the two groups.
- **`{…}` groups whole remainders**, and ends the path. Its elements may be different lengths and may
  nest further groups, which lets one entry cover routes of different shapes.

```rust
delegate_components! {
    App {
        namespace ExtendedNamespace;

        @app.{
            ErrorRaiserComponent.{&'static str, String},
            ErrorWrapperComponent,
        }: RaiseFrom,
    }
}
```

Remember it this way: `[…]` is a choice *within* a segment and `{…}` is a choice *of tails*.

### The value forms

A value is normally just a type: the provider, or the table a `->` forwards into. One other form
exists, and it is legacy.

:::info

### Legacy: read, don't write

Older code dispatches per type by nesting a table inside a [`UseDelegate`](../providers/use_delegate.md)
value instead of using the `open` statement below. **Prefer `open` for anything new**: it needs neither
a separate table type nor a wrapper. This page keeps the form because you will see it in existing
code, including in CGP's own error and handler components, which are still defined this way.

:::

Nesting a table inside a `UseDelegate` value looks like this:

```rust
delegate_components! {
    MyApp {
        AreaCalculatorComponent:
            UseDelegate<new AreaCalculatorComponents {
                Rectangle: RectangleArea,
                Circle: CircleArea,
            }>,
    }
}
```

A few details of the form are easy to miss. The inner braces hold a **full table body**, so an inner
table accepts everything an outer one does, nesting included. The **wrapper is not fixed to
`UseDelegate`**: the macro accepts any single-parameter wrapper type, which is how a component
dispatched on a tuple of parameters gets wired to a matching `UseDelegate2`. And the **inner table's
name may carry generics**, which a per-entry generic on the outer key threads into:

```rust
delegate_components! {
    new MyComponents {
        <T> WidthKey<T>: UseDelegate<new WidthValue<T> {
            HeightKey: HeightValue<T>,
        }>,
    }
}
```

One prerequisite is not visible in either snippet: **the component must carry
[`#[derive_delegate(UseDelegate<Shape>)]`](../attributes/derive_delegate.md)**, which generates the
dispatch impl the wrapper resolves through. Leave it off and the table still expands, then fails at the
check with an unsatisfied `IsProviderFor` that never mentions the missing attribute.

### Choosing a provider per type: the `open` statement

When a component is generic over a type parameter, you usually want a different provider for each
value of it. The `open` statement folds those per-value entries straight into the context's own table.
The per-type wiring on this page therefore dispatches a generic `CanCalculateArea<Shape>`, whose
provider trait is `AreaCalculator<Context, Shape>`, rather than the parameterless component the
opening example wires:

```rust
#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}
```

A leading `open …;` header opens one or more components, and `@`-path entries then assign a provider
per key:

```rust
delegate_components! {
    MyApp {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: RectangleArea,
        @AreaCalculatorComponent.Circle: CircleArea,
    }
}
```

Braces are optional when opening a single component, so `open AreaCalculatorComponent;` and
`open { AreaCalculatorComponent };` are the same. The braced list is needed to open several at once,
and the macro rejects a braceless header naming more than one component.

`open` does not need an extra attribute on the component, because it works through the
[`RedirectLookup`](../providers/redirect_lookup.md) impl that every `#[cgp_component]` already
generates. It is a lightweight special case of the full [namespace](./cgp_namespace.md) feature, suited
to a context wiring its own components directly, and it does not combine with a joined namespace where
the component carries a [`#[prefix(...)]`](../attributes/prefix.md).

### The namespace statements

The remaining statements opt a context into a [`cgp_namespace!`](./cgp_namespace.md), and that page
describes them in full, since they are most often written there.

**`namespace SomeNamespace;`** joins the namespace, so every lookup the table does not wire directly
falls through to it. **`for <T, Provider> in SomeTable { … }`** reads each entry of another lookup
table and emits one mapping per entry, which is how a context adopts per-type defaults wholesale. Its
body holds only `:` mappings, and an optional `where` clause on the loop constrains the entries it
wires.

### Statements come first

**Every statement must precede every mapping.** Putting one after a mapping is a *parse* error, so the
message names the unexpected token rather than the ordering rule. Beyond that the order is free:
several statements may appear in any order, and they mix freely with the mappings that follow.

```rust
delegate_components! {
    App {
        open AreaCalculatorComponent;

        PerimeterCalculatorComponent -> GeometryComponents,

        [ColorComponent, LabelComponent]: DefaultStyle,

        @AreaCalculatorComponent.{Rectangle, Circle}:
            ShapeArea,

        @AreaCalculatorComponent.[Square, Triangle]:
            PolygonArea,
    }
}
```

Read as a whole, that is still one table. Every line lowers to the same pair of impls. The forms
differ only in how many entries a line produces and in what each entry resolves to.

### Attributes

The macro **does not accept attributes** anywhere: not on the table, not on a key, and not on a key
inside a `for` loop or a nested table. It rejects any it finds rather than ignoring them.
Attribute-driven variants such as `#[check_params(...)]` and `#[skip_check]` belong to
[`delegate_and_check_components!`](./delegate_and_check_components.md).

## Examples

A component, a provider, and a context wired to use it:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

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

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

fn print_area(rect: &Rectangle) {
    println!("area = {}", rect.area());
}
```

The benefit shows up when a second context wants the same capability answered differently. It
writes its own provider and its own table entry, and the code that calls `area()` does not change:

```rust
#[cgp_impl(new SquareArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] side: f64) -> f64 {
        side * side
    }
}

#[derive(HasField)]
pub struct Square {
    pub side: f64,
}

delegate_components! {
    Square {
        AreaCalculatorComponent: SquareArea,
    }
}
```

`Rectangle` and `Square` now both implement `CanCalculateArea`, through different providers reading
different fields. A function generic over `CanCalculateArea` serves both without knowing that any of it
happened. Each context's choice stays one line a reader can search for, and finding that line tells
them which implementation runs.

When one context needs a different provider *per type* rather than one provider outright, the
component carries the type as a parameter, as the generic `CanCalculateArea<Shape>` under
[Usage](#choosing-a-provider-per-type-the-open-statement) does. The context then opens the component
and fills the slots:

```rust
pub struct MyApp;

delegate_components! {
    MyApp {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: RectangleArea,
        @AreaCalculatorComponent.{Circle, Ellipse}: CurveArea,
    }
}
```

## When to use it

Use `delegate_components!` whenever a context needs to choose providers, which is any time you use
[`#[cgp_component]`](./cgp_component.md) at all. The real decision is not whether to wire but **how to
check the wiring**, because CGP's wiring is *lazy*: a table with a missing or wrong entry still
compiles, and the failure only surfaces later, where the capability is used.

- **Pair it with [`check_components!`](./check_components.md)** for anything beyond simple wiring. A
  separate check gives you full control over what it asserts: concrete parameters for generic keys,
  per-provider layers, and opened or namespaced wiring. This is why larger codebases keep the two
  macros apart.
- **Use [`delegate_and_check_components!`](./delegate_and_check_components.md)** when you are getting
  started or the wiring is plain `Component: Provider` entries. It fuses the two so you cannot forget
  the check. It accepts this whole grammar, but it derives checks only from mappings keyed on a
  component *name*, so it wires opened, redirected, and namespaced entries and leaves them unchecked
  without a warning.
- **Use plain `delegate_components!` without a check** for an **aggregate provider** (the
  [`new`-keyword form](#defining-the-target-at-the-same-time)). That target is a provider other contexts
  delegate to rather than a context in its own right, so a context-side check on it asks the wrong
  question: it either passes vacuously or blames the bundle for requirements a real context would have
  met. Verify it through a context that delegates to it instead.

Two more choices inside the table have a clear default. Prefer the **`open` statement** over the
legacy nested `UseDelegate` table for per-type dispatch, because it needs neither a separate table type
nor a wrapper. And use a **[namespace](./cgp_namespace.md)** rather than a longer table once the same
wiring is repeated across contexts, or once one table has grown too long to read. Below that
threshold, the extra hop costs more than it saves.

Never leave a context's wiring unchecked. The macro you use to check it depends on how complicated the
wiring is, but the need for a check does not.

## Under the hood

Each entry becomes a pair of impls. From the single-entry table above, the
[`DelegateComponent`](../traits/wiring/delegate_component.md) impl records the choice:

```rust
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = RectangleArea;
}
```

That impl alone is the whole lookup. The provider blanket impl generated by
[`#[cgp_component]`](./cgp_component.md) reads it, so `Rectangle` inherits `AreaCalculator<Rectangle>`
from `RectangleArea`, and the consumer blanket impl then gives `Rectangle` the `CanCalculateArea` trait.
The hand-written equivalent of one wiring entry is exactly that block.

The [`IsProviderFor`](../traits/wiring/is_provider_for.md) impl forwards the provider's requirements:

```rust
impl<__Context__, __Params__>
    IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
```

This is why missing dependencies stay diagnosable. `RectangleArea`'s own `IsProviderFor` impl carries
the `where` bounds it needs, so an unsatisfied requirement flows back through this forwarding impl to
the point of use. Note that these parameters are literally named `__Context__` and `__Params__` in the
emitted code.

**Every other form on this page lowers to that same pair.** A form changes only how many pairs one
line produces and what the `Delegate` type is.

**A list key repeats the pair per name.** A table pairing
`[AreaCalculatorComponent, PerimeterCalculatorComponent]` with `RectangleGeometry`, alongside a
`ColorComponent: SolidColor` entry, yields three `DelegateComponent` impls and three matching
`IsProviderFor` impls.

**A `->` mapping projects through the value's table**, and adds the bound that makes the projection
well-formed:

```rust
impl DelegateComponent<PerimeterCalculatorComponent> for ScaledGeometryComponents
where
    GeometryComponents: DelegateComponent<PerimeterCalculatorComponent>,
{
    type Delegate = <GeometryComponents as DelegateComponent<PerimeterCalculatorComponent>>::Delegate;
}
```

**A `=>` mapping wires a [`RedirectLookup`](../providers/redirect_lookup.md)** over the table and the
path. Against a plain key the path is complete, so `AreaCalculatorComponent => @shared` gives
`RedirectLookup<Table, Path!(@shared)>`. Against a path key, both sides instead end in a shared
wildcard parameter, so an entry like `@cgp.core.error => @app` rewrites one path prefix to another and
passes everything beyond it through unchanged.

**The `open` statement** expands to a redirect plus per-value entries stored on the context itself. The
header wires the component to a `RedirectLookup` rooted at the component name inside `MyApp`'s own
table:

```rust
impl DelegateComponent<AreaCalculatorComponent> for MyApp {
    type Delegate = RedirectLookup<MyApp, Path!(@AreaCalculatorComponent)>;
}
```

`AreaCalculatorComponent => @AreaCalculatorComponent,` produces this same impl, byte-for-byte. Each
`@AreaCalculatorComponent.Rectangle: RectangleArea` entry then stores its provider in that same table
under a path key:

```rust
impl<__Wildcard__>
    DelegateComponent<PathCons<AreaCalculatorComponent, PathCons<Rectangle, __Wildcard__>>>
    for MyApp
{
    type Delegate = RectangleArea;
}
```

The [`RedirectLookup`](../providers/redirect_lookup.md) impl appends the dispatch parameter onto the
path and reads the result back, so `MyApp: CanCalculateArea<Rectangle>` resolves to `RectangleArea`.

The tail of that key is a **generic `__Wildcard__` parameter rather than `Nil`**, and this makes it
work. The entry matches any path *beginning* with this component and this dispatch type, whatever the
lookup appends after it. So one entry answers the redirect without having to predict the exact length
of the path reaching it.

Know one display difference before you compare these listings against your own expansion. Both paths
above are [`PathCons`](../types/path_cons.md) lists, but `cargo cgp expand` prints them differently.
It resugars the header's redirect target to [`Path!`](./path.md) form while leaving the per-entry key
as the raw list, so the same kind of type appears in two spellings in one expansion.

**A grouped path key expands to the cartesian product**, one impl pair per combination, each keyed on a
full prefix ending in `__Wildcard__`. So
`@app.[AreaCalculatorComponent, PerimeterCalculatorComponent].[u64, String]: RectangleGeometry` emits
four pairs, and a braced group does the same with whole tails rather than single segments, producing
keys of different lengths.

**A nested-table value lifts its inner table out** into its own definition, wiring the outer entry to
`UseDelegate` over the generated type. So the legacy form is equivalent to writing two separate
`delegate_components!` blocks, the inner one carrying `new`. Its per-value entries key on the same
parameter `open` keys on. The only difference is that they live in a separate table type.

**The namespace statements** share one lowering: an impl generic over a `__Key__` and a `__Value__`,
bounded on the namespace trait with a `Delegate = __Value__` binding. This makes the namespace's answer
the context's answer. A bare `namespace DefaultNamespace;` produces the blanket form:

```rust
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
```

A `for` loop produces the same shape once per mapping in its body, plus any predicates its `where`
clause adds.

## Formal grammar

In the Rust Reference's [notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
DelegateComponents -> Generics? `new`? TargetType `{` TableBody `}`

TargetType    -> Type

TableBody     -> Statement* ( Mapping ( `,` Mapping )* `,`? )?

Statement     -> OpenStmt | NamespaceStmt | ForStmt

OpenStmt      -> `open` ( `{` Type ( `,` Type )* `,`? `}` | Type ) `;`

NamespaceStmt -> `namespace` IDENTIFIER `;`

ForStmt       -> `for` `<` IDENTIFIER `,` IDENTIFIER `>` `in` TypePath WhereClause?
                 `{` ( NormalMapping ( `,` NormalMapping )* `,`? )? `}`

Mapping       -> NormalMapping
               | Key `->` ProviderValue
               | Key `=>` PathValue

NormalMapping -> Key `:` ProviderValue

Key           -> SingleKey | MultiKey | PathKey
SingleKey     -> Generics? Type
MultiKey      -> `[` SingleKey ( `,` SingleKey )* `,`? `]`
PathKey       -> Generics? `@` PathHead

PathHead      -> KeySegment ( `.` PathHead )?
               | `[` KeySegment ( `,` KeySegment )* `,`? `]` ( `.` PathHead )?
               | `{` PathHead ( `,` PathHead )* `,`? `}`

KeySegment    -> Generics? PathSegment

PathValue     -> `@` PathSegment ( `.` PathSegment )*

PathSegment   -> Type

ProviderValue -> Type
               | IDENTIFIER `<` `new` InnerTable `>`

InnerTable    -> IDENTIFIER BoundFreeGenerics? `{` TableBody `}`
```

A leading `Generics` list makes the table generic over the target, and `new` additionally emits the
target struct. The operator choice is independent of the key form: `:` maps a key to a provider, `->`
delegates to the value's own entry for that key, and `=>` redirects along a path. `NormalMapping` is
named separately because a `ForStmt` body admits only that form.

The grouping forms inside a `PathHead` differ in what they group and in whether the path may
continue. A bracketed group holds alternative segments for one position and may be followed by `.` and
more path. A braced group holds alternative whole remainders and terminates the path, which is why only
the braced form can nest. Both expand to the cartesian product with the rest of the path.

The segment productions differ in what they permit, which is why they are named apart. A
`KeySegment` (inside a `PathKey`) may carry its own generic list, and the macro merges the parameters
of every segment along one path onto that entry's impls. A `PathSegment` (inside a `PathValue`, the
right-hand side of a `=>`) cannot carry a generic list or a group. It is [`Path!`](./path.md)'s own
production, which keeps the two in step. And every `Generics` list on a key is an impl-position list,
so the parser accepts a bound (`<T: Clone> WidthKey<T>: WidthProvider`) and rejects a parameter
*default*: `<T = u32>` fails with `invalid impl generics syntax`.

A `ProviderValue`'s nested-table form carries a full `TableBody`, so an inner table accepts every form
an outer one does. An `InnerTable` is an identifier with an optional generic list rather than a full
`TargetType`, because a nested table always names a fresh struct the macro declares. Its
`BoundFreeGenerics` is a definition-position list: it allows neither bounds nor defaults, and a `const`
parameter is written as the bare name. Put a bound on the entry's own generics instead. The parser
rejects a bound written on the inner table as `expected ','`, because the value parser tries the
nested-table form speculatively and then falls back to reading the whole value as a plain type.

The remaining rules restate points made above. An `OpenStmt` may omit its braces when opening exactly
one component. Every `Statement` precedes every `Mapping`. [`cgp_namespace!`](./cgp_namespace.md)
describes `NamespaceStmt` and `ForStmt`. The macro does not accept attributes anywhere and rejects any
it finds.

## Common Mistakes

**Wiring is lazy.** A table with a missing entry, or one naming a provider whose own dependencies are
unmet, still compiles. The failure appears later, at the place where code uses the capability, often
as a long error naming types you did not write. This is the single most common source of confusion with CGP. The
answer is to check the table (see [`check_components!`](./check_components.md)) and to run
[`cargo cgp check`](/docs/cargo-cgp/check) in place of `cargo check`, which leads with the root cause
for the classes it recognizes.

**Statements must lead the block**, and getting it wrong produces a misleading message. Once the parser
has consumed the statements it expects only mappings, so it reads the `open` keyword as a *key* and
then complains about the component name where it wanted an operator:

```text
error: expected `:`
```

The caret sits on the component being opened, so the message blames it rather than the misplaced
`open`, and never mentions the ordering rule.

**`[…]` and `{…}` in a path are not the same group.** A bracketed group holds alternatives for one
segment and may be followed by more path. A braced group holds whole tails and ends the path. So
`@AreaCalculatorComponent.{String, u32}.bool: RectangleGeometry` does not parse, and the message does
not mention groups. It reports the trailing `.bool` sitting where the operator should be:

```text
error: expected `:`
```

Use a bracketed group when the path must continue past the alternatives, and a braced group only at the
end.

**Naming a provider struct that does not exist** is easy to do when you wrote a provider with
[`#[cgp_impl]`](./cgp_impl.md) *without* the `new` keyword and never declared it separately. The error
names an unresolved type rather than anything about wiring.

**Two entries claiming one key conflict**, and the compiler reports it as a coherence error rather than
as a wiring one. This covers the obvious duplicate, an `open` header colliding with an explicit mapping
for the same component, and a generic `<Shape> AreaCalculatorComponent<Shape>` entry overlapping a
specific `AreaCalculatorComponent<Rectangle>` one. The same applies to a direct entry for a path a
joined namespace itself binds: see [`cgp_namespace!`](./cgp_namespace.md#common-mistakes).

## Related constructs

- [`#[cgp_component]`](./cgp_component.md) — defines the components this table wires.
- [`#[cgp_impl]`](./cgp_impl.md) and [`#[cgp_provider]`](./cgp_provider.md) — write the providers it
  names.
- [`check_components!`](./check_components.md) — asserts the table is complete and resolvable.
- [`delegate_and_check_components!`](./delegate_and_check_components.md) — wires and checks together,
  for the plain entries.
- [`DelegateComponent`](../traits/wiring/delegate_component.md) and
  [`IsProviderFor`](../traits/wiring/is_provider_for.md) — the traits each entry expands into.
- [`RedirectLookup`](../providers/redirect_lookup.md) — what `open` and `=>` resolve through.
- [`Path!`](./path.md) — the path syntax behind `@`-keys and `=>` values.
- [`UseDelegate`](../providers/use_delegate.md) — the legacy dispatch form `open` supersedes.
- [`cgp_namespace!`](./cgp_namespace.md) — reusable tables, for wiring that outgrows one context; it
  reuses this macro's whole body grammar.
- [`UseField`](../providers/use_field.md) — the usual value for a field-backed getter.

The ideas behind it:

- [Bypassing coherence](/docs/concepts/coherence) — why a context chooses its providers through a
  table rather than through impls.
- [Aggregate providers](/docs/concepts/aggregate-providers) — the `new`-keyword bundle, and why it
  is a provider rather than a context.
- [Checking your wiring](/docs/concepts/check-traits) — what makes an unchecked table a problem.

## Source

- Entry point: [`delegate_components.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/delegate_components.rs)
- Implementation: [`types/delegate_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/)
- Path keys and their grouping forms: [`types/path/path_head.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/path/path_head.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
