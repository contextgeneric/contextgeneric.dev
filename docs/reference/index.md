---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'The complete reference for every CGP construct, grouped by job, with the constructs a newcomer needs first and a lookup table for names without a page.'
---

# Reference

This is the complete reference for every CGP construct: what each one is for, when to reach for it,
how to write it, and what it generates. It is the place to look when you already know the name of the
thing you need.

If you are still learning CGP, the [tutorials](/docs/tutorials/hello) are the better starting point.
This section assumes you know what a component and a provider are, and it explains constructs one at a
time rather than building an idea up. When a page here tells you *what* a construct does and you want
to know *why* CGP works that way, the [concepts](/docs/concepts/) are where each idea is explained on
its own.

## Start with these six

Most CGP code uses a small number of constructs over and over. If you read nothing else here, read
these.

| Construct | What it does |
|---|---|
| [`#[cgp_component]`](./macros/cgp_component.md) | Turns a trait into a component that can have many implementations |
| [`#[cgp_impl]`](./macros/cgp_impl.md) | Writes one of those implementations |
| [`delegate_components!`](./macros/delegate_components.md) | Chooses which implementation a given type uses |
| [`check_components!`](./macros/check_components.md) | Verifies that choice at compile time |
| [`#[cgp_fn]`](./macros/cgp_fn.md) | Defines a trait that only ever needs one implementation |
| [`#[implicit]`](./attributes/implicit.md) | Reads a value out of the context as if it were a function argument |

The first four are the full component cycle: define, implement, wire, verify. The last two are how
most code avoids needing that cycle at all. Reach for them first, and climb to a full component when
a trait needs a second implementation.

## By what you are trying to do

The rest of the reference is grouped by the job a construct does rather than by what kind of thing
it is. If you know which *file* you want rather than which job, the sidebar groups the same pages by
kind: macros, attributes, derives, and so on.

### Define a trait, and implement it

[`#[cgp_component]`](./macros/cgp_component.md) is the foundational macro, turning one trait into
the consumer trait callers use and the provider trait implementations target. A provider is then
written with [`#[cgp_impl]`](./macros/cgp_impl.md), which keeps `self` and the consumer method
signatures; [`#[cgp_provider]`](./macros/cgp_provider.md) is the lower-level form underneath it,
which you will read in generated code more often than you write. When a trait needs only one
implementation and does not need wiring, [`#[cgp_fn]`](./macros/cgp_fn.md) builds it straight from a
function, and [`#[blanket_trait]`](./macros/blanket_trait.md) does the same from a trait with
default methods. [`#[async_trait]`](./macros/async_trait.md) is how a CGP trait declares an
`async fn`.

### Wire a type to the implementations it uses

[`delegate_components!`](./macros/delegate_components.md) builds the table that maps each component to
its provider, and [`check_components!`](./macros/check_components.md) asserts that the table is
complete and every dependency behind it is satisfied.
[`delegate_and_check_components!`](./macros/delegate_and_check_components.md) does both at once, which
suits simple wiring and getting started. Two providers appear in tables constantly:
[`UseContext`](./providers/use_context.md), which routes back through the context's own
implementation, and [`UseDefault`](./providers/use_default.md), which selects a component's default
method bodies. Underneath it all is [`DelegateComponent`](./traits/wiring/delegate_component.md), the trait
the table is made of.

### Read values out of the context

An [`#[implicit]`](./attributes/implicit.md) argument is the default way to read a field: it looks
like an ordinary function parameter and is filled from a same-named field on the context. The field
access itself comes from [`#[derive(HasField)]`](./derives/derive_has_field.md) and the
[`HasField`](./traits/field-access/has_field.md) trait, whose mutable form is
[`HasFieldMut`](./traits/field-access/has_field_mut.md). Four further traits sit underneath.
[`FieldGetter`](./traits/field-access/field_getter.md) and
[`MutFieldGetter`](./traits/field-access/mut_field_getter.md) are the provider-side forms of field
access, which a getter component is wired through, and
[`MapField`](./traits/field-access/map_field.md) with its provider-side mirror
[`FieldMapper`](./traits/field-access/field_mapper.md) reads through a field of a generic context,
where a chained `get_field` would need a lifetime bound on the field's type.

Getter traits are the sparing alternative, for the cases an implicit argument cannot reach.
[`#[cgp_auto_getter]`](./macros/cgp_auto_getter.md) generates one from the method name;
[`#[cgp_getter]`](./macros/cgp_getter.md) makes the getter a full component so the field it reads is
chosen by wiring, through [`UseField`](./providers/use_field.md), its borrowing form
[`UseFieldRef`](./providers/use_field_ref.md), or [`UseFields`](./providers/use_fields.md), which
reads the field named after each method.
[`ChainGetters`](./providers/chain_getters.md) reaches a field on a nested context, and
[`MRef`](./types/mref.md) is the return type of a getter that may lend or produce its value.

### Declare what an implementation needs

These attributes state what an implementation needs, and they divide by whether the requirement
stays private to it. [`#[uses]`](./attributes/uses.md) imports the traits the body depends on and
[`#[use_provider]`](./attributes/use_provider.md) does the same for an inner provider in a
higher-order provider. Both land on the implementation alone, so a caller never sees them. Where a
requirement should instead be part of what the trait promises, [`#[extend]`](./attributes/extend.md)
adds it as a supertrait and [`#[extend_where]`](./attributes/extend_where.md) as a predicate on the
generated trait itself. When the body needs a type that a field of the context fixes,
[`#[impl_generics]`](./attributes/impl_generics.md) declares the parameter on the implementation
alone, so callers never name it.

### Let each context choose a type

[`#[cgp_type]`](./macros/cgp_type.md) defines an abstract-type component, such as an error type, a
runtime, or a scalar, that each context fills in by wiring the component to
[`UseType<T>`](./providers/use_type.md), or to
[`UseDelegatedType`](./providers/use_delegated_type.md) to resolve it through a table. Generic code
names such a type by importing it with [`#[use_type]`](./attributes/use_type.md), which is a
different thing from the `UseType` provider despite the shared name. Beside the named components
sits [`HasType`](./components/has_type.md), CGP's tag-indexed abstract-type component, and
[`WithProvider`](./providers/with_provider.md) is the adapter that lets a foundational provider
stand in as a named component's, usually written through one of its aliases:
[`WithType`](./providers/with_type.md), [`WithField`](./providers/with_field.md),
[`WithFieldRef`](./providers/with_field_ref.md), [`WithContext`](./providers/with_context.md), and
[`WithDelegatedType`](./providers/with_delegated_type.md).

### Handle errors

[`HasErrorType`](./components/has_error_type.md) gives a context one shared error type,
[`CanRaiseError`](./components/can_raise_error.md) constructs it from a source error, and
[`CanWrapError`](./components/can_wrap_error.md) attaches detail to it. The interchangeable
strategies that satisfy them, such as `RaiseFrom`, `ReturnError`, and `DebugError`, are the [error
providers](./providers/error/index.md).

### Compute things

The [handler family](./components/handler/index.md) models computation along three axes: synchronous
or async, fallible or not, taking an input or not. [`Computer`](./components/handler/computer.md) is
the plain synchronous transform, [`TryComputer`](./components/handler/try_computer.md) adds
fallibility, [`Handler`](./components/handler/handler.md) is the general async and fallible case,
and [`Producer`](./components/handler/producer.md) is the input-free one.
[`CanRun`](./components/runner.md) runs tasks, [`CanSendRun`](./components/send_runner.md) is its
`Send`-future variant, and [`HasRuntime`](./components/has_runtime.md) with
[`HasRuntimeType`](./components/has_runtime_type.md) supplies the runtime they run on. `Computer`,
`TryComputer`, and `Handler` also have by-reference siblings, and `Computer` has async ones:
[`ComputerRef`](./components/handler/computer_ref.md),
[`TryComputerRef`](./components/handler/try_computer_ref.md),
[`HandlerRef`](./components/handler/handler_ref.md),
[`AsyncComputer`](./components/handler/async_computer.md), and
[`AsyncComputerRef`](./components/handler/async_computer_ref.md). Each is a component with its own
page under the [handler family](./components/handler/index.md).

Providers in this family are written from plain functions with
[`#[cgp_computer]`](./macros/cgp_computer.md) and [`#[cgp_producer]`](./macros/cgp_producer.md), and
composed with the [handler combinators](./providers/handler/index.md), the [dispatch
combinators](./providers/dispatch/index.md), and the [monad providers](./providers/monad/index.md)
built on the monad traits: [`MonadicBind`](./traits/monad/monadic_bind.md),
[`ContainsValue`](./traits/monad/contains_value.md), [`LiftValue`](./traits/monad/lift_value.md),
and [`MonadicTrans`](./traits/monad/monadic_trans.md).
[`#[cgp_auto_dispatch]`](./macros/cgp_auto_dispatch.md) generates a dispatching handler from a
per-type trait.

### Work with a type's structure

These let generic code build and read structs and enums by their fields and variants without naming
the concrete type. What a type opts in with is a derive:
[`#[derive(CgpData)]`](./derives/derive_cgp_data.md) is the umbrella one, with
[`#[derive(CgpRecord)]`](./derives/derive_cgp_record.md) and
[`#[derive(CgpVariant)]`](./derives/derive_cgp_variant.md) as its struct and enum faces, and its
individual slices are [`#[derive(HasFields)]`](./derives/derive_has_fields.md),
[`#[derive(BuildField)]`](./derives/derive_build_field.md),
[`#[derive(ExtractField)]`](./derives/derive_extract_field.md), and
[`#[derive(FromVariant)]`](./derives/derive_from_variant.md).

Each of those generates traits, and the traits are where the operations live. The whole-shape view is
[`HasFields`](./traits/shape/has_fields.md) with its conversions [`ToFields`](./traits/shape/to_fields.md) and
[`FromFields`](./traits/shape/from_fields.md); the builder family runs from
[`HasBuilder`](./traits/builder/has_builder.md) through [`BuildField`](./traits/builder/build_field.md) to
[`FinalizeBuild`](./traits/builder/finalize_build.md); the extractor family runs from
[`HasExtractor`](./traits/variant/has_extractor.md) through [`ExtractField`](./traits/variant/extract_field.md) to
[`FinalizeExtract`](./traits/variant/finalize_extract.md), with
[`FromVariant`](./traits/variant/from_variant.md) constructing rather than deconstructing. Underneath sit the
presence markers of [`MapType`](./traits/type-level/map_type.md), the list algebra of
[`AppendProduct`](./traits/type-level/append_product.md), the structural casts
[`CanUpcast`](./traits/casting/can_upcast.md) and [`CanBuildFrom`](./traits/casting/can_build_from.md), and the
optional-field extensions starting at
[`HasOptionalBuilder`](./traits/optional/has_optional_builder.md). Each entry in a shape is a
[`Field`](./types/field.md).

Those families run to a good many traits, most of which you read rather than write. The table below is
the whole set, so that a name met in an expansion or an error message can be looked up from here.

| Family | The traits in it |
|---|---|
| The shape | [`HasFields`](./traits/shape/has_fields.md), [`HasFieldsRef`](./traits/shape/has_fields_ref.md), [`ToFields`](./traits/shape/to_fields.md), [`FromFields`](./traits/shape/from_fields.md), [`ToFieldsRef`](./traits/shape/to_fields_ref.md) |
| Building a record | [`HasBuilder`](./traits/builder/has_builder.md), [`IntoBuilder`](./traits/builder/into_builder.md), [`BuildField`](./traits/builder/build_field.md), [`TakeField`](./traits/builder/take_field.md), [`UpdateField`](./traits/builder/update_field.md), [`PartialData`](./traits/builder/partial_data.md), [`FinalizeBuild`](./traits/builder/finalize_build.md) |
| Taking an enum apart | [`HasExtractor`](./traits/variant/has_extractor.md), [`HasExtractorRef`](./traits/variant/has_extractor_ref.md), [`HasExtractorMut`](./traits/variant/has_extractor_mut.md), [`ExtractField`](./traits/variant/extract_field.md), [`FinalizeExtract`](./traits/variant/finalize_extract.md), [`FinalizeExtractResult`](./traits/variant/finalize_extract_result.md), [`FromVariant`](./traits/variant/from_variant.md) |
| Converting between shapes | [`CanUpcast`](./traits/casting/can_upcast.md), [`CanDowncast`](./traits/casting/can_downcast.md), [`CanDowncastFields`](./traits/casting/can_downcast_fields.md), [`CanBuildFrom`](./traits/casting/can_build_from.md) |
| Optional and defaulted fields | [`HasOptionalBuilder`](./traits/optional/has_optional_builder.md), [`ToOptional`](./traits/optional/to_optional.md), [`SetOptional`](./traits/optional/set_optional.md), [`FinalizeOptional`](./traits/optional/finalize_optional.md), [`CanFinalizeWithDefault`](./traits/optional/can_finalize_with_default.md), [`CanBuildWithDefault`](./traits/optional/can_build_with_default.md) |
| Field state, and changing it | [`MapType`](./traits/type-level/map_type.md), [`MapTypeRef`](./traits/type-level/map_type_ref.md), [`TransformMap`](./traits/type-level/transform_map.md), [`TransformMapFields`](./traits/type-level/transform_map_fields.md), [`TransformMapDefault`](./traits/optional/transform_map_default.md), [`TransformOptional`](./traits/optional/transform_optional.md) |
| Computing a shape | [`AppendProduct`](./traits/type-level/append_product.md), [`ConcatProduct`](./traits/type-level/concat_product.md), [`MapFields`](./traits/type-level/map_fields.md) |

### Keep large wiring manageable

[`cgp_namespace!`](./macros/cgp_namespace.md) defines a reusable, inheritable wiring table that many
contexts can join, which is how top-level wiring stays short as component counts grow. It works
through [`RedirectLookup`](./providers/redirect_lookup.md), which re-routes a lookup along a
[`Path!`](./macros/path.md), together with the three lookup traits that resolve inherited and
per-type defaults: [`DefaultNamespace`](./traits/namespace/default_namespace.md) for a key that is a
component alone, and [`DefaultImpls1`](./traits/namespace/default_impls1.md) and
[`DefaultImpls2`](./traits/namespace/default_impls2.md) when the key carries one further type or
two. A component registers into a namespace under a path prefix with
[`#[prefix(...)]`](./attributes/prefix.md), and a provider registers itself as one of those defaults
with [`#[default_impl(...)]`](./attributes/default_impl.md). The `open` statement of
[`delegate_components!`](./macros/delegate_components.md) is a lightweight special case of the same
mechanism, and it supersedes the older [`UseDelegate`](./providers/use_delegate.md) tables and the
[`#[derive_delegate]`](./attributes/derive_delegate.md) attribute that generates them.

### Understand a generated type or an error

These are the type-level building blocks the rest of CGP is made of, and the place to look when a
generated type or an error names one you did not write. [`PhantomData`](./types/phantom_data.md) is
the one to start with: it is what lets a provider or a tag carry a type without storing a value of
it, and it underlies most of the rest. You write the lists through sugar, with
[`Symbol!`](./macros/symbol.md) for a field name, [`Product!`](./macros/product.md) for a record
list, [`Sum!`](./macros/sum.md) for its dual, and [`Path!`](./macros/path.md) for a route. You only
need to recognize the [lists](./types/index.md) they expand into, [`Cons`](./types/cons.md) and
[`Nil`](./types/nil.md), [`Either`](./types/either.md) and [`Void`](./types/void.md),
[`Chars`](./types/chars.md), and [`PathCons`](./types/path_cons.md), when one shows up in an error.
Each entry in a record or a variant is a [`Field`](./types/field.md), tagged by a
[`Symbol!`](./macros/symbol.md) or an [`Index`](./types/index_type.md), and
[`Life`](./types/life.md) lifts a lifetime into a type where the wiring needs one.
[`MRef`](./types/mref.md) is the odd one out, a runtime value a getter returns rather than a
type-level marker. Two traits turn a type-level string back into runtime data:
[`StaticString`](./traits/formatting/static_string.md) decodes it into a constant, and
[`StaticFormat`](./traits/formatting/static_format.md) writes it into a formatter, which is what
makes it printable. [`ConcatPath`](./traits/formatting/concat_path.md) joins two paths at the type
level.

When wiring fails, three traits are what you will see named:
[`DelegateComponent`](./traits/wiring/delegate_component.md),
[`IsProviderFor`](./traits/wiring/is_provider_for.md), which is what makes a missing dependency show up by
name, and [`CanUseComponent`](./traits/wiring/can_use_component.md), which is what a check asserts. The
[compile errors](./errors.md) page covers the recurring failures and how to read them.

If what you are missing is a *word* rather than a construct, such as environmental context,
impl-side dependency, or type class, the [glossary](./glossary.md) defines the terms this
documentation uses and links to the page that explains each one.

## Looking for a name you don't see?

Almost every construct has a page of its own. The exceptions are names that are not separately
*constructs*: a **marker** is a type implementing a trait, an **alias** is another spelling of a
construct, and a **variant** differs from a base construct by one axis. A component's **consumer
trait** shares a page with its provider trait, and the **helper traits and generated types** a
construct uses are explained on its page. Each is documented on the page of the thing it belongs to.
If you arrived knowing one of these names, this is where it lives.

| Looking for | It's on |
|---|---|
| `#[cgp_new_provider]` | [`#[cgp_provider]`](./macros/cgp_provider.md) |
| `Symbol` (the type, not the `Symbol!` macro) | [`Symbol!`](./macros/symbol.md) and [`Chars`](./types/chars.md) |
| `IdentMonadic`, `OkMonadic`, `ErrMonadic`, `OkMonadicTrans`, `ErrMonadicTrans` | [Monad providers](./providers/monad/index.md) |
| `MatchWithHandlersRef`, `MatchWithHandlersMut` | [`MatchWithHandlers`](./providers/dispatch/match_with_handlers.md) |
| `MatchFirstWithHandlersRef`, `MatchFirstWithHandlersMut`, and the `MatchFirstWithValueHandlers` and `MatchFirstWithFieldHandlers` forms | [`MatchFirstWithHandlers`](./providers/dispatch/match_first_with_handlers.md) |
| `MatchWithValueHandlersRef`, `MatchWithValueHandlersMut` | [`MatchWithValueHandlers`](./providers/dispatch/match_with_value_handlers.md) |
| `MatchWithFieldHandlersRef` | [`MatchWithFieldHandlers`](./providers/dispatch/match_with_field_handlers.md) |
| `ExtractFirstFieldAndHandle`, `HandleFirstFieldValue` | [`ExtractFieldAndHandle`](./providers/dispatch/extract_field_and_handle.md), [`HandleFieldValue`](./providers/dispatch/handle_field_value.md) |
| `DispatchMatchers` | [`MatchWithHandlers`](./providers/dispatch/match_with_handlers.md) |
| `ToFieldHandlers`, `HasFieldHandlers`, `MapFieldHandler`, `MapExtractFieldAndHandle`, `MapExtractFirstFieldAndHandle` | [`MatchWithFieldHandlers`](./providers/dispatch/match_with_field_handlers.md) |
| `ToBuildAndMergeHandler` | [`BuildAndMergeOutputs`](./providers/dispatch/build_and_merge_outputs.md) |
| `IsPresent`, `IsNothing`, `IsVoid`, `IsOptional` | [`MapType`](./traits/type-level/map_type.md) |
| `IsRef`, `IsMut`, `IsOwned` | [`MapTypeRef`](./traits/type-level/map_type_ref.md) |
| `product!` (the value-level form) | [`Product!`](./macros/product.md) |
| `CanCompute`, `CanComputeRef`, `CanComputeAsync`, `CanComputeAsyncRef` | [`Computer`](./components/handler/computer.md), [`ComputerRef`](./components/handler/computer_ref.md), [`AsyncComputer`](./components/handler/async_computer.md), [`AsyncComputerRef`](./components/handler/async_computer_ref.md) |
| `CanTryCompute`, `CanTryComputeRef`, `CanHandle`, `CanHandleRef`, `CanProduce` | [`TryComputer`](./components/handler/try_computer.md), [`TryComputerRef`](./components/handler/try_computer_ref.md), [`Handler`](./components/handler/handler.md), [`HandlerRef`](./components/handler/handler_ref.md), [`Producer`](./components/handler/producer.md) |
| `ErrorOf`, `ErrorOnly` | [`HasErrorType`](./components/has_error_type.md) |
| `TypeOf` | [`HasType`](./components/has_type.md) |
| `RuntimeOf` | [`HasRuntimeType`](./components/has_runtime_type.md) |
| `FieldsBuilder`, `FieldsExtractor` | [`CanBuildFrom`](./traits/casting/can_build_from.md), [`CanUpcast`](./traits/casting/can_upcast.md) |
| `TryPromoteProviders` | [`PipeMonadic`](./providers/monad/pipe_monadic.md) |
| `__Partial{Name}`, `__PartialRef{Name}` (a derive's companion types) | [`#[derive(BuildField)]`](./derives/derive_build_field.md), [`#[derive(ExtractField)]`](./derives/derive_extract_field.md) |
| `{Name}Component` (a component's marker) | [`#[cgp_component]`](./macros/cgp_component.md) |
| `#[check_trait(...)]`, `#[check_providers(...)]` | [`check_components!`](./macros/check_components.md) |
| `#[check_params(...)]`, `#[skip_check]` | [`delegate_and_check_components!`](./macros/delegate_and_check_components.md) |

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
