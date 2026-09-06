---
sidebar_label: '#[default_impl]'
sidebar_position: 9
---

# `#[default_impl(...)]`

Register a provider as a namespace's default for a key.

## Overview

A [namespace](/docs/concepts/namespaces) is a reusable table of default wirings that a **context**, the
type the capability runs against, can opt into and then selectively override. Ordinarily you write a
namespace's entries in its own body. `#[default_impl(...)]` lets a *provider* register itself instead,
at the point where it is defined:

```rust
#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> { /* … */ }
```

Read that as: *`ShowString` is the default for `String`, in the `DefaultImpls1<ShowImplComponent>`
table.* A context that pulls that table in gets this provider without naming it.

Registering at the definition keeps the provider and its default together, so a new per-type
implementation is one edit rather than two.

## Usage

The attribute goes on a [`#[cgp_impl]`](../macros/cgp_impl.md) provider and takes one argument in two
parts, joined by the keyword `in`:

```rust
#[default_impl(Key in NamespacePath)]
```

**`Key` becomes the emitted impl's `Self`**, and `NamespacePath` names the lookup trait plus whatever
leading arguments you write inside it. The macro appends the table parameter for you. So the example above
emits:

```rust
impl<Components> DefaultImpls1<ShowImplComponent, Components> for String {
    type Delegate = ShowString;
}
```

That rule is worth remembering, because the trait's own parameter names suggest the opposite
arrangement (see [`DefaultImpls1`](../traits/namespace/default_impls1.md#the-one-thing-to-get-right)).

**The key is a type or a `@`-path.** A type key, as above, is the usual form for a per-type default:
the type is the value of the component's dispatch parameter, and the component is named inside the
lookup trait's arguments. A path key, in [`Path!`](../macros/path.md)'s syntax, binds the provider at a
full path inside the namespace. Its common use is a prefixed component's own path, so that a context
which joins the namespace resolves the component with no `for` loop at all:

```rust
#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
#[default_impl(@app.GreeterComponent in AppNamespace)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello!".to_owned()
    }
}
```

Unlike [`#[prefix]`](./prefix.md), the macro appends nothing to a path key. You write the whole path,
marker included, and any dispatch type after it, as in `@test.ShowImplComponent.u32`. A path key
cannot declare generic parameters, and it takes neither of the `[…]` and `{…}` grouping forms a
[`delegate_components!`](../macros/delegate_components.md) key allows.

**Repeat the attribute to register into several tables**, one attribute per table. The argument
itself is not comma-separated: each attribute carries exactly one `Key in NamespacePath` pair.

**The path may name any trait**, not only the three CGP ships. A trait of your own with the right shape
works identically, which is why [`DefaultImpls2`](../traits/namespace/default_impls2.md) needed no new construct to
be usable.

The three built-in targets are [`DefaultNamespace`](../traits/namespace/default_namespace.md) for a component-only
key, [`DefaultImpls1`](../traits/namespace/default_impls1.md) for a per-type default (the usual choice), and
[`DefaultImpls2`](../traits/namespace/default_impls2.md) for a two-type key.

## Examples

The whole chain, from a provider declaring itself a default to a context pulling it in:

```rust
use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;
use core::fmt::Display;

#[cgp_component(ShowImpl)]
#[prefix(@test in DefaultNamespace)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}
```

Then the context:

```rust
pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        for <T, Provider> in DefaultImpls1<ShowImplComponent> {
            @test.ShowImplComponent.T: Provider,
        }

        @test.ShowImplComponent.u64: ShowWithDisplay,   // overrides the inherited default
    }
}
```

**Environmental context, parameter-targeted**: `App` carries the wiring and the shown value is a
parameter. The loop wires every type with a registered default, and the direct `u64` line shadows
whatever the namespace would otherwise supply for that one type.

A path key does not need a loop. With `CanGreet` registered under `@app` and `GreetHello` bound at
`@app.GreeterComponent` as in [Usage](#usage), a context resolves the component by joining the
namespace alone:

```rust
cgp_namespace! {
    new AppNamespace: DefaultNamespace {}
}

pub struct App;

delegate_components! {
    App {
        namespace AppNamespace;
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}
```

The prefix routes `GreeterComponent` to `@app.GreeterComponent`, and the registration answers that path
with `GreetHello`.

## When to use it

**Reach for it when a provider is the natural default for its key and you want that recorded where the
provider is written.** The alternatives differ in where the entry lives.

- **Use it** when per-type defaults accumulate (a conversion, a formatter, a codec with one provider per
  type), because then the alternative is a namespace body that has to be edited every time a type is
  added.
- **Write the entry in the namespace body instead** when the defaults belong together as a set, or when
  the provider is one of several candidates and none is obviously *the* default. A
  [`cgp_namespace!`](../macros/cgp_namespace.md) body reads as a table; scattered attributes do not.
- **Do not reach for a namespace at all** until the top-level wiring is long enough to be a problem.

**One constraint decides where the attribute may be written, and it is Rust's orphan rule rather than
anything CGP chose.** The emitted impl is `impl Namespace<..> for Key`, and Rust accepts it when the
crate owns the namespace trait, or when a local type appears in the impl header ahead of the table
parameter. That local type may be the key itself, or a component named inside the namespace path, so a
crate that owns `ShowImplComponent` may write `String in DefaultImpls1<ShowImplComponent>` against the
foreign `DefaultImpls1`. A path key is a `PathCons` list, which is never a local type even when it
contains a local marker, so a [`#[prefix]`](./prefix.md)-ed component's path can be registered **only
in the namespace's own crate**. Wiring that must live downstream goes in the namespace body of the crate
that owns it, or into a local namespace that inherits the foreign one.

## Under the hood

The attribute emits a single impl of the named lookup trait, for the key type:

```rust
impl<Components> DefaultImpls1<ShowImplComponent, Components> for String {
    type Delegate = ShowString;
}
```

The `Components` parameter is the table the lookup runs against, appended by the macro and left generic
so one registration serves every context. A path key emits the same impl for the path type, a
[`PathCons`](../types/path_cons.md) list that `cargo cgp expand` prints as `Path!(@app.GreeterComponent)`:

```rust
impl<Components> AppNamespace<Components> for Path!(@app.GreeterComponent) {
    type Delegate = GreetHello;
}
```

A context that joins `AppNamespace` resolves such a key without a loop: the join forwards every lookup
through the namespace, the component's prefix redirects `GreeterComponent` to the path, and this impl
answers it.

**The registration impl carries only the parameters naming the key and the provider, plus the table,
never the provider's own `where` clause.** That is deliberate, and it lets the attribute work with
ordinary providers: a provider whose bounds come from [`#[use_type]`](./use_type.md),
[`#[uses]`](./uses.md), [`#[implicit]`](./implicit.md), or [`#[use_provider]`](./use_provider.md)
registers cleanly, because those bounds stay on the provider's impl and its
[`IsProviderFor`](../traits/wiring/is_provider_for.md) and are checked when a real context resolves it.

Consumption runs the other way. A `for <T, Provider> in DefaultImpls1<Component> { … }` loop inside
[`delegate_components!`](../macros/delegate_components.md) emits a
[`DelegateComponent`](../traits/wiring/delegate_component.md) impl whose `where` clause projects the default:

```rust
where T: DefaultImpls1<Component, App, Delegate = Provider>
```

Because the loop variables appear only in that bound and in the key, **the key must mention them**, or
the parameter is unconstrained and the compiler rejects the impl with `E0207`.

## Formal grammar

The attribute argument is a key type, the keyword `in`, and a namespace path, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
DefaultImplArgs -> Key `in` NamespacePath

Key             -> Type | Path

Path            -> `@` PathSegment ( `.` PathSegment )*
PathSegment     -> Type

NamespacePath   -> TypePath GenericArgs?
```

`Key` is either a Rust `Type` or a `Path`, and the leading `@` tells them apart. A `Type` becomes the
emitted impl's `Self`. A `Path` is [`Path!`](../macros/path.md)'s own production and lowers to a
`PathCons` list in the same position. `NamespacePath` is the lookup trait with its leading generic
arguments written out. The table parameter is appended by the macro and must not be given. Each attribute takes
exactly one such argument, and the attribute may be repeated.

## Common Mistakes

**`Key` becomes `Self`, not a parameter.** Read `#[default_impl(Key in Path)]` as "`Key` becomes `Self`"
and the trait's positions follow. The parameter names on
[`DefaultImpls1`](../traits/namespace/default_impls1.md) suggest otherwise.

**Do not write the table parameter.** The macro appends it, so supplying it yourself makes the path's
arity wrong.

**On a prefixed component it is confined to the namespace's crate**, by the orphan rule. This is not
something to work around: put the wiring in the namespace body instead.

**The lookup trait must be imported.** The emitted impl names it, so
[`DefaultImpls1`](../traits/namespace/default_impls1.md) and [`DefaultImpls2`](../traits/namespace/default_impls2.md) need
`use cgp::core::component::…`; [`DefaultNamespace`](../traits/namespace/default_namespace.md) is in the prelude.

**A provider whose impl is generic cannot register a default.** The registration impl copies the
provider impl's generic parameters but drops its `where` clause, so a parameter of the impl appears
nowhere in `impl DefaultImpls1<ShowImplComponent, Components> for String`:

```text
error[E0207]: the type parameter `T` is not constrained by the impl trait, self type, or predicates
   |
   | impl<T: Display> ShowImpl<T> {
   |      ^ unconstrained type parameter
```

This holds even when the provider struct is not generic. Write per-type defaults for concrete impls,
and wire a generic provider in a namespace body or directly on the context instead.

**Two registrations for one key conflict.** Each emits an impl of the lookup trait for the same key,
and the compiler rejects the second, with both carets on the keys inside the attributes:

```text
error[E0119]: conflicting implementations of trait `DefaultImpls1<ShowImplComponent, _>` for type `String`
   |
   | #[default_impl(String in DefaultImpls1<ShowImplComponent>)]
   |                ------ first implementation here
...
   | #[default_impl(String in DefaultImpls1<ShowImplComponent>)]
   |                ^^^^^^ conflicting implementation for `String`
```

**A registered default is a fallback, not an assignment.** A context's direct entry silently shadows it.

**On the `#[cgp_impl(Self)]` form the registration is emitted with `Delegate = Self`.** That form does
not build a provider, so the macro fills the delegate with `Self`, which inside the registration impl is
the key itself. The attribute cannot be used sensibly there; leave it off.

## Related constructs

- [`DefaultImpls1`](../traits/namespace/default_impls1.md) — the usual target, and where the positional rule is
  worked out.
- [`DefaultNamespace`](../traits/namespace/default_namespace.md) and
  [`DefaultImpls2`](../traits/namespace/default_impls2.md) — the component-only and two-type targets.
- [`cgp_namespace!`](../macros/cgp_namespace.md) — defines a namespace, and the path-key form's usual
  target.
- [`#[prefix(...)]`](./prefix.md) — the component-side registration this attribute pairs with: it
  routes, where this attribute binds.
- [`delegate_components!`](../macros/delegate_components.md) — carries the `namespace` header and the
  `for … in` loop that consume a registration.
- [`#[cgp_impl]`](../macros/cgp_impl.md) — the host this attribute goes on.
- [`IsProviderFor`](../traits/wiring/is_provider_for.md) — where a registered provider's real bounds are
  checked.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the usual `Delegate` value.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables and preset-style
  configuration.

## Source

- The attribute: [`attributes/default_impl/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/attributes/default_impl)
- The lookup traits: [`namespaces.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/namespaces.rs)
- Header and loop codegen: [`delegate_component/statement/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/statement)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
