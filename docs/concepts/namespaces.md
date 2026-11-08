---
title: 'Share and inherit wiring with namespaces'
sidebar_label: 'Namespaces'
sidebar_position: 9
description: 'How a namespace names and routes shared wiring, how contexts inherit it while filling unbound paths, and why an inherited binding cannot be overridden.'
---

# Namespaces

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations, and
namespaces keep the table that selects them short. Namespaces give shared wiring a name that
contexts can inherit. They organize component lookups under paths, allowing a shared table to bind
the choices its contexts have in common and leave other destinations for each context to fill. The
distinction matters: a context can fill an unbound path, but it cannot replace an inherited binding.

## Repeated wiring obscures the differences

When production and test contexts differ in only a few choices, writing their tables separately
repeats the choices they share. These fragments assume the named components, providers, and
contexts are defined, with CGP's prelude imported:

```rust
delegate_components! {
    App {
        GreeterComponent: GreetHello,
        FarewellComponent: SayGoodbye,
        AnnouncerComponent: AnnounceLoudly,
    }
}

delegate_components! {
    TestApp {
        GreeterComponent: GreetQuietly,
        FarewellComponent: SayGoodbye,
        AnnouncerComponent: AnnounceLoudly,
    }
}
```

Here the greeter changes, while the farewell and announcer entries must stay in agreement.
A larger shared table would leave more entries to keep synchronized. Giving those entries a
namespace lets both contexts inherit the common choices and state their differences separately.

## Lookups follow a path

For a context to make its own choice, the namespace needs to route a component lookup without
binding its final provider. Registering the greeter under an application path makes that distinction
visible:

```rust
cgp_namespace! { new AppNamespace {} }

#[cgp_component(Greeter)]
#[prefix(@app.GreeterComponent in AppNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}
```

`AppNamespace` routes requests for the greeter through `@app.GreeterComponent`. A context joins the
namespace and supplies the provider at that path:

```rust
delegate_components! {
    App {
        namespace AppNamespace;

        @app.GreeterComponent: GreetHello,
    }
}
```

The context entry completes the route declared by `AppNamespace`, and Rust resolves both parts
at compile time. This works because the destination was left unbound. Had the namespace already
bound it, adding a local entry would create an overlap; joining a namespace gives local entries
no priority over inherited ones.

The route can also cover a group of components. Redirecting a shared prefix sends the subsystem's
lookups to another prefix, while the rest of each path still identifies the component and its
parameters. The [`cgp_namespace!` reference](/docs/reference/macros/cgp_namespace) gives the syntax
for these grouped redirections.

## Bind shared choices and leave varying paths open

A child namespace can inherit routes and bind the choices its contexts share. Assume `Farewell`
is registered in `AppNamespace` at `@app.FarewellComponent`, just as the greeter is registered above:

```rust
cgp_namespace! {
    new AppDefaults: AppNamespace {
        @app.FarewellComponent: SayGoodbye,
    }
}
```

`AppDefaults` supplies the farewell provider and leaves the greeter path unbound. Contexts joining
it choose only their greeter:

```rust
delegate_components! { App     { namespace AppDefaults; @app.GreeterComponent: GreetHello } }
delegate_components! { TestApp { namespace AppDefaults; @app.GreeterComponent: GreetQuietly } }
```

The farewell choice now comes from `AppDefaults`, and only the greeter differs in the two context
tables. Because the shared configuration refers to components rather than particular application
types, a library can publish it for contexts that downstream applications define later.

## A bound entry cannot be overridden

A context or child namespace cannot rebind a key supplied by its parent namespace. The following
separate example deliberately binds the greeter directly, then tries to replace that binding:

```rust
cgp_namespace! {
    new AppDefaults {
        GreeterComponent: GreetHello,
    }
}

// error[E0119]: conflicting implementations of trait
//               `DelegateComponent<GreeterComponent>` for type `TestApp`
delegate_components! {
    TestApp {
        namespace AppDefaults;

        GreeterComponent: GreetQuietly,
    }
}
```

Joining this namespace already generates a forwarding implementation for `GreeterComponent`.
The local entry tries to implement the same lookup again, so Rust rejects the overlap. A child
namespace that redefines an inherited key runs into the same conflict.

Keep configurable paths unbound in the shared base. Returning to the path-based example, each
configuration can bind the open greeter path in its own child namespace:

```rust
cgp_namespace! { new ProductionDefaults: AppDefaults { @app.GreeterComponent: GreetHello  } }
cgp_namespace! { new TestDefaults:       AppDefaults { @app.GreeterComponent: GreetQuietly } }
```

Both children inherit the farewell choice, while each fills the greeter path that `AppDefaults`
left open. A context can then join the production or test configuration as appropriate. The shared
base remains consistent because each child adds a binding where the parent supplied only a route.

## Presets and per-type dispatch

This inherit-and-complete arrangement also gives namespaces their role as presets. A library can
bind a shared configuration while leaving paths open for the choices applications make. Child
namespaces fill those paths to produce more specific configurations, subject to the same rule
about inherited bindings.

The `open` statement uses related path routing for a smaller task: selecting a provider for each
type parameter of one component. This standalone fragment assumes an encoder component without a
namespace prefix:

```rust
delegate_components! {
    App {
        open EncoderComponent;

        @EncoderComponent.String: EncodeAsText,
    }
}
```

Here `open` starts the lookup path at `EncoderComponent`, and the following entry chooses the
provider for `String`. This uses path routing without requiring a shared namespace. If the
component has a namespace prefix, however, its lookups already follow that prefixed route. Entries
must use the full path, because opening its bare name does not reach them.

## What it costs

Binding a shared key commits every joining context to that choice. If a later context needs a
different provider, the namespace design must expose an unbound destination for it. Changing that
design can affect existing contexts, so decide which choices vary before publishing the shared table.

Inheritance also spreads the explanation of a lookup across tables. With an
[aggregate provider](./aggregate-providers.md), the context lists the components it delegates to the
bundle. With a namespace, registration and the parent chain can supply those routes, so a reader
may need to follow several tables to find the selected provider.

Tracing a failure can involve those paths too, making diagnostics longer. Joining the namespace
also leaves inherited dependencies unchecked until they are required. Separate
[`check_components!`](/docs/reference/macros/check_components) assertions bring the components a
concrete context needs into an explicit check.

Small tables often remain clearer when written directly. Namespaces become useful when sharing a
configuration or grouping routes reduces enough repetition to justify the inheritance and path
structure. An [aggregate provider](/docs/reference/glossary#aggregate-provider) offers a simpler way to share a named group of implementations.

## Where to go next

These pages cover simpler grouping and the namespace constructs:

- [Aggregate providers](./aggregate-providers.md): Reusing selected groups of providers.
- [Bypassing coherence](./coherence.md) and [Dispatching](./dispatching.md): Provider selection
  and per-type dispatch.
- [`cgp_namespace!`](/docs/reference/macros/cgp_namespace): Namespace definitions and inheritance.
- [`#[prefix]`](/docs/reference/attributes/prefix): Registering component routes.
- [`delegate_components!`](/docs/reference/macros/delegate_components): Joining a namespace and
  using `open`.
- [`RedirectLookup`](/docs/reference/providers/redirect_lookup) and
  [`Path!`](/docs/reference/macros/path): The routing provider and path types.
- [Comparison: Dynamic dispatch](/docs/comparisons/dynamic-dispatch): Shared defaults and the
  differences between namespace inheritance and runtime prototype lookup.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
