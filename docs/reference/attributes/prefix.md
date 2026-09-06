---
sidebar_label: '#[prefix]'
sidebar_position: 8
---

# `#[prefix(...)]`

Register a component into a namespace under a type-level path, so every context that joins the
namespace addresses the component by that path.

## Overview

A [namespace](/docs/concepts/namespaces) is a reusable wiring table that a **context**, the type the
capability runs against, joins with one line and then overrides where it needs to. A namespace answers
a lookup by *routing* it: asked for a component, it says where to look next. `#[prefix(...)]` is how
a component contributes its own route. Written on the component's trait beside
[`#[cgp_component]`](../macros/cgp_component.md), it registers the component into the named namespace
under a path prefix:

```rust
#[cgp_component(Greeter)]
#[prefix(@app in AppNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}
```

Read that as: *in `AppNamespace`, `GreeterComponent` is found at `@app.GreeterComponent`.* A context
that joins `AppNamespace` and asks for `GreeterComponent` is sent to that path, and whatever is bound
there is the provider that runs. The attribute records only the route. A provider is bound at the
path somewhere else: by a direct `@app.GreeterComponent: GreetHello` entry on the context, by an
entry in the namespace's body, or by a [`#[default_impl]`](./default_impl.md) on a provider.

Prefixes turn a flat wiring table into a tree. With components registered under `@app.auth`,
`@app.finance`, and `@app.error`, related entries sort together, a reader finds the auth wiring
without reading the finance wiring, and a library can register its components into a shared
namespace for applications it will never see. CGP's own components do this. `HasErrorType` and the
error components carry `#[prefix(@cgp.core.error in DefaultNamespace)]`, and the handler family
carries `#[prefix(@cgp.extra.handler in DefaultNamespace)]`.

## Usage

Apply the attribute to a trait that carries [`#[cgp_component]`](../macros/cgp_component.md), or one
of the macros built on it, [`#[cgp_type]`](../macros/cgp_type.md) and
[`#[cgp_getter]`](../macros/cgp_getter.md). The argument has two parts joined by the keyword `in`:

```rust
#[prefix(@path.segments in NamespacePath)]
```

| Part | Meaning |
|---|---|
| `@path.segments` | The prefix: a `@`-sigil path of one or more dot-separated segments, in [`Path!`](../macros/path.md)'s syntax. The macro appends the component's marker after it. |
| `in` | Required keyword. |
| `NamespacePath` | The namespace to register into: `DefaultNamespace`, or one defined with [`cgp_namespace!`](../macros/cgp_namespace.md). It may be a qualified path and may carry generic arguments. |

**The path is a prefix, and the macro appends the component marker for you.**
`#[prefix(@app in DefaultNamespace)]` on `CanGreet` registers `GreeterComponent` at
`@app.GreeterComponent`. Writing the marker yourself doubles it; see
[Common Mistakes](#common-mistakes).

Segments follow [`Path!`](../macros/path.md)'s convention. A lowercase identifier that is not a
primitive type name becomes a type-level string, so `@app` and `@cgp.core.error` are strings. Any
other segment names a type, so `@MyApp.MyBarComponent` is two types. Case decides the meaning. A
segment cannot declare generic parameters, and the parser accepts neither of the `[…]` and `{…}`
grouping forms a [`delegate_components!`](../macros/delegate_components.md) key allows. One
attribute registers under exactly one prefix.

**Repeat the attribute to register into several namespaces**, one attribute per namespace:

```rust
#[cgp_component(BarProvider)]
#[prefix(@MyApp.MyBarComponent in MyNamespace)]
#[prefix(@my_app.MyBarComponent in OtherNamespace)]
pub trait Bar {
    fn bar(&self);
}
```

**A component with type parameters is addressed with the parameters after the marker.** The lookup
appends them to the path, so a context joining the namespace wires a `CanShow<T>` registered under
`@app` with entries such as `@app.ShowImplComponent.String: ShowWithDisplay`. Lifetime parameters
do not appear in the path.

**Any crate may register its own components into any namespace.** The emitted impl is for the
component's marker, a type the component's crate owns, so Rust's orphan rule is satisfied even when
the namespace comes from another crate. That is why an application can register into
`DefaultNamespace`. It is also the difference from [`#[default_impl]`](./default_impl.md), whose key
on a prefixed component is a foreign path.

### Choosing a prefix

A prefix is part of the component's public surface, and expensive to change once wiring depends on
it, so choose it for every implementation the component might have rather than for the one in front
of you. Give components separate sub-paths whenever they are likely to need separate providers, even
when the current wiring happens to treat them alike. Abstract types belong under a `types` sub-path
of their layer, such as `@app.auth.types`, because a production context typically points the logic
under `@app.auth` at a database while leaving the types on the same concrete choices.

Register prefixes into a **base** namespace, `DefaultNamespace` or a namespace that describes the
application's structure, and bind providers in a namespace that inherits it. A namespace's entry for
a key cannot be overridden once defined, so a namespace that held both the prefixes and one backend's
providers could not lend its prefixes to a second backend.

## Examples

A component registered under a prefix, a provider, and a context that joins the namespace and binds
the provider at the prefixed path:

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}

#[derive(HasField)]
pub struct App {
    pub name: String,
}

delegate_components! {
    App {
        namespace DefaultNamespace;

        @app.GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}
```

Reading the resolution: `app.greet()` looks up `GreeterComponent`. `App` does not wire it directly,
so the lookup falls through to `DefaultNamespace`, which redirects to `@app.GreeterComponent`, and
`App`'s own table binds that path to `GreetHello`. **Environmental context, self-targeted**: `App`
exists to carry the wiring and a `name` field, and the capability is about `App` itself.

A second context joins the same namespace and binds a different provider at the same path, with
nothing repeated between the two:

```rust
#[cgp_impl(new GreetFormally)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Good day, {name}.")
    }
}

#[derive(HasField)]
pub struct FormalApp {
    pub name: String,
}

delegate_components! {
    FormalApp {
        namespace DefaultNamespace;

        @app.GreeterComponent: GreetFormally,
    }
}
```

The prefixes CGP's own components carry are wired the same way. A context that joins
`DefaultNamespace` chooses its error type and an error strategy by full path, with the dispatch type
of the generic `CanRaiseError` component written after its marker:

```rust
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::ReturnError;

pub struct Service;

delegate_components! {
    Service {
        namespace DefaultNamespace;

        @cgp.core.error.ErrorTypeProviderComponent: UseType<String>,
        @cgp.core.error.ErrorRaiserComponent.String: ReturnError,
    }
}
```

## When to use it

**Reach for `#[prefix]` as soon as a wiring table has enough components that grouping helps a
reader.** It costs nothing at the point of use, it is the one namespace tool without a downstream
restriction, and it pays off at once in a table that reads as a directory listing. It is also the
form a library uses to publish its components into a shared namespace.

Some situations call for something else.

- **A component only ever wired directly on a context** needs no prefix. The `open` statement of
  [`delegate_components!`](../macros/delegate_components.md) dispatches it per type without a
  namespace, and the two do not combine on one component.
- **Binding a provider at a path** is the job of the wiring, not of the component. Use a direct entry
  on the context, a [`cgp_namespace!`](../macros/cgp_namespace.md) body entry, or
  [`#[default_impl]`](./default_impl.md) on the provider.
- **A small, explicitly delegated bundle** is an
  [aggregate provider](../macros/delegate_components.md#defining-the-target-at-the-same-time),
  which contexts adopt by delegating named components to it. Namespaces and prefixes earn their extra
  indirection when there are many components, when inheritance is wanted, or when a library publishes
  defaults for applications it does not know about.

## Under the hood

The attribute adds one impl to `#[cgp_component]`'s output, after the standard provider impls: an
impl of the namespace trait for the component marker, whose `Delegate` is a
[`RedirectLookup`](../providers/redirect_lookup.md) down the prefixed path. From
`#[prefix(@app in DefaultNamespace)]` on `CanGreet`:

```rust
impl<__Components__> DefaultNamespace<__Components__> for GreeterComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@app.GreeterComponent)>;
}
```

`__Components__` is the table the lookup runs against. The macro appends it as the namespace trait's
last generic argument and leaves it generic, so one registration serves every context that joins. A
namespace defined with `cgp_namespace!` gets the same shape against its own trait:
`impl<__Components__> MyNamespace<__Components__> for BarProviderComponent`.

The path is the prefix with the marker appended, a [`PathCons`](../types/path_cons.md) list.
`cargo cgp expand` prints it as `Path!(@app.GreeterComponent)`, and a raw compiler error prints the
underlying `PathCons<Symbol<3, Chars<'a', …>>, PathCons<GreeterComponent, Nil>>`.

Resolution then runs in three hops. Joining the namespace emits, on the context, a blanket
`DelegateComponent` impl whose `Delegate` is whatever the namespace answers, so asking `App` for
`GreeterComponent` yields `RedirectLookup<App, Path!(@app.GreeterComponent)>`. That provider's impl
of `Greeter` looks the path up in `App`'s own table, as `App: DelegateComponent<Path!(...)>`, and
forwards to the delegate it finds there, `GreetHello`. For a component with type parameters,
`RedirectLookup` appends the parameters to the path before the lookup, which is why per-type entries
carry them after the marker.

The impl has the same shape as a `=>` entry written in a namespace body:
`GreeterComponent => @app.GreeterComponent` inside `cgp_namespace!` emits the same item. The
attribute lets the component's own crate contribute that entry, which a foreign crate could not write
into the namespace's body.

Two details are easy to misread. The namespace's key is the *marker*, not the path, so a `help:`
list in an error names `GreeterComponent` as implementing the namespace trait even when the path it
routes to is empty. And a component marker declared with generics through the `name:` key of
`#[cgp_component]` carries those generics onto the impl after `__Components__`.

## Formal grammar

The attribute argument is a path, the keyword `in`, and a namespace path, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
PrefixArgs    -> Path `in` NamespacePath

Path          -> `@` PathSegment ( `.` PathSegment )*
PathSegment   -> Type

NamespacePath -> TypePath GenericArgs?
```

`Path` is [`Path!`](../macros/path.md)'s own production: a leading `@`, then one or more segments,
each parsed as a Rust `Type` and encoded as a type-level string or a named type by the rule above. A
segment cannot declare generic parameters, and the parser does not accept the `[…]` and `{…}` groups
of a [`delegate_components!`](../macros/delegate_components.md) path key. `NamespacePath` is a Rust
type path with optional generic arguments, and the macro appends the table argument itself. Both
parts are required, the attribute takes exactly one such argument, and it is repeated for several
namespaces.

## Common Mistakes

**The marker is appended for you, so do not write it.** `#[prefix(@app.GreeterComponent in Ns)]`
registers the component at `@app.GreeterComponent.GreeterComponent`. It compiles, and a context that
binds `@app.GreeterComponent` then finds its entry never consulted, because the route and the binding
name different paths. Write the prefix alone.

**Registering routes a component and binds nothing.** A prefixed component compiles even when
nothing binds its path, and so does a context that joins the namespace. Only a
[`check_components!`](../macros/check_components.md) reports it, as an unsatisfied bound on the
*path* rather than on a provider:

```text
error[E0277]: the trait bound `PathCons<Symbol<3, Chars<'a', ...>>, PathCons<GreeterComponent, Nil>>: DefaultNamespace<App>` is not satisfied
   |
   |         GreeterComponent,
   |         ^^^^^^^^^^^^^^^^ unsatisfied trait bound
   |
   = help: the following other types implement trait `DefaultNamespace<Components>`:
             ErrorRaiserComponent
             GreeterComponent
```

The `help:` list names `GreeterComponent` itself, which reads like a contradiction and is in fact the
diagnosis: the registration worked, and only the binding at the leaf is missing.

The [compile errors](../errors.md#nothing-is-wired-there) page reads this diagnostic in full. The
fix is to bind a provider at the path: a direct entry on the context, a namespace body entry, or a
`#[default_impl]` on the provider.

**Two attributes naming the same namespace conflict.** Each emits an impl of that namespace's trait
for the same marker, and the compiler rejects the second:

```text
error[E0119]: conflicting implementations of trait `AppNamespace<_>` for type `GreeterComponent`
```

A component has one route per namespace. To reach it under two paths, define two namespaces.

**`open` does not reach a prefixed component in a joined namespace.** `open` roots the route at the
bare marker, while the prefix routes under the path, so the per-type entries `open` expects are never
consulted. Write the entries with the full prefixed path instead, as
[`cgp_namespace!`](../macros/cgp_namespace.md#common-mistakes) explains.

**Case decides a segment's meaning silently.** `@app` is a type-level string and `@App` is a type,
both are valid, and a capitalization slip routes to a path nothing binds. The failure surfaces as the
unbound path above, without a hint about the letter; see
[`Path!`](../macros/path.md#common-mistakes).

**On `#[cgp_auto_getter]` the attribute is accepted and dropped.** That macro runs the same attribute
collector as `#[cgp_component]`, to apply `#[extend]` and `#[use_type]`, but it generates no
component to register, so a `#[prefix]` on it registers nothing and reports nothing. A getter that
must live in a namespace is a [`#[cgp_getter]`](../macros/cgp_getter.md) component.

**On `#[cgp_impl]` or `#[cgp_fn]` the attribute is unknown.** Nothing consumes it there, so it
reaches the compiler as an attribute that does not exist:

```text
error: cannot find attribute `prefix` in this scope
```

The provider-side counterpart is [`#[default_impl]`](./default_impl.md), which binds a provider at a
path rather than routing a component to one.

## Related constructs

- [`cgp_namespace!`](../macros/cgp_namespace.md) — defines the namespaces a prefix registers into,
  and the `=>` body entry that emits the same impl.
- [`delegate_components!`](../macros/delegate_components.md) — the `namespace` statement that joins,
  and the `@`-path entries that bind a provider at a prefixed path.
- [`#[default_impl]`](./default_impl.md) — the provider-side registration: it binds, where this
  attribute routes.
- [`DefaultNamespace`](../traits/namespace/default_namespace.md) — the built-in namespace CGP's own
  components register into.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the delegate every registration resolves
  through.
- [`Path!`](../macros/path.md) and [`PathCons`](../types/path_cons.md) — the path syntax and the
  type-level list it builds.
- [`#[cgp_component]`](../macros/cgp_component.md), [`#[cgp_type]`](../macros/cgp_type.md), and
  [`#[cgp_getter]`](../macros/cgp_getter.md) — the hosts.
- [`check_components!`](../macros/check_components.md) — the only thing that catches a route bound to
  nothing.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables, and why presets need
  no separate construct.

## Source

- The attribute: [`types/attributes/prefix.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/prefix.rs)
- Collection on the component: [`types/attributes/cgp_component_attributes.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/cgp_component_attributes.rs)
- Emission, after the standard provider impls: [`types/cgp_component/evaluated/item.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_component/evaluated/item.rs)
- A library use: [`has_error_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-error/src/traits/has_error_type.rs)
- Expansion snapshots: [`namespaces/`](https://github.com/contextgeneric/cgp/tree/main/crates/tests/cgp-tests/tests/namespaces/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
