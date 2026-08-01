---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Reference

This is the complete reference for every CGP construct: what each one is for, when to reach for it,
how to write it, and what it generates. It is the place to look when you already know the name of the
thing you need.

If you are still learning CGP, the [tutorials](/docs/tutorials/hello) are the better starting point.
This section assumes you know what a component and a provider are, and it explains constructs one at a
time rather than building an idea up.

## Start with these six

Most CGP code uses a small number of constructs over and over. If you read nothing else here, read
these.

| Construct | What it does |
|---|---|
| [`#[cgp_component]`](./macros/cgp_component.md) | Turns a trait into a component that can have many implementations |
| [`#[cgp_impl]`](./macros/cgp_impl.md) | Writes one of those implementations |
| [`delegate_components!`](./macros/delegate_components.md) | Chooses which implementation a given type uses |
| [`check_components!`](./macros/check_components.md) | Verifies that choice at compile time |
| [`#[cgp_fn]`](./macros/cgp_fn.md) | Defines a capability that only ever needs one implementation |
| [`#[implicit]`](./attributes/implicit.md) | Reads a value out of the context as if it were a function argument |

The first four are the full component cycle: define, implement, wire, verify. The last two are how
most code avoids needing that cycle at all — reach for them first, and climb to a full component when
a capability genuinely needs a second implementation.

## By what you are trying to do

The rest of the reference is grouped by the job a construct does rather than by what kind of thing it
is. If you know which *file* you want rather than which job, the sidebar groups the same pages by kind
— macros, attributes, derives, and so on.

### Define a capability, and implement it

[`#[cgp_component]`](./macros/cgp_component.md) is the foundational macro, turning one trait into the
consumer trait callers use and the provider trait implementations target. A provider is then written
with [`#[cgp_impl]`](./macros/cgp_impl.md), which keeps `self` and the consumer method signatures;
[`#[cgp_provider]`](./macros/cgp_provider.md) is the lower-level form underneath it, which you will
read in generated code more often than you write. When a capability needs only one implementation and
no wiring at all, [`#[cgp_fn]`](./macros/cgp_fn.md) builds it straight from a function, and
[`#[blanket_trait]`](./macros/blanket_trait.md) does the same from a trait with default methods.
[`#[async_trait]`](./macros/async_trait.md) is how a CGP trait declares an `async fn`.

### Wire a type to the implementations it uses

[`delegate_components!`](./macros/delegate_components.md) builds the table that maps each component to
its provider, and [`check_components!`](./macros/check_components.md) asserts that the table is
complete and every dependency behind it is satisfied.
[`delegate_and_check_components!`](./macros/delegate_and_check_components.md) does both at once, which
suits simple wiring and getting started. Two providers appear in tables constantly:
[`UseContext`](./providers/use_context.md), which routes back through the context's own
implementation, and [`UseDefault`](./providers/use_default.md), which selects a component's default
method bodies. Underneath it all is [`DelegateComponent`](./traits/delegate_component.md), the trait
the table is made of.

### Read values out of the context

An [`#[implicit]`](./attributes/implicit.md) argument is the default way to read a field: it looks like
an ordinary function parameter and is filled from a same-named field on the context. The field access
itself comes from [`#[derive(HasField)]`](./derives/derive_has_field.md) and the
[`HasField`](./traits/has_field.md) trait.

Getter traits are the sparing alternative, for the cases an implicit argument cannot reach.
[`#[cgp_auto_getter]`](./macros/cgp_auto_getter.md) generates one from the method name;
[`#[cgp_getter]`](./macros/cgp_getter.md) makes the getter a full component so the field it reads is
chosen by wiring, through [`UseField` and its siblings](./providers/use_field.md).
[`ChainGetters`](./providers/chain_getters.md) reaches a field on a nested context, and
[`MRef`](./types/mref.md) is the return type of a getter that may lend or produce its value.

### Declare what an implementation needs

These attributes state a provider's dependencies where the implementation lives, so they never appear
in the interface callers see. [`#[uses]`](./attributes/uses.md) imports the capabilities it depends on,
[`#[use_provider]`](./attributes/use_provider.md) does the same for an inner provider in a
higher-order provider, and [`#[extend]`](./attributes/extend.md) and
[`#[extend_where]`](./attributes/extend_where.md) add bounds to a generated trait rather than to its
implementation.

### Let each context choose a type

[`#[cgp_type]`](./macros/cgp_type.md) defines an abstract-type component — an error type, a runtime, a
scalar — that each context fills in by wiring the component to
[`UseType<T>`](./providers/use_type.md), or to
[`UseDelegatedType`](./providers/use_delegated_type.md) to resolve it through a table. Generic code
names such a type by importing it with [`#[use_type]`](./attributes/use_type.md), which is a different
thing from the `UseType` provider despite the shared name. All of it rests on
[`HasType`](./components/has_type.md), CGP's built-in abstract-type component, and
[`WithProvider`](./providers/with_provider.md) is the adapter that lets a foundational provider stand
in as a named component's.

### Handle errors

[`HasErrorType`](./components/has_error_type.md) gives a context one shared error type, and
[`CanRaiseError` and `CanWrapError`](./components/can_raise_error.md) construct it from a source error
and attach detail. The interchangeable strategies that satisfy them — `RaiseFrom`, `ReturnError`,
`DebugError`, and the rest — are the [error providers](./providers/error_providers.md).

### Compute things

The handler family models computation along three axes: synchronous or async, fallible or not, taking
an input or not. [`Computer`](./components/computer.md) is the plain synchronous transform,
[`TryComputer`](./components/try_computer.md) adds fallibility,
[`Handler`](./components/handler.md) is the general async and fallible case, and
[`Producer`](./components/producer.md) is the input-free one.
[`CanRun`](./components/runner.md) runs tasks and [`HasRuntime`](./components/has_runtime.md) supplies
the runtime they run on.

Providers in this family are written from plain functions with
[`#[cgp_computer]`](./macros/cgp_computer.md) and [`#[cgp_producer]`](./macros/cgp_producer.md), and
composed with the [handler combinators](./providers/handler_combinators.md), the
[dispatch combinators](./providers/dispatch_combinators.md), and the
[monad providers](./providers/monad_providers.md) built on the
[monad traits](./traits/monad.md). [`#[cgp_auto_dispatch]`](./macros/cgp_auto_dispatch.md) generates a
dispatching handler from a per-type trait.

### Work with a type's structure

These let generic code build and read structs and enums by their fields and variants without naming
the concrete type. [`#[derive(CgpData)]`](./derives/derive_cgp_data.md) is the umbrella derive, and its
individual slices are [`HasFields`](./derives/derive_has_fields.md),
[`BuildField`](./derives/derive_build_field.md),
[`ExtractField`](./derives/derive_extract_field.md), and
[`FromVariant`](./derives/derive_from_variant.md).

The traits behind them are [`HasFields`](./traits/has_fields.md), the builder family in
[`HasBuilder`](./traits/has_builder.md), the extractor family in
[`ExtractField`](./traits/extract_field.md), [`FromVariant`](./traits/from_variant.md), the presence
markers of [`MapType`](./traits/map_type.md), the list algebra of
[`AppendProduct`](./traits/product_ops.md), the structural
[casts](./traits/cast.md), and the [optional-field extensions](./traits/optional_fields.md). Each entry
in a shape is a [`Field`](./types/field.md).

### Keep large wiring manageable

[`cgp_namespace!`](./macros/cgp_namespace.md) defines a reusable, inheritable wiring table that many
contexts can join, which is how top-level wiring stays short as component counts grow. It works
through [`RedirectLookup`](./providers/redirect_lookup.md), which re-routes a lookup along a
[`Path!`](./macros/path.md), together with the
[`DefaultNamespace`](./traits/default_namespace.md) traits that resolve inherited and per-type
defaults. The `open` statement of [`delegate_components!`](./macros/delegate_components.md) is a
lightweight special case of the same mechanism, and it supersedes the older
[`UseDelegate`](./providers/use_delegate.md) tables and the
[`#[derive_delegate]`](./attributes/derive_delegate.md) attribute that generates them.

### Understand a generated type or an error

These are the type-level building blocks the rest of CGP is made of. You mostly write them through
sugar — [`Symbol!`](./macros/symbol.md) for a field name, [`Product!`](./macros/product.md) for a list,
[`Sum!`](./macros/sum.md) for its dual, [`Path!`](./macros/path.md) for a route — and only need to
recognize [what they expand into](./types/type_level_spines.md) when it appears in an error message.
[`Index`](./types/index.md) tags a tuple field and [`Life`](./types/life.md) lifts a lifetime into a
type, while [`StaticFormat`](./traits/static_format.md) turns type-level strings back into runtime
data.

When wiring fails, three traits are what you will see named:
[`DelegateComponent`](./traits/delegate_component.md),
[`IsProviderFor`](./traits/is_provider_for.md), which is what makes a missing dependency show up by
name, and [`CanUseComponent`](./traits/can_use_component.md), which is what a check asserts. The
[compile errors](./errors.md) page covers the recurring failures and how to read them.

## Looking for a name you don't see?

A few constructs are documented alongside a close relative rather than on a page of their own, because
they are chosen together and separate pages would make you collate them. If you arrived knowing one of
these names, this is where it lives.

| Looking for | It's on |
|---|---|
| `#[cgp_new_provider]` | [`#[cgp_provider]`](./macros/cgp_provider.md) |
| `#[derive(CgpRecord)]`, `#[derive(CgpVariant)]` | [`#[derive(CgpData)]`](./derives/derive_cgp_data.md) |
| `UseFieldRef`, `UseFields` | [`UseField`](./providers/use_field.md) |
| `WithType`, `WithField`, `WithContext` | [`WithProvider`](./providers/with_provider.md) |
| `Cons`, `Nil`, `Either`, `Void`, `Chars`, `PathCons` | [Type-level spines](./types/type_level_spines.md) |
| `HasFieldMut`, `FieldGetter` | [`HasField`](./traits/has_field.md) |
| `ConcatProduct`, `MapFields` | [`AppendProduct`](./traits/product_ops.md) |
| `CanDowncast`, `CanBuildFrom` | [`CanUpcast`](./traits/cast.md) |
| `StaticString`, `ConcatPath` | [`StaticFormat`](./traits/static_format.md) |
| `ComposeHandlers`, `PipeHandlers`, `ReturnInput`, `Promote*` | [Handler combinators](./providers/handler_combinators.md) |
| `MatchWithHandlers`, `ExtractFieldAndHandle` | [Dispatch combinators](./providers/dispatch_combinators.md) |
| `PipeMonadic`, `BindOk`, `BindErr` | [Monad providers](./providers/monad_providers.md) |
| `RaiseFrom`, `ReturnError`, `DebugError`, `DisplayError` | [Error providers](./providers/error_providers.md) |
| `#[impl_generics(...)]` | [`#[cgp_fn]`](./macros/cgp_fn.md) |
| `#[prefix(...)]` | [`cgp_namespace!`](./macros/cgp_namespace.md) |
| `#[default_impl(...)]` | [`DefaultNamespace`](./traits/default_namespace.md) |
| `#[check_providers(...)]`, `#[check_params(...)]` | [`check_components!`](./macros/check_components.md) |

## This reference is still being written

Most pages here are placeholders at present, and each one says so. The construct list is complete —
every construct the `cgp` crate exports has a page, or a named place on a page it shares — so nothing
is missing from this index even where the page behind it is not yet filled in.
