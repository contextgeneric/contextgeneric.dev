---
sidebar_label: 'namespace'
sidebar_position: 3
description: 'A CGP application whose components sit under paths in a namespace, so a production and a test context each wire it in two lines and differ in one.'
---

# Group an application's wiring under paths

This stage places the backend's nine components under paths, so that a context wires the whole
application in two lines, and a test context differs from production in one. It is the third stage
of [`web-app`](../index.md), a wiring study from the [cgp-examples](../../index.md) repository,
built with [CGP](/docs/). Nothing in the crate runs; every provider body is `todo!()`, and the
compiler's check of the wiring is what this page runs.

:::tip

### New to CGP?

[`fine_grained`](./fine-grained.md) introduces the nine components and the bundles this stage
reuses. For CGP itself, the [Hello World tutorial](/docs/tutorials/hello) introduces wiring, and
[Namespaces](/docs/concepts/namespaces) explains the paths and lookups this stage relies on.

:::

## Check it

From the root of the [cgp-examples repository](https://github.com/contextgeneric/cgp-examples):

```sh
cargo check -p cgp-example-web-app
```

The check passes. For this stage, that means both `ProductionApp` and `TestApp` satisfy all nine
components.

## Each component gets a path

The module,
[`namespace.rs`](https://github.com/contextgeneric/cgp-examples/blob/main/web-app/src/namespace.rs),
declares the same nine components as the previous stage, each with a
[`#[prefix]`](/docs/reference/attributes/prefix) that registers it under a path:

```rust
#[cgp_component(UserCreator)]
#[prefix(@app.core.user in DefaultNamespace)]
pub trait CanCreateUser {
    fn create_user(&self, username: &str, email: &Email) -> Result<User, Error>;
}
```

`DefaultNamespace` is the namespace CGP provides for components to register in, and a
[namespace](/docs/concepts/namespaces) is a wiring table that turns a lookup for a component into a
lookup for a path. The nine components fill three paths under two parents:

| Path | Components |
|---|---|
| `@app.core.user` | `UserCreator`, `UserGetter`, `UserUpdater` |
| `@app.core.post` | `PostCreator`, `PostGetter`, `PostUpdater`, `PostDeleter` |
| `@app.extra.content_filter` | `UsernameCensor`, `SpamMessageDetector` |

So a context that joins `DefaultNamespace` and is asked for `UserGetterComponent` looks for an entry
at `@app.core.user`, and one entry there can answer for all three user components.

## Bundles are grouped by path

The providers and the three bundles of the previous stage return unchanged, along with a fourth
bundle, `DummyContentFilterComponents`, that wires placeholder filters for testing. On top of them
sit three bundles keyed by path:

```rust
delegate_components! {
    new PostgresCoreComponents {
        namespace DefaultNamespace;

        @app.core.user: PostgresUserComponents,
        @app.core.post: PostgresPostComponents,
    }
}

delegate_components! {
    new ProductionExtraComponents {
        namespace DefaultNamespace;

        @app.extra.content_filter: AiContentFilterComponents,
    }
}
```

`PostgresCoreComponents` routes the user and post paths to their bundles, and
`ProductionExtraComponents` routes the content filters to the AI bundle. `DummyExtraComponents` is
the same with the placeholder bundle. Each of these bundles joins `DefaultNamespace` itself, with
its `namespace` line, because a lookup arrives at a bundle by the component's name, and it is the
namespace that turns the name into the path the bundle's entries match. Without the line, the bundle
has no entry for the name.

## Two contexts, one line apart

The production and test contexts each join the namespace and route the two top-level paths:

```rust
delegate_components! {
    ProductionApp {
        namespace DefaultNamespace;

        @app.core: PostgresCoreComponents,
        @app.extra: ProductionExtraComponents,
    }
}

delegate_components! {
    TestApp {
        namespace DefaultNamespace;

        @app.core: PostgresCoreComponents,
        @app.extra: DummyExtraComponents,
    }
}
```

Each is a type that stands for one configuration of the application, with the database as its one
field. They share the whole core and differ only in their content filters, and the difference is
visible at a glance. One [`check_components!`](/docs/reference/macros/check_components) block
asserts all nine components on each.

## Try a change

Remove `TestApp`'s `@app.extra` line, and run [`cargo cgp check`](/docs/cargo-cgp/check). The two
creators fail, each with the filter its wrapper uses:

```text
error[E0277]: [CGP-E001] the consumer traits `CanCreateUser` and `CanCensorUsername` are not implemented for context `TestApp`
    = note: root cause: [CGP-E107] context `TestApp` does not contain any delegate entry for `@app.extra.content_filter.UsernameCensorComponent`
error[E0277]: [CGP-E001] the consumer traits `CanCreatePost` and `CanDetectSpamMessage` are not implemented for context `TestApp`
    = note: root cause: [CGP-E107] context `TestApp` does not contain any delegate entry for `@app.extra.content_filter.SpamMessageDetectorComponent`
```

Each root cause names the full path the namespace routed the lookup to, which says where the entry
belongs. The other five components still check, since the getters, updaters, and deleter need
nothing from the extras. `cargo cgp check` leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class.

## The pattern

This stage shows **components grouped under paths in a namespace**, so wiring routes a whole group
with one entry, and a hierarchy of paths lets a parent route its children through a bundle. The
contexts become short enough that their differences are what a reader sees.
[Namespaces](/docs/concepts/namespaces) explains the lookups, and [Aggregate
providers](/docs/concepts/aggregate-providers) explains bundles behind paths.

The cost is indirection. To find what `TestApp` uses for `UserGetterComponent`, a reader follows the
component's prefix to a path, the path to `PostgresCoreComponents`, that bundle to
`PostgresUserComponents`, and that bundle to `GetUserWithPostgres`. The compiler does this
instantly, and `cargo cgp check` prints the chain when something is missing, but a person reads four
tables where the coarse stage had one line. And every context still names the core bundle; the [next
stage](./default-impls.md) moves that into the namespace too.

## Where to go next

- [`default_impls`](./default-impls.md): the next stage, a namespace that supplies the defaults.
- [web-app overview](../index.md): the four stages side by side.
- [Namespaces](/docs/concepts/namespaces): the CGP idea behind paths and the lookups they route.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
