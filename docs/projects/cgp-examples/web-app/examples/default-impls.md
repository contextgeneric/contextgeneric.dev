---
sidebar_label: 'default_impls'
sidebar_position: 4
description: 'A CGP namespace that supplies seven of an application''s nine providers as defaults, so the context wires only its filters, and why a default is final.'
---

# Supply an application's defaults from a namespace

This stage moves seven of the backend's nine providers into a namespace as defaults, so the
application's context joins the namespace and wires only what varies: its content filters. It is the
last stage of [`web-app`](../index.md), a wiring study from the [cgp-examples](../../index.md)
repository, built with [CGP](/docs/). Nothing in the crate runs; every provider body is `todo!()`,
and the compiler's check of the wiring is what this page runs.

:::tip

### New to CGP?

[`namespace`](./namespaces.md) introduces the paths this stage builds on. For CGP itself, the [Hello
World tutorial](/docs/tutorials/hello) introduces wiring, and
[Namespaces](/docs/concepts/namespaces) explains inheriting a namespace and binding its paths.

:::

## The problem

The task is to supply the backend's usual choices as defaults, so that an application built on it
wires only what it varies, here its content filters, and gets the storage and the checks without
naming them. This is what a library wants to offer its users: a working configuration, open at the
points where applications differ.

### Without CGP

In plain Rust, defaults are usually supplied at run time: a builder or a constructor with default
values and optional overrides, or a `Default` implementation for a configuration struct. That is
flexible, and it is often the right answer. What it does not give is a compile-time check that a
configuration is complete, since the configuration is assembled from values when the program runs.
Trait default methods are compile-time, but they give each trait one default for every implementer,
not a set of defaults an application opts into. This stage publishes the defaults as a namespace an
application joins.

## Check it

From the root of the [cgp-examples repository](https://github.com/contextgeneric/cgp-examples):

```sh
cargo check -p cgp-example-web-app
```

The check passes. For this stage, that means `ProductionApp`, wired with one entry of its own,
satisfies all nine components.

## A namespace of defaults

The module,
[`default_impls.rs`](https://github.com/contextgeneric/cgp-examples/blob/main/web-app/src/default_impls.rs),
keeps the previous stage's components and paths, and declares a namespace of its own that inherits
`DefaultNamespace`:

```rust
cgp_namespace! {
    new DefaultAppComponents: DefaultNamespace
}
```

[`cgp_namespace!`](/docs/reference/macros/cgp_namespace) declares `DefaultAppComponents`, and the
`: DefaultNamespace` makes it a child that inherits every path the components registered. So far it
routes lookups and binds nothing. The rest of the module binds providers at its paths, in two ways.

## Defaults registered on the providers

Five providers register themselves as the namespace's default for their component, with
[`#[default_impl]`](/docs/reference/attributes/default_impl):

```rust
#[cgp_impl(new GetUserWithPostgres)]
#[default_impl(@app.core.user.UserGetterComponent in DefaultAppComponents)]
impl UserGetter {
    fn get_user(&self, #[implicit] database: &PostgresDb, user_id: &UserId) -> Result<User, Error> {
        todo!()
    }
}
```

The attribute names the full path, the component's prefix followed by its name, and the namespace to
bind it in. `UpdateUserWithPostgres`, `GetPostWithPostgres`, `UpdatePostWithPostgres`, and
`DeletePostWithPostgres` do the same for their components.

## Defaults written in a namespace block

The two creators cannot register this way, because their default is not a provider on its own but a
filter wrapped around one. So a second block binds them in the namespace directly:

```rust
cgp_namespace! {
    DefaultAppComponents {
        @app.core.user.UserCreatorComponent:
            FilterCensoredUsername<CreateUserWithPostgres>,

        @app.core.post.PostCreatorComponent:
            FilterSpamMessage<CreatePostWithPostgres>,
    }
}
```

With these, the namespace binds every path under `@app.core` and nothing under `@app.extra`. The
content filters are left for each context to choose.

## The context wires what is left

`ProductionApp` joins the namespace and wires the content filters, the one group the namespace
leaves open:

```rust
delegate_components! {
    new ContentFilterComponents {
        UsernameCensorComponent:
            AiUserCensor,
        SpamMessageDetectorComponent:
            AiSpamMessageDetector,
    }
}

delegate_components! {
    ProductionApp {
        namespace DefaultAppComponents;

        @app.extra.content_filter: ContentFilterComponents,
    }
}
```

`ProductionApp` is a type that stands for the application, with the database as its one field, and
this is the whole of its wiring. A check of all nine components passes on it. The filter entry is
still needed: the default creators wrap their filters, so without it the creators fail their checks
for want of a censor and a spam detector.

## Try a change

Try to replace one default. Add a provider of your own for the user getter at the end of the module:

```rust
#[cgp_impl(new GetCachedUser)]
impl UserGetter {
    fn get_user(&self, user_id: &UserId) -> Result<User, Error> {
        todo!()
    }
}
```

and wire it in `ProductionApp`, beside the namespace line:

```rust
@app.core.user.UserGetterComponent: GetCachedUser,
```

Run [`cargo cgp check`](/docs/cargo-cgp/check), and the entry is rejected:

```text
error[E0119]: [CGP-E005] `ProductionApp` cannot wire `@app.core.user.UserGetterComponent.*` that is already set through `DefaultAppComponents`
```

A default a namespace binds is final. The context's entry would be a second answer to the same
lookup, and Rust rejects the two as conflicting implementations, which is `E0119` underneath the
tool's message. A context that needs a different getter joins a different namespace, one that does
not bind the getter. `cargo cgp check` leads with the root cause for the classes it recognizes, and
the tool does not yet reshape every class.

## The pattern

This stage shows **defaults supplied by a namespace**: a library, or a part of an application,
publishes a namespace that binds the choices every context shares and leaves open the ones each
context makes, so a context is only its differences. [Namespaces](/docs/concepts/namespaces)
explains binding shared choices and leaving varying paths open.

The cost is the finality the change above shows. A context that joins the namespace takes every
default it binds, with no way to replace one, so the namespace has to leave open everything a
context might want to vary. Deciding which choices are shared is a design decision made once, for
every context that joins; where it is wrong, the fix is a different namespace, not a local entry.

## Where to go next

- [web-app overview](../index.md): the four stages side by side, from one trait per domain to here.
- [`cgp_namespace!`](/docs/reference/macros/cgp_namespace): the macro, including inheritance and
  bound entries.
- [Namespaces](/docs/concepts/namespaces): the CGP idea behind shared defaults, and why a bound
  entry cannot be overridden.
- [CGP v0.8.0](/blog/v0.8.0-release#default-implementations): the release post that develops this
  application stage by stage.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
