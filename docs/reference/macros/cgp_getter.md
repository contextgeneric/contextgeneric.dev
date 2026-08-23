---
sidebar_label: '#[cgp_getter]'
sidebar_position: 9
---

# `#[cgp_getter]`

Define a getter as a full component, so the field it reads is chosen by wiring rather than by its name.

## Overview

[`#[cgp_auto_getter]`](./cgp_auto_getter.md) ties a getter to a field of the same name: declare
`fn name(&self) -> &str` and every context with a `name` field satisfies it. That is the right trade almost
always, and it has one hard edge: a context that stores the value under a different name cannot use the
getter at all.

`#[cgp_getter]` removes that coupling by making the getter a real component. The **context** is the
type the capability runs against, and it supplies the values it needs as its own fields. It then says
in its wiring which field the getter should read:

```rust
delegate_components! {
    Person {
        NameGetterComponent: UseField<Symbol!("first_name")>,
    }
}
```

`Person::name()` now returns the `first_name` field. The method is still called `name`; the field it reads
has moved into the wiring, where a context can change it without touching the trait or any code that calls
it.

**This is an advanced tool, not the next step up from `#[cgp_auto_getter]`.** What you pay for the
decoupling is a line of wiring per context, and what you get is only useful when a context needs
to control which field is read, or to supply the value some way other than reading a field. Most getters
want neither. The [When to use it](#when-to-use-it) section draws the line.

## Usage

Apply the attribute to a getter trait, exactly as with [`#[cgp_auto_getter]`](./cgp_auto_getter.md). It
accepts the same method forms: every receiver shape, including a
[typed reference to another type](./cgp_auto_getter.md#reading-a-field-of-another-type) in place of
`self` and an [optional `PhantomData` argument](./cgp_auto_getter.md#an-optional-phantomdata-argument);
and the `&str`, `&[T]`, `Option<&T>`, `Option<&str>`, `MRef<'_, T>`, owned, and associated-type return
shorthands, with their [access rules](./cgp_auto_getter.md#how-the-return-type-decides-the-read)
unchanged. The two macros share one parser, so a method the one accepts the other does too:

```rust
#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}
```

Because `#[cgp_getter]` builds on [`#[cgp_component]`](./cgp_component.md), it needs a provider trait name
and derives one from the **trait** name: a leading `Has` is stripped and `Getter` appended, so `HasName`
yields the provider trait `NameGetter` and the marker `NameGetterComponent`. The macro is therefore at its
most ergonomic when getter traits follow the `Has{Field}` convention.

Pass an identifier to override it, as with `#[cgp_component]`:

```rust
#[cgp_getter(GetName)]
pub trait HasName {
    fn name(&self) -> &str;
}
```

Here the provider trait is `GetName` and the component `GetNameComponent`. The keyed
`name` / `provider` / `context` form works too; only the default for `provider` differs from
`#[cgp_component]`.

### What a context can wire it to

Three providers come out of the macro, and which you name decides where the value comes from.

| Wire it to | The getter reads |
|---|---|
| [`UseField<Symbol!("f")>`](../providers/use_field.md) | the field named `f`, whatever the method is called |
| `UseFields` | the field named after each method, the `#[cgp_auto_getter]` behaviour as a provider |
| [`WithProvider<P>`](../providers/with_provider.md) | whatever the field-getter provider `P` supplies |

`UseField` is the one to reach for, and the reason the construct exists. `UseFields` is useful when a
trait has several methods and the names all happen to match, and it is the only one of the three that
works for a multi-method trait: the other two presuppose a single field.

A `#[cgp_getter]` trait can also be implemented directly on a concrete context, like any Rust trait, when
a particular context wants neither wiring nor a field.

## Examples

The case `#[cgp_auto_getter]` cannot express: the trait method is `name`, and the context stores the value
in `first_name`.

```rust
use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Person {
    pub first_name: String,
}

delegate_components! {
    Person {
        NameGetterComponent: UseField<Symbol!("first_name")>,
    }
}

pub fn greet(person: &Person) {
    println!("Hello, {}!", person.name());
}
```

`person.name()` returns `first_name`, because the wiring said so. A second context can store it under yet
another name and wire accordingly, with `HasName` and every caller unchanged. This is the whole
difference from the blanket-impl getter.

Where the names *do* line up, `UseFields` gives the auto-getter behaviour without giving up the component:

```rust
delegate_components! {
    Employee {
        NameGetterComponent: UseFields,
    }
}
```

And a context that computes the value rather than storing it implements the consumer trait directly,
skipping both the wiring and the field:

```rust
pub struct Anonymous;

impl HasName for Anonymous {
    fn name(&self) -> &str {
        "anonymous"
    }
}
```

That last form is worth seeing, because it shows a `#[cgp_getter]` trait is an ordinary component whose
consumer trait can be implemented like any Rust trait.

## When to use it

**Do not reach for `#[cgp_getter]` by default.** Among the three ways to read a value from a context it is
the last resort, and the ordering is worth holding whole:

1. **An [`#[implicit]`](../attributes/implicit.md) argument** for a provider reading a field of its own
   context. No trait, no wiring. This covers most reads.
2. **[`#[cgp_auto_getter]`](./cgp_auto_getter.md)** when the accessor has to exist as a *named
   capability* other code depends on, when the field lives on another type, or when the getter carries a
   type inferred from the field. One blanket impl, still no wiring.
3. **`#[cgp_getter]`** only when a context must control *how the getter is satisfied*.

That third condition is narrow, and it has two real forms. One is a **field name that differs per
context**: the same capability reading `first_name` on one type and `display_name` on another. The other
is a context that supplies the value **some way other than a plain field read**, through
`WithProvider` or a hand-written impl, while other contexts still read a field.

Reach for something else in these cases.

- **Every context stores the field under the method's name.** Then the wiring line is pure ceremony:
  `#[cgp_auto_getter]` gives the same result with nothing to wire, and wiring `UseFields` here is a sign
  the component was not needed.
- **Only one provider reads the value.** An `#[implicit]` argument is shorter and keeps the requirement
  where the provider is written.
- **The value is a *type* rather than a value.** [`#[cgp_type]`](./cgp_type.md) is the abstract-type
  component, and `UseType` is its counterpart to this construct's `UseField`.
- **The trait has several methods and you want per-context field choice for each.** `UseField` handles one
  field, so a multi-method getter wired this way has to go through `UseFields` or a hand-written provider.
  Splitting into one component per field is usually the better answer.

## Under the hood

`#[cgp_getter]` emits everything [`#[cgp_component]`](./cgp_component.md) would: the consumer trait, the
provider trait, the two blanket impls, the marker, and the standard
[`UseContext`](../providers/use_context.md) and [`RedirectLookup`](../providers/redirect_lookup.md)
impls. It then adds the getter providers below. The macro always emits `UseFields`; it emits `UseField`
and `WithProvider` only when the trait has exactly one method, since both presuppose a single field.

The important addition is the **`UseField` impl**, which decouples the field from the method name.
From this trait:

```rust
#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}
```

the macro generates:

```rust
impl<__Context__, __Tag__> NameGetter<__Context__> for UseField<__Tag__>
where
    __Context__: HasField<__Tag__, Value = String>,
{
    fn name(__context__: &__Context__) -> &str {
        __context__.get_field(PhantomData::<__Tag__>).as_str()
    }
}
```

**`__Tag__` is a free generic parameter**, and that is the entire difference from
[`#[cgp_auto_getter]`](./cgp_auto_getter.md), whose blanket impl hard-codes the tag to
`Symbol!("name")`. Here the tag is whatever the wiring supplies, so `UseField<Symbol!("first_name")>`
reads `first_name`. The `&str` shorthand behaves identically in both: the bound asks for a `String` and the
body appends `.as_str()`.

Next is the `UseFields` impl, which is the auto-getter's behaviour expressed as a provider, with the
tag fixed to each method's own name:

```rust
impl<__Context__> NameGetter<__Context__> for UseFields
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(__context__: &__Context__) -> &str {
        __context__.get_field(PhantomData::<Symbol!("name")>).as_str()
    }
}
```

Last is a [`WithProvider`](../providers/with_provider.md) impl, which adapts a general field-getter
provider into this component, so the value need not come from a field read at all:

```rust
impl<__Context__, __Provider__> NameGetter<__Context__> for WithProvider<__Provider__>
where
    __Provider__: FieldGetter<__Context__, NameGetterComponent, Value = String>,
{
    fn name(__context__: &__Context__) -> &str {
        __Provider__::get_field(__context__, PhantomData::<NameGetterComponent>).as_str()
    }
}
```

Each of these is paired with a matching [`IsProviderFor`](../traits/wiring/is_provider_for.md) impl carrying the
same bounds, so a missing field is reported by name when the component is checked.

## Formal grammar

The attribute argument is the same grammar as [`#[cgp_component]`](./cgp_component.md)'s, in the Rust
Reference's [notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpGetterArgs -> CgpComponentArgs    // see #[cgp_component]
```

A bare provider name, or the keyed `name` / `provider` / `context` form. The only difference from
`#[cgp_component]` is the default applied when `provider` is omitted: rather than being required, it is
derived from the trait name by stripping a leading `Has` and appending `Getter`, so `HasName` yields
`NameGetter`. Every other key and default behaves as documented there.

## Common Mistakes

**A multi-method trait gets no `UseField` provider.** Both `UseField` and `WithProvider` are emitted only
for a single-method trait, because each supplies one field. Wiring a two-method getter to
`UseField<Symbol!("width")>` therefore reports the provider as not being a provider for the component,
with the tag expanded into its raw spine:

```text
error[E0277]: the trait bound `UseField<Symbol<5, Chars<'w', ...>>>: IsProviderFor<..., ...>`
              is not satisfied
```

Nothing in that says the trait had too many methods, which is the actual cause. Use `UseFields`, or split
the trait into one component per field.

**The macro derives the provider name by stripping `Has`.** `HasName` yields `NameGetterComponent`, so
a trait *not* named `Has…` produces a component whose name may surprise you: `Dimensions` yields
`DimensionsGetterComponent`. Pass the name explicitly when the convention does not fit.

**Wiring `UseFields` everywhere means the component was unnecessary.** If no context ever names a
different field, [`#[cgp_auto_getter]`](./cgp_auto_getter.md) does the same job with no wiring at all, and
the component is a line per context of pure overhead.

**The field's type still has to match the return type's rule.** `UseField<Symbol!("first_name")>` on a getter
returning `&str` requires `first_name` to be a `String`; a `&'static str` field does not qualify, because the
tag chose the field and not the conversion:

```text
error[E0271]: type mismatch resolving `<P as HasField<Symbol<10, Chars<'f', ...>>>>::Value == String`
```

The `Symbol<10, …>` in that message is `first_name` with its length in bytes, worth being able to read
since the tag tells you which field the mismatch is about.

## Related constructs

- [`#[cgp_auto_getter]`](./cgp_auto_getter.md) — the blanket-impl getter, and the one to prefer.
- [`#[implicit]`](../attributes/implicit.md) — the default way to read a field, ahead of either getter.
- [`UseField`](../providers/use_field.md) — the provider that makes the field name a wiring decision, with
  `UseFieldRef` and `UseFields` alongside it.
- [`#[cgp_component]`](./cgp_component.md) — the macro this extends, and the source of the naming rules.
- [`#[derive(HasField)]`](../derives/derive_has_field.md) and [`HasField`](../traits/field-access/has_field.md) — the
  field access underneath.
- [`Symbol!`](./symbol.md) — the type-level field name a wiring entry supplies.
- [`WithProvider`](../providers/with_provider.md) — the adapter for a value that does not come from a field.
- [`#[cgp_type]`](./cgp_type.md) — the same idea for a type rather than a value.

The ideas behind it:

- [Implicit arguments](/docs/concepts/implicit-arguments) — the default way to read a field, and why
  this is the advanced fallback.

## Source

- Entry point: [`cgp_getter.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/cgp_getter.rs)
- Implementation: [`types/cgp_getter/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_getter/)
- Getter parsing, shared with `#[cgp_auto_getter]`: [`functions/getter/parse.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/getter/parse.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
