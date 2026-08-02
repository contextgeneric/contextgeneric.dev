---
sidebar_label: 'cgp_namespace!'
---

# `cgp_namespace!`

Define a reusable, inheritable wiring table that many contexts can join.

## What it's for

With [`delegate_components!`](./delegate_components.md) alone, every **context** — the type the capability
runs against, which supplies the values it needs as its fields — spells out its own wiring entry by entry.
Two contexts that should share the same providers repeat the same lines, and a table grows with the number
of components until the wiring is the largest thing in the file.

`cgp_namespace!` lifts a table out of any one context and gives it a name, so other contexts can adopt it:

```rust
cgp_namespace! {
    new AppNamespace {
        // shared wiring lives here
    }
}
```

A context then **joins** it with one line inside its own table, after which every lookup it does not wire
directly falls through to the namespace. Its own entries win, so a namespace behaves like a base
configuration each context specializes — most of the wiring for free, and a handful of overrides where they
matter.

That inherit-and-override behaviour is why **a namespace is how CGP expresses presets.** There is no
separate preset construct: a library publishes a namespace of sensible defaults, an application joins it and
changes the few entries it cares about. Namespaces can also inherit from one another, so a base can be
extended into a richer one that every downstream context picks up.

One structural point saves confusion later: **a namespace is not a context.** It is a trait — named after
the namespace — carrying a `Delegate` associated type and implemented once per key. Nothing instantiates it,
and it holds no wiring of its own at the context level; it only says where a lookup should go next.

## Using it

The body resembles a [`delegate_components!`](./delegate_components.md) table with an optional header. The
`new` keyword tells the macro to emit the namespace's trait and its backing struct:

```rust
cgp_namespace! {
    new MyNamespace {
        FooProviderComponent =>
            @MyFooComponent,
    }
}
```

Omit `new` only when the trait and struct are already declared elsewhere.

### The two entry forms

A namespace body accepts two kinds of entry, and they do different things.

| Entry | Meaning |
|---|---|
| `Key => @path` | **Redirect.** Asked for `Key`, look up `@path` instead. The provider is decided wherever the path lands. |
| `Key: Provider` | **Bind.** Asked for `Key`, resolve straight to `Provider`, as in `delegate_components!`. |

Redirection is what makes namespaces composable: because a lookup is keyed by a *path* rather than a bare
component name, a whole subtree can be rerouted at once, and a more specific path takes precedence over an
inherited one. Paths are written with the `@` sigil as dotted sequences — `@MyFooComponent`,
`@app.ErrorRaiserComponent`, `@cgp.core.error` — where lowercase segments become type-level strings and
capitalized segments name types. [`Path!`](./path.md) covers the syntax in full.

### The rest of the body grammar

**A namespace body is parsed by the same code as a
[`delegate_components!`](./delegate_components.md) table**, so everything that macro accepts parses here:
all three operators, all three key forms — including bracketed list keys and `@`-path keys with their
`[…]` and `{…}` groups — per-key generics, and the leading `open`, `namespace`, and `for` statements. That
page documents each of them; what follows is only what differs here.

What differs is what an entry becomes. A `delegate_components!` entry records a choice *for a context*; a
namespace entry records where a lookup for a key should *go next*, for any context that later joins. That
is why the two forms in the table above are the ones worth writing, and why the others are mostly not:

- **`->` direct delegation** still projects through the *value's* own table, so it names a concrete table
  inside a definition that is meant to be table-generic.
- **`open Component;`** is accepted and generates exactly what `Component => @Component,` generates —
  occasionally a convenient spelling for rooting a component's route at its own name.
- **`namespace Other;`** is accepted, but inheritance is written with the `: ParentNamespace` header
  above. That is the form overriding and the cycle diagnostics are defined in terms of.
- **A nested table value** — `UseDelegate<new Inner { … }>` — parses and then fails to compile. See
  [Gotchas](#gotchas).

### Inheriting from a parent

Name a parent after a colon in the header, and the child resolves everything the parent does plus its own
entries:

```rust
cgp_namespace! {
    new ExtendedNamespace: BaseNamespace {
        @cgp.core.error =>
            @app,
    }
}
```

The parent may itself be parameterized. The child's entries layer on top, and a path-rewriting entry like
the one above reroutes a whole subtree of the parent's namespace rather than a single component.

### The other two halves: registering, and joining

Defining a namespace is only a third of the pattern. A component **registers into** one with the
`#[prefix(...)]` attribute on its trait, which puts that component's lookups under a path:

```rust
#[cgp_component(ShowImpl)]
#[prefix(@show in AppNamespace)]
pub trait CanShow {
    fn show(&self) -> String;
}
```

A context **joins** one with a `namespace` header inside its own table, after which unwired lookups forward
through it:

```rust
delegate_components! {
    MyApp {
        namespace AppNamespace;

        @show.ShowImplComponent: ShowWithDebug,
    }
}
```

A third statement form, `for <T, Provider> in SomeTable { … }`, reads each entry of another lookup table and
emits one mapping per entry — which is how per-type defaults are pulled in wholesale.

## Examples

A complete namespace, end to end: a component registered under a prefix, a namespace to hold the route, and
a context that joins it and supplies the provider at the prefixed path.

```rust
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
#[prefix(@show in AppNamespace)]
pub trait CanShow {
    fn show(&self) -> String;
}

#[cgp_impl(new ShowWithDebug)]
#[uses(core::fmt::Debug)]
impl ShowImpl {
    fn show(&self) -> String {
        format!("{:?}", self)
    }
}

cgp_namespace! {
    new AppNamespace {}
}

#[derive(Debug)]
pub struct MyApp;

delegate_components! {
    MyApp {
        namespace AppNamespace;

        @show.ShowImplComponent: ShowWithDebug,
    }
}

check_components! {
    MyApp {
        ShowImplComponent,
    }
}
```

Reading the resolution: `MyApp.show()` looks up `ShowImplComponent`, finds no direct entry, falls through to
`AppNamespace`, which redirects to the path `@show.ShowImplComponent` — and `MyApp`'s own table binds that
path to `ShowWithDebug`. **The division of labour is the point.** The namespace owns the *route*; the context
owns the *provider*. A second context joins the same namespace and supplies a different provider at the same
path, with nothing duplicated between them.

Inheritance composes the same way. `ExtendedNamespace` resolves everything its parent does, so a context
joining the child inherits the whole chain:

```rust
cgp_namespace! { new BaseNamespace {} }

cgp_namespace! { new ExtendedNamespace: BaseNamespace {} }

delegate_components! {
    MyApp {
        namespace ExtendedNamespace;

        @show.ShowImplComponent: ShowWithDebug,
    }
}
```

## When to reach for it, and when not

**Reach for a namespace when the same wiring is repeated across contexts, or when a top-level table has
grown too long to read.** Those are the two problems it solves, and below that threshold it costs more than
it saves — a namespace adds a layer of indirection between a component and its provider, which is one more
hop for a reader tracing what runs.

Three lighter tools cover most cases, and it is worth knowing where each stops.

- **A plain [`delegate_components!`](./delegate_components.md) table** is right while each context's wiring
  is short and not shared. Most applications never outgrow this.
- **The [`open` statement](./delegate_components.md#choosing-a-provider-per-type-the-open-statement)** is the
  lightweight special case of the same path machinery, for dispatching *one* component on its type parameter
  directly on a context. It needs no namespace, no prefix, and no shared table — reach for it when the goal
  is per-type dispatch rather than shared wiring.
- **An [aggregate provider](./delegate_components.md#defining-the-target-at-the-same-time)** — the
  `new`-keyword bundle — packages a group of wirings that contexts adopt by *delegating* named components to
  it, rather than by joining and inheriting. It is the more direct mechanism and the better choice for a
  small, explicitly-delegated bundle. A namespace earns its extra machinery when there are many components,
  when inheritance is wanted, or when a library is publishing defaults for applications it does not know
  about.

Two things a namespace is *not* for. It will not make a single context's wiring shorter on its own — the
entries still have to exist somewhere. And it is not how one component gets per-type dispatch, which is
`open`'s job; the two do not combine on the same component, as the [Gotchas](#gotchas) explain.

## Under the hood

:::note

### Advanced

This section shows the trait and impls a namespace becomes. You do not need them to use one, but namespace
failures are reported in terms of these types, and the resolution is far easier to follow once you have seen
that a namespace is a trait with one associated type. `cargo cgp expand` prints the same thing for your own
code.

:::

`cgp_namespace!` emits, in order, an optional backing struct, an optional lookup trait, and one impl of that
trait per entry — plus one inheritance impl when a parent is named. From this input:

```rust
cgp_namespace! {
    new MyNamespace {
        FooProviderComponent =>
            @MyFooComponent,
    }
}
```

the `new` keyword emits a struct whose name wraps the namespace's, then the trait, which carries the table as
a parameter and a single associated type:

```rust
pub struct __MyNamespaceComponents;

pub trait MyNamespace<__Table__> {
    type Delegate;
}
```

Each `=>` entry becomes an impl of that trait **for the key**, whose `Delegate` is a
[`RedirectLookup`](../providers/redirect_lookup.md) aiming the table at the entry's path:

```rust
impl<__Table__> MyNamespace<__Table__> for FooProviderComponent {
    type Delegate = RedirectLookup<__Table__, Path!(@MyFooComponent)>;
}
```

Read it back as: `MyNamespace`'s delegate for `FooProviderComponent` is "look up the path `MyFooComponent`
inside whatever `__Table__` is". The namespace names no provider — it only reroutes, so the provider is
decided wherever the path finally lands. A `:` entry skips the indirection and maps the key straight to a
provider, one impl per key when the array form is used.

`#[prefix(...)]` is the other side of this, and it generates an impl of exactly the same shape. From
`#[prefix(@show in AppNamespace)]` on a `CanShow` component:

```rust
impl<__Components__> AppNamespace<__Components__> for ShowImplComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@show.ShowImplComponent)>;
}
```

So the component contributes its own route into the namespace, under the prefix, ending at its own component
name. That is why a joining context binds `@show.ShowImplComponent` rather than the bare component. A
component may carry several `#[prefix]` attributes to register into several namespaces.

When a parent is named, the macro prepends a blanket impl forwarding every key the parent resolves:

```rust
impl<__Table__, __Key__, __Value__> ExtendedNamespace<__Table__> for __Key__
where
    __Key__: BaseNamespace<__ExtendedNamespaceComponents>,
    __Key__: BaseNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
```

For any key the parent resolves, the child resolves it to the same value. The child's own entries are emitted
after this impl and win where their keys are more specific.

Two naming details appear verbatim in errors and are worth recognizing: the table parameter is literally
`__Table__`, and the inheritance impl uses `__Key__` and `__Value__`. And every `@` path is a
[type-level list](../types/type_level_spines.md) built by [`Path!`](./path.md) — `expand` resugars it to
`Path!(@…)` form, while a raw compiler error prints the underlying spine.

<details>
<summary>Formal grammar</summary>

The body is a header and a table, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpNamespace    -> Generics? `new`? NamespaceName ( `:` ParentNamespace )? `{` NamespaceBody `}`

NamespaceName   -> IDENTIFIER GenericArgs?
ParentNamespace -> TypePath GenericArgs?

NamespaceBody   -> Statement* ( Mapping ( `,` Mapping )* `,`? )?
```

`NamespaceBody` is [`delegate_components!`](./delegate_components.md)'s `TableBody` production unchanged,
so its `Statement` and `Mapping` rules — every operator, every key form, every value form — are defined on
that page rather than restated here. The two a namespace normally uses are `=>` to an `@`-path and `:` to a
provider. The `:` between `NamespaceName` and `ParentNamespace` is the inheritance colon, distinct from a
mapping's. `NamespaceName` becomes both a trait and, with `new`, a struct.

Two of the three statement forms in that shared production exist for this macro, because their job is
joining a context's table to a namespace. A `NamespaceStmt` — `namespace SomeNamespace;` — forwards every
unwired lookup through the named namespace. A `ForStmt` —
`for <T, Provider> in SomeTable where … { … }` — binds a key variable and a provider variable, reads each
entry of the table named after `in`, and emits one mapping per entry; its body admits only the `:` form,
and its optional `where` clause is merged into every impl the loop generates. Like
`delegate_components!`, the body accepts **no attributes** on any entry and rejects any it finds.

</details>

## Gotchas

**A context cannot override a path its namespace itself terminates.** Joining with `namespace N;` emits a
blanket `DelegateComponent` impl covering every path `N` resolves, so a direct entry for one of those paths
is a second impl for the same key and the compiler rejects the overlap with `E0119`. Overriding works only on
a path the namespace *routes to* without binding — which is why the example above has the namespace own the
route and the context own the provider. A path the namespace binds with a `:` entry or a `#[default_impl]`
has to be changed in the namespace instead. The same restriction stops a child namespace from redefining a
key its parent binds.

**Joining two namespaces on one context conflicts**, for the same reason — two blanket forwarding impls each
covering every key:

```text
error[E0119]: conflicting implementations of trait `DelegateComponent<_>` for type `App`
```

A bare-key `for` loop (`for <Key, Value> in Table { Key: Value }`) alongside a `namespace` join collides the
same way, which is why a loop's key is normally embedded in a path — `@app.SomeComponent.Key: Value`.

**`open` does not combine with a prefixed component in a joined namespace.** Once a component's lookups are
routed under a prefix, `open` — which roots the route at the bare component name — no longer reaches those
entries. Write the per-value entries with the full prefixed path instead.

**A cyclic parent chain and a self-inheriting namespace fail differently.** Two namespaces inheriting each
other make the forwarding impl's `where` clause loop, and the compiler rejects it eagerly at both
definitions:

```text
error[E0275]: overflow evaluating the requirement `__Key__: A<__BComponents>`
note: required for `__Key__` to implement `B<__AComponents>`
```

A namespace inheriting *itself* — `new A: A {}` — does not overflow. It produces a forwarding impl whose
value parameter nothing can determine:

```text
error[E0207]: the type parameter `__Value__` is not constrained by the impl trait, self type, or predicates
```

Both mean the inheritance chain is not acyclic, but only the first says so recognizably.

**A nested table inside a namespace entry parses and then fails to compile.** The
[`UseDelegate<new Inner { … }>`](./delegate_components.md#the-two-value-forms) value is accepted by the
parser, but `cgp_namespace!` — unlike `delegate_components!` — never lifts the inner table out into its own
struct and impls, so the entry ends up naming a type nothing declares:

```rust
cgp_namespace! {
    new NestedNs {
        FooProviderComponent:
            UseDelegate<new FooTable {
                String: DummyFoo,
            }>,
    }
}
```

```text
error[E0425]: cannot find type `FooTable` in this scope
```

The message names the missing table rather than the unsupported form, so it reads like a typo. Declare the
table in its own `delegate_components! { new FooTable { … } }` block and bind the key to
`UseDelegate<FooTable>` — or, since the nested form is legacy anyway, leave per-type dispatch to the
context and its [`open` statement](./delegate_components.md#choosing-a-provider-per-type-the-open-statement).

**A registered component with no provider bound anywhere still compiles.** If a `#[prefix]` routes a component
into a namespace and nothing ever binds a provider at its path, the redirect lands on an empty slot and the
context compiles regardless. Only a [`check_components!`](./check_components.md) reports it, and what it
reports is the *path* failing to resolve rather than a provider being absent:

```text
error[E0277]: the trait bound `PathCons<Symbol<4, Chars<'s', ...>>, ...>: AppNamespace<...>`
              is not satisfied
```

The `Symbol<4, …>` is the prefix — `show` — so the message is saying "this route is not something the
namespace resolves". This is the lazy-wiring problem in namespace clothing, and the answer is the same: check
the context.

## Related constructs

- [`delegate_components!`](./delegate_components.md) — where a context joins a namespace, and the source of
  the shared table grammar.
- [`Path!`](./path.md) — the `@`-path syntax namespace entries and `#[prefix]` use.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the provider every redirect resolves through.
- [`DefaultNamespace`](../traits/default_namespace.md) — inherited and per-type default resolution,
  including `#[default_impl(...)]`.
- [`#[cgp_component]`](./cgp_component.md) — the host of the `#[prefix(...)]` attribute.
- [`DelegateComponent`](../traits/delegate_component.md) — the per-key table a redirect finally walks.
- [`check_components!`](./check_components.md) — the only thing that catches a route bound to nothing.

## Source

- Entry point: [`cgp_namespace.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_namespace.rs)
- Implementation: [`types/namespace/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/namespace/)
- The `#[prefix(...)]` attribute: [`types/attributes/prefix.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/prefix.rs)
- Runtime traits and `RedirectLookup`: [`cgp-component/src/namespaces.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/namespaces.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
