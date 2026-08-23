---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Traits

The traits on these pages are the machinery CGP's macros generate and consume. A component definition,
a wiring table, a derive, or a field read all expand into impls of these traits, so you meet the names
here in an expansion or a compiler error far more often than you write them. This section groups the
traits by the job they do, so that a name from a diagnostic has a page to land on.

If you are new to CGP, start with the [reference overview](/docs/reference/) rather than here. This page
is the fuller map for once the essentials are familiar.

## The traits you meet first

Three traits carry every wiring, and a wiring error names one of them. [`DelegateComponent`](wiring/delegate_component.md)
records which provider a context uses for each component. [`IsProviderFor`](wiring/is_provider_for.md)
makes a missing dependency show up by name instead of hiding behind a bare "trait not implemented."
[`CanUseComponent`](wiring/can_use_component.md) is the bound a check asserts. Read
[Wiring and checking](wiring/index.md) first.

Reading a value out of a context is the next most common need.
[Field access](field-access/index.md) covers reading one field by its type-level name, through
[`HasField`](field-access/has_field.md) and the provider-side and lifetime-safe traits around it.

## The extensible-data families

Most of the remaining traits build and read structs and enums generically, by their fields and
variants. They divide into families that mirror the extensible-data derives:

- [Field structure](shape/index.md) exposes a type's whole shape as a single type, and moves a value
  through it in both directions.
- [Record builders](builder/index.md) assemble a struct one field at a time, with field presence
  tracked at compile time so an incomplete record cannot be finalized.
- [Optional and defaulted fields](optional/index.md) relax that strict builder, filling a missing field
  from `Default` or reporting it at run time.
- [Extensible variants](variant/index.md) construct an enum from one named variant and take one apart
  variant by variant, with exhaustiveness proven in the type.
- [Structural casts](casting/index.md) convert between two data types by their shared fields or
  variants, with no hand-written conversion.
- [Type-level list algebra](type-level/index.md) holds the per-field state markers and the list
  operations the families above are built from.

## The specialized machinery

Three smaller groups each serve one area:

- [Namespaces and defaults](namespace/index.md) are the lookup traits behind a namespace and a per-type
  default.
- [Type-level strings and paths](formatting/index.md) recover a field name or a path as runtime data.
- [Monad interface](monad/index.md) holds the four traits that give a monad marker its meaning for
  monadic handler composition.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
