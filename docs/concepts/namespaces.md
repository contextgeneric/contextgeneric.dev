---
sidebar_label: 'Namespaces'
sidebar_position: 9
---

# Namespaces

Namespaces share wiring across contexts while leaving selected provider choices to each context.
They are useful when related contexts repeat large parts of the same table. This page explains
path-based routing, inherited bindings, and the rule that determines what can vary: a context can
fill an unbound path, but it cannot replace an inherited binding.

## Repeated wiring obscures the differences

Separate wiring tables can repeat choices that are intended to stay together. These fragments
assume the named components, providers, and context types are defined, with CGP's prelude imported:

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

Only the greeter differs, but each table repeats the shared farewell and announcer choices.
As the shared portion grows, maintaining the tables requires checking that those entries still
agree. A namespace gives the shared wiring a name that contexts can inherit.

## Lookups follow a path

A namespace can register a route for a component while leaving the provider at its destination
unspecified. This example registers the greeter under an application path:

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

The namespace supplies the route, and `App` supplies its destination binding. Rust resolves both
at compile time. The context's entry must not overlap an entry the namespace already binds;
joining a namespace does not give local entries priority over inherited ones.

Paths also let wiring redirect a group of related lookups. A shared prefix can route a subsystem
to another prefix, with the remaining path identifying its component and parameters. The
[`cgp_namespace!` reference](/docs/reference/macros/cgp_namespace) specifies that redirection syntax.

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

Both contexts inherit `SayGoodbye`, and each states its own greeting choice. A library can publish
this shared configuration for applications to use without knowing those applications' context types.

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

Rust rejects the overlapping implementations. Joining the namespace generates a forwarding
implementation for its keys, and the direct `GreeterComponent` entry would implement the same
lookup again. A child namespace redefining an inherited key has the same problem.

Keep configurable paths unbound in the shared base. Returning to the path-based example, each
configuration can bind the open greeter path in its own child namespace:

```rust
cgp_namespace! { new ProductionDefaults: AppDefaults { @app.GreeterComponent: GreetHello  } }
cgp_namespace! { new TestDefaults:       AppDefaults { @app.GreeterComponent: GreetQuietly } }
```

`ProductionDefaults` and `TestDefaults` both inherit the shared farewell choice and supply different
greeters. A context then joins the configuration it needs. Neither child replaces a parent binding.

## Presets and per-type dispatch

A namespace can serve as a preset by grouping shared choices and exposing unbound paths for
configuration. Inheritance adds the remaining choices. The distinction between a fixed binding
and an open path determines which parts an application can customize.

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

`open` roots the route at `EncoderComponent`, and the context supplies entries beneath that root.
It does not require a shared namespace. Once a component is registered through a namespace prefix,
its entries must use that full prefixed route instead; opening the same component at its bare name
does not reach them.

## What it costs

Binding a shared key commits every joining context to that choice. If a later context needs a
different provider, the namespace design must expose an unbound destination for it. Changing that
design can affect existing contexts, so decide which choices vary before publishing the shared table.

Inherited wiring takes more work to trace. An [aggregate provider](./aggregate-providers.md)
lets the context list which components it delegates to a bundle. A namespace can supply those
routes through its registration and parent chain, so finding the selected provider may require
reading several tables.

Path types can make diagnostics longer, and joining a namespace does not verify every inherited
component's dependencies. Use separate [`check_components!`](/docs/reference/macros/check_components)
assertions for the components a concrete context needs.

Small tables often remain clearer when written directly. Namespaces become useful when sharing a
configuration or grouping routes reduces enough repetition to justify the inheritance and path
structure. An aggregate provider offers a simpler way to share a named group of implementations.

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
