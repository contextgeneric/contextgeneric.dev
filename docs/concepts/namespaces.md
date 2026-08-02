---
sidebar_label: 'Namespaces'
sidebar_position: 9
---

# Namespaces

Reusable, inheritable wiring tables that keep a context's own table short as the number of
components grows.

This page answers *how does wiring stay readable once there is a lot of it?* It shows what a table
looks like when it has outgrown its usefulness, the routing that fixes it, and the one rule that
decides how a namespace has to be designed — a rule most people discover from a compiler error. It
closes on when the lighter alternative is the better buy.

## When a table stops being readable

Wiring is meant to be the one place a context's choices are visible. That works while the table is
short, and stops working when two contexts want almost the same table:

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

Two entries out of three are duplicated, and nothing records that they are meant to stay the same.
Scale that to the thirty components a real application accumulates and the interesting difference —
one line — is buried in twenty-nine identical ones, which is the opposite of what the table was for.

A **namespace** is a table lifted out of any one context and given a name, so that contexts can share
it.

## Lookups follow a path

The mechanism that makes sharing work is that a namespace resolves **paths** rather than bare
component names. A component registers itself at a path, and the path is a *route* rather than an
answer:

```rust
cgp_namespace! { new AppNamespace {} }

#[cgp_component(Greeter)]
#[prefix(@app.GreeterComponent in AppNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}
```

`AppNamespace` now knows that anything asking for this capability should look under
`@app.GreeterComponent`. It does not know what it will find there. A context joins the namespace and
supplies the answer at that path:

```rust
delegate_components! {
    App {
        namespace AppNamespace;

        @app.GreeterComponent: GreetHello,
    }
}
```

The `namespace` line makes everything `App` does not wire itself fall through to `AppNamespace`.
Routing through paths rather than flat keys is also what lets a whole group be redirected at once and
what makes inheritance possible, since a path has structure a name does not.

On its own this is more machinery for the same result. What it buys arrives next.

## Binding what is shared, leaving open what varies

A namespace can also **bind** a path — supply the provider itself, rather than routing to it. That is
what turns it into a set of defaults:

```rust
cgp_namespace! {
    new AppDefaults: AppNamespace {
        @app.FarewellComponent: SayGoodbye,
    }
}
```

`AppDefaults` inherits the routing from `AppNamespace` and answers the farewell path itself. A context
joining it supplies only what is left:

```rust
delegate_components! { App     { namespace AppDefaults; @app.GreeterComponent: GreetHello } }
delegate_components! { TestApp { namespace AppDefaults; @app.GreeterComponent: GreetQuietly } }
```

The two contexts now differ by exactly the thing that differs between them. Everything they share is
stated once, in a place that can be published by a library and adopted by applications that library has
never heard of.

## A bound entry cannot be overridden

Here is the rule, and it is worth learning from this page rather than from the error:
**once a namespace binds a key, no one downstream can rebind it. Only a path the namespace leaves open
is available to a context.**

The natural thing to reach for — "the namespace sets this, I want something different" — does not
compile:

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

The same rejection catches a child namespace redefining a key it inherits. The reason is coherence
rather than policy: joining a namespace generates an implementation covering *every* key the namespace
answers, and a specific entry for one of those keys overlaps it. The compiler cannot see that you meant
the specific one to win.

So a namespace is designed around the question *what varies?*, and this is the practical shape:
**bind what every context agrees on, and leave open what any context might need to differ on.** Where a
key varies, do not bind it in the shared namespace at all — inherit and bind it per configuration
instead:

```rust
cgp_namespace! { new ProductionDefaults: AppDefaults { @app.GreeterComponent: GreetHello  } }
cgp_namespace! { new TestDefaults:       AppDefaults { @app.GreeterComponent: GreetQuietly } }
```

Each child binds the open path rather than overriding a bound one, and a context becomes a single line
naming which configuration it is.

## Namespaces are how CGP does presets

There is no separate preset construct, and no `cgp_preset!` to look for. A preset — a curated bundle of
defaults you adopt and then adjust — is exactly the inherit-and-adjust behaviour above, so a namespace
*is* one.

The same machinery serves a much smaller case. Dispatching a single component per type, with the
`open` statement, is path routing applied to one component on one context:

```rust
delegate_components! {
    App {
        open EncoderComponent;

        @EncoderComponent.String: EncodeAsText,
    }
}
```

`open` roots a route at the bare component name and puts the per-type entries in the context's own
table, with no shared namespace involved. It is the lightweight end of the same mechanism, and it is
what most code uses — which is why [bypassing coherence](./coherence.md) and
[dispatching](./dispatching.md) can use it without mentioning namespaces at all.

The two do not combine for the same component: once a component is registered behind a namespace
prefix, `open` would root the route at the wrong place, and the full prefixed path is what reaches its
entries.

## What it costs

**The design decision comes first, and it is hard to revise.** Which keys a namespace binds and which
it leaves open is fixed by the rule above, and changing your mind means changing the namespace — which
reaches every context that joined it. A bundle of wiring is easier to get wrong here than anywhere else
in CGP, because the mistake is not visible until a context wants to differ.

**It is another hop, and a less obvious one.** With an
[aggregate provider](./aggregate-providers.md) the context says which components come from the bundle.
With a namespace it says nothing — everything not wired locally falls through — so answering "where
does this capability come from?" means knowing the namespace and its parents.

**Paths are a second vocabulary.** `@app.GreeterComponent` is a type-level path, and it appears in
error messages spelled out at length. The toolchain resugars it for the classes it recognizes; the raw
form is still what a plain `cargo check` shows.

**And it pays only at scale.** For three components shared by two contexts, the duplication at the top
of this page is fine and a namespace is not worth its cost. The threshold is a table that has outgrown
reading, or a library publishing defaults for applications to adopt.

## Where to go next

[Aggregate providers](./aggregate-providers.md) is the lighter way to package reusable wiring, and the
right one until a table is genuinely long. [Bypassing coherence](./coherence.md) and
[Dispatching](./dispatching.md) both use the `open` form of path routing without needing anything on
this page.

For the constructs, [`cgp_namespace!`](/docs/reference/macros/cgp_namespace) defines a namespace and
carries the `#[prefix(...)]` registration attribute,
[`delegate_components!`](/docs/reference/macros/delegate_components) carries the `namespace` and `open`
statements, [`RedirectLookup`](/docs/reference/providers/redirect_lookup) is the provider doing the
routing, and [`Path!`](/docs/reference/macros/path) is the path type underneath.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
