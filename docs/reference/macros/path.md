---
sidebar_label: 'Path!'
---

# `Path!`

A type-level path, used by namespaces and redirected lookups.

## What it's for

`Path!(@a.B.c)` builds a type-level **route** — a list of segments naming a way through nested wiring tables.
Read left to right, each segment narrows a lookup one step: through a namespace, through a prefix, down to a
component key.

CGP needs such routes because [namespaces](./cgp_namespace.md) resolve lookups by path rather than by bare
component name, which is what lets a whole subtree be rerouted at once and a single inherited entry be shadowed
without disturbing the rest. Written as the underlying spine those routes are unreadable —
`PathCons<…, PathCons<…, Nil>>` nested several deep — so `Path!` lets one be written the way it reads:

```rust
Path!(@app.error.ErrorRaiserComponent)
```

It is the path-shaped sibling of CGP's other type-level construction macros. Where
[`Symbol!`](./symbol.md) turns a literal into a type-level string and [`Product!`](./product.md) and
[`Sum!`](./sum.md) build record and variant lists, `Path!` builds the routing list, sharing their right-nested,
`Nil`-terminated shape.

**You will more often write the syntax than the macro.** The same `@`-path form is embedded directly in
[`cgp_namespace!`](./cgp_namespace.md) entries, in `#[prefix(...)]` attributes, and in the `@`-path keys of
[`delegate_components!`](./delegate_components.md) — and that is where paths are normally written. The bare
macro is for the occasional case where a route needs naming as a type on its own.

## Using it

`Path!` takes a single `@`-prefixed path of one or more dot-separated segments. The leading `@` is required — it
is the sigil marking the body as a path rather than a plain type — and at least one segment must follow:

```rust
Path!(@app)
Path!(@app.error)
Path!(@app.error.ErrorRaiserComponent)
```

### How a segment is encoded

Each segment is parsed as a type, and **its first character decides how it is treated**. This is the one rule
worth learning, because it is what lets a path mix names and types without any extra syntax:

| The segment | Becomes |
|---|---|
| A single lowercase identifier — `app`, `error` | a [`Symbol`](./symbol.md) type-level string |
| A capitalized name — `ErrorRaiserComponent` | that named type |
| A primitive type name — `u32`, `bool`, `str` | that type, *not* a symbol |

So lowercase segments read as namespace and prefix names, capitalized ones as component keys or marker types,
and the primitive exception keeps `@u32` meaning the type `u32` rather than the string `"u32"`. Mixing is
normal: `@my_app.ShowImplComponent` interleaves a symbol and a component.

This is the same convention [namespaces](./cgp_namespace.md) describe for their `@`-paths, because it is
literally the same parser.

## Examples

Used directly, `Path!` names a route that a [`RedirectLookup`](../providers/redirect_lookup.md) resolves against
a table:

```rust
use cgp::prelude::*;

type ErrorRoute = Path!(@app.error.ErrorRaiserComponent);
// ErrorRoute = PathCons<Symbol!("app"),
//                  PathCons<Symbol!("error"),
//                      PathCons<ErrorRaiserComponent, Nil>>>
```

In practice the same syntax appears embedded rather than through the bare macro. A namespace entry redirects a
component along a path:

```rust
cgp_namespace! {
    new MyNamespace {
        FooProviderComponent =>
            @MyFooComponent,
    }
}
```

and a `#[prefix(...)]` attribute registers a component under one:

```rust
#[cgp_component(ShowImpl)]
#[prefix(@show in AppNamespace)]
pub trait CanShow {
    fn show(&self) -> String;
}
```

and a wiring table keys an entry on one:

```rust
delegate_components! {
    MyApp {
        namespace AppNamespace;

        @show.ShowImplComponent: ShowWithDebug,
    }
}
```

All four build the same kind of list. The macro and the embedded syntax are two places to write it, not two
different things.

## When to reach for it, and when not

**Write the `@`-path syntax wherever a namespace or a wiring key asks for a route**, and reach for the bare
`Path!` macro only when a route needs to be a named type on its own — which is rare.

- **Prefer the embedded form.** A namespace entry, a `#[prefix]`, or an `@`-path wiring key is where a route
  belongs, and each accepts the syntax directly. Naming a `type SomeRoute = Path!(…)` and using it indirectly
  usually makes the wiring harder to follow rather than easier.
- **Never hand-write the spine.** `PathCons<Symbol!("app"), PathCons<…, Nil>>` is what the macro expands to,
  and writing it out is longer and identical in meaning.
- **Reach for the [`open` statement](./delegate_components.md#choosing-a-provider-per-type-the-open-statement)
  rather than constructing paths yourself** when the goal is per-type dispatch on one component. `open` builds
  the route for you, and its `@Component.Key` entries are the paths — you write the keys, not the routing.
- **Do not use `Path!` as a general type-level list.** [`Product!`](./product.md) is the list for a sequence of
  types; `Path!` differs in its segment-classification rule, which exists to serve routing and would be
  surprising anywhere else.

## Under the hood

:::note

### Advanced

This section shows the spine the macro builds. You do not need it to use `Path!`, but a namespace failure prints
the expanded path, so recognizing the shape is what lets you read which route came up empty.
`cargo cgp expand` resugars it back for your own code.

:::

`Path!` expands to a right-nested chain of `PathCons` terminated by `Nil`, with each segment encoded by the
lowercase-versus-capitalized rule:

```rust
// before
Path!(@app.error.ErrorRaiserComponent)

// after — readable form
PathCons<
    Symbol!("app"),
    PathCons<
        Symbol!("error"),
        PathCons<ErrorRaiserComponent, Nil>,
    >,
>
```

The macro parses the segments after the `@` into a list and folds them right to left onto `Nil`, wrapping each in
a `PathCons` whose tail is the accumulated rest. A single-segment path is therefore `PathCons<Segment, Nil>`.

**A lowercase segment nests a second spine inside the first**, which is why a fully-expanded path is longer than
it looks: the `Symbol` is itself a `Chars`/`Nil` chain, so `@app` desugars all the way down to

```rust
PathCons<Symbol<3, Chars<'a', Chars<'p', Chars<'p', Nil>>>>, Nil>
```

That is what a raw compiler error prints, and it is the main reason this section is worth reading once. The
segment-classification rule is visible in the result: `@MyComp` gives `PathCons<MyComp, Nil>` with no `Symbol`
in it, and `@u32` gives `PathCons<u32, Nil>` rather than treating the primitive as a name.

The same fold drives the embedded forms. A [`cgp_namespace!`](./cgp_namespace.md) redirect
`FooProviderComponent => @MyFooComponent` produces a
`RedirectLookup<__Table__, PathCons<MyFooComponent, Nil>>`, and `#[prefix(@show in AppNamespace)]` on a
`CanShow` component produces a `PathCons<Symbol!("show"), PathCons<ShowImplComponent, Nil>>` — the prefix
followed by the component's own key.

One presentational note: `cargo cgp expand` resugars a path back to `Path!(@…)` form in most positions, but not
uniformly — an `open` statement's per-entry key comes back as a raw `PathCons` spine while its header's redirect
target is resugared. Seeing the same kind of type in two spellings in one expansion is expected rather than a
sign that they differ.

<details>
<summary>Formal grammar</summary>

The input is a leading `@` followed by one or more dot-separated segments, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
PathInput   -> `@` PathSegment ( `.` PathSegment )*

PathSegment -> Type
```

The leading `` `@` `` is required and at least one segment must follow. Each `PathSegment` is parsed as a Rust
`Type`, but its encoding is decided semantically: a single lowercase identifier that is not a primitive type name
becomes a `Symbol` type-level string, while every other segment — a capitalized name or a primitive — is kept as
the named type. This same grammar is what [`cgp_namespace!`](./cgp_namespace.md) entries and `#[prefix(...)]`
attributes embed, where it appears as the `Path` production.

</details>

## Gotchas

**The leading `@` is not optional**, and the macro says exactly what it wanted:

```text
error: expected `@`
```

**Case decides meaning, silently.** `@app` and `@App` are entirely different segments — a type-level string
versus a named type — and both are valid, so a capitalization slip produces a path that compiles and routes
somewhere else. The failure surfaces later as a lookup that resolves to nothing, with no hint that the cause was
a capital letter.

**A lowercase primitive is the type, not a string.** `@u32`, `@bool`, and `@str` name those types. That is
almost always what you want, and it means a path segment cannot be the *string* `"u32"` through this syntax.

**A path that routes nowhere still compiles.** A route is just a type; nothing checks that anything is bound at
its end. An unbound route is reported only when a [`check_components!`](./check_components.md) evaluates the
lookup, as an unsatisfied bound naming the expanded path — which is why namespace mistakes tend to surface at
the check rather than at the definition.

## Related constructs

- [`cgp_namespace!`](./cgp_namespace.md) — where paths are mostly written, in entries and `#[prefix]`
  attributes.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the provider that resolves a path against a table.
- [`delegate_components!`](./delegate_components.md) — `@`-path keys and the `open` statement that builds routes
  for you.
- [`Symbol!`](./symbol.md) — what a lowercase segment becomes.
- [Type-level spines](../types/type_level_spines.md) — the `PathCons` chain the expansion builds.
- [`Product!`](./product.md) and [`Sum!`](./sum.md) — the sibling construction macros, sharing the fold shape.
- [`DelegateComponent`](../traits/delegate_component.md) — the per-key table a resolved path finally reads.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — where paths are actually written, and what a route is
  for.

## Source

- Entry point: [`path.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/path.rs)
- Parsing and the fold: [`types/path/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/path/)
- Runtime `PathCons`: [`types/path.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/path.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
