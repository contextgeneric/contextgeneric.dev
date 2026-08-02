---
sidebar_label: 'Compile errors'
sidebar_position: 90
---

# Compile errors

The errors CGP produces after macro expansion, and how to read each one back to the mistake behind it.

CGP's macros expand to ordinary Rust, so most mistakes are not caught by a macro at all. They are caught
by the compiler afterwards, checking code you did not write — which is why a CGP error can name types
that appear nowhere in your source. This page is the catalog of those failures: what each one looks
like, whether the message contains the real cause, and what to change.

It is meant to be looked up rather than read through. If you have an error in front of you, start from
[Find your error](#find-your-error).

## Run the checker first

**Before decoding anything, run [`cargo cgp check`](/docs/cargo-cgp/check) in place of `cargo check`.**
It rewrites the classes it recognizes to lead with the cause, and for the worst class it recovers a
cause that plain `cargo check` discards entirely. Every "what you see" example below is its real output.

It is a `v0.1.0-alpha` and does not reshape everything. Where a class passes through as the compiler
wrote it, this page says so and shows the raw form instead.

## The one distinction that explains everything

**The same mistake produces two completely different errors depending on how you run into it**, and
knowing which one you are looking at decides whether there is a cause in the message to find at all.

CGP wiring is lazy: [`delegate_components!`](./macros/delegate_components.md) records that a **context**
— the type the capability runs against, which supplies the values it needs as its fields — uses some
provider, and nothing checks that the provider's own requirements hold. That check happens later, and
*how* it happens decides what you see.

- **Surfaced.** When a [`check_components!`](./macros/check_components.md) block forces the question, the
  compiler proves the obligation outright and names the bound that failed. The cause is in the message.
- **Hidden.** When you instead call the method on the context, the compiler sees a blanket impl among
  several candidates, abandons it, and reports only that the method's bounds were not satisfied. The
  cause is **absent** — not buried. Reading harder will not find it.

This is why the standard first move on a confusing CGP error is to add a check at the wiring site: it
converts a hidden error into a surfaced one. `cargo cgp check` does the equivalent for you, which is why
both forms produce the same output through the tool.

## Find your error

Match the compiler's code and the shape of the message.

| What the compiler says | Usually means | Section |
|---|---|---|
| `E0599` method exists but its trait bounds were not satisfied | A provider's dependency is unmet, reached by a method call | [A dependency is not met](#a-dependency-is-not-met) |
| `E0599` no associated function found for type parameter | An inner provider called but never imported | [Using something you did not declare](#using-something-you-did-not-declare) |
| `E0277` … `CanUseComponent` is not satisfied | A checked component's provider is missing something | [A dependency is not met](#a-dependency-is-not-met) |
| `E0277` the trait bound `f64: Eq` is not satisfied | An ordinary Rust bound on a wired type | [When the missing bound is an ordinary Rust trait](#when-the-missing-bound-is-an-ordinary-rust-trait) |
| `E0277` … `PathCons<…>: SomeNamespace<…>` is not satisfied | A namespace path nothing binds | [Nothing is wired there](#nothing-is-wired-there) |
| `E0277` the size for values of type `[u8]` cannot be known | A field-type shorthand lowered to an unsized type | [A field-type shorthand with no rule](#a-field-type-shorthand-with-no-rule) |
| `E0271` type mismatch resolving `<… as …>::Assoc == …` | A type pinned one way and wired another | [When two decisions disagree about a type](#when-two-decisions-disagree-about-a-type) |
| `E0119` conflicting implementations | Two wiring entries claim one key | [The wiring itself does not compile](#the-wiring-itself-does-not-compile) |
| `E0428` the name … is defined multiple times | A generated name declared twice | [Two entries claim one key](#two-entries-claim-one-key) |
| `E0275` overflow evaluating the requirement | A wiring or inheritance cycle | [A lookup that never terminates](#a-lookup-that-never-terminates) |
| `E0210` / `E0117` orphan rule | Registering into a namespace you do not own | [Registering into a namespace you do not own](#registering-into-a-namespace-you-do-not-own) |
| `E0207` type parameter is not constrained | A per-entry generic that never reaches the key | [A generic that reaches nothing](#a-generic-that-reaches-nothing) |
| `E0107` trait takes N generic arguments but N+1 supplied | A consumer trait written where a provider trait belongs | [Naming the wrong half of a component](#naming-the-wrong-half-of-a-component) |
| `E0433` / `E0425` cannot find type | A name the generated code cannot see | [A name the generated code cannot see](#a-name-the-generated-code-cannot-see) |
| `E0576` cannot find associated type … in trait | A `#[use_type]` import with a wrong name | [An imported type that does not exist](#an-imported-type-that-does-not-exist) |
| `E0404` expected trait, found type parameter | An abstract type named after its own bound | [A name the generated code cannot see](#a-name-the-generated-code-cannot-see) |

## A dependency is not met

This is the most common CGP failure by a wide margin: a provider needs something from its context, and
the context does not supply it.

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) {
        let _ = self.name();
    }
}

#[derive(HasField)]
pub struct Person {
    pub age: u8, // no `name` field, so `Person` cannot satisfy `HasName`
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}
```

That block **compiles**. Nothing has asked yet whether `GreetHello` can do its job here.

### What you see

Through the tool, both routes to the failure give the same answer. Checking it:

```text
error[E0277]: [CGP-E001] the consumer trait `CanGreet` is not implemented for context `Person`
  --> src/main.rs:56:9
   |
56 |         GreeterComponent,
   |         ^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E106] missing field `name` on `Person`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanGreet` for context `Person`
             └─ [CGP-E102] provider trait impl `Greeter` with context `Person` for provider `GreetHello`
               └─ [CGP-E105] trait impl `HasName` for `Person`
                 └─ [CGP-E106] missing field `name` on `Person`
```

**Read the `root cause:` line and stop.** It names the fix. The tree beneath it is the chain of
obligations that demanded it, read top to bottom: the context must implement the consumer trait, so its
wired provider must be a provider for that context, so the context must have the field.

Calling `person.greet()` instead of checking produces the identical note under an `E0599`. That recovery
is the single largest thing the tool does.

### What the raw compiler says

Without the tool the two routes diverge sharply. The **checked** form is readable: an `E0277` on
`CanUseComponent`, with a `help:` note naming the missing `HasField<Symbol!("name")>` and a
`required for …` chain building outward from it. The cause sits early, in that `help:` note, rather than
at the end.

The **method-call** form is the one to learn to recognize, because there is nothing in it to find:

```text
error[E0599]: the method `greet` exists for struct `Person`, but its trait bounds were not satisfied
   |
   | pub struct Person {
   | ----------------- method `greet` not found for this struct because it doesn't satisfy
   |                   `Person: CanGreet` or `Person: Greeter<Person>`
   |
   |     Person { age: 0 }.greet();
   |                       ^^^^^ this is an associated function, not a method
```

It names the traits it could not satisfy and stops. `HasName` and the missing field appear nowhere. The
"this is an associated function, not a method" line is an artifact of how a provider trait is generated
— the suggestion to write `Person::greet()` is wrong, and it is noise rather than a clue.

The suppression is a [long-standing compiler limitation](https://github.com/rust-lang/rust/issues/61661)
rather than a CGP quirk: with a blanket impl among the candidates, rustc declines to expand which
`where` bound made it inapplicable.

### The fix

Give the context what the provider needs — add the `name` field, or wire the getter to a field that
exists. If you are looking at the hidden form, add a check at the wiring site first:

```rust
check_components! {
    Person {
        GreeterComponent,
    }
}
```

### When the derive is missing entirely

A variant worth knowing, because the fix differs and the usual landmark disappears. If the struct *has*
the field but never derived [`HasField`](./derives/derive_has_field.md), it implements the accessor for
nothing at all — and the tool reports that as one cause rather than one per field:

```text
   = help: make sure that `#[derive(HasField)]` is used for `Person`
   = note: root cause: [CGP-E108] accessor trait `HasField` with field `name` is not implemented for `Person`
```

In the raw output the tell is an absence. A genuinely missing field draws a "but trait
`HasField<Symbol!("age")>` is implemented for it" hint pointing at the nearest field; a missing derive
cannot, because no field is implemented.

### When the missing bound is an ordinary Rust trait

A provider's dependency may be a plain Rust trait rather than a CGP capability — `Scalar: Eq`,
`Item: Ord` — unmet by the type the context wired. Here the tool deliberately keeps rustc's own headline,
because it was already the clearest statement, and tidies only the frames beneath it:

```text
error[E0277]: the trait bound `f64: Eq` is not satisfied
  --> src/main.rs:59:9
   |
59 |         ScalarEqualityComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^ the trait `Eq` is not implemented for `f64`
   |
   = note: this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCompareScalars` for context `App`
             └─ [CGP-E102] provider trait impl `ScalarEquality` with context `App` for provider `CompareScalars`
               └─ the trait bound `f64: Eq` is not satisfied
```

This is the one class here with no `[CGP-Exxx]` headline, and the absence is deliberate.

The fix is not to wire a component or add a field but to satisfy the trait: wire the abstract type to
something that implements it, implement it for a type you own, or relax the provider's bound if it asked
for more than it needed. One connective step is left to you — the message names `f64`, not the
`ScalarTypeProviderComponent: UseType<f64>` line that chose it.

### When two decisions disagree about a type

A provider can *pin* an [abstract type](./macros/cgp_type.md) to a concrete one while the context wires
that same abstract type to something else. The trait half of the bound still holds, so the compiler
reports a projection mismatch rather than a missing impl:

```text
error[E0271]: [CGP-E017] expected the abstract type `Scalar` of `HasScalarType` on `Rectangle` to be `f64`, but found `u32`
   |
   = help: wire `ScalarTypeProviderComponent` to `UseType<f64>` in the wiring for `Rectangle`, or
           change the provider to work with `u32`
```

Both halves of the disagreement are named, and the `help:` names the wiring line to change. The raw
`E0271` is much worse: it states only what was *required*, never what the context supplied, and puts its
caret on the `#[cgp_type]` attribute that declared the type — nowhere near either decision.

The same shape appears one level down, when a *field's* type is expressed through an abstract type
(`#[implicit] database: &Pool<Db>`). The tool then gives the requirement in both forms —
`Pool<<App as HasDbType>::Db>` followed by `(Pool<Postgres>)` — because the projection says where the
requirement came from and the reduced type is what you compare the field against.

### When the provider is a stack

With a [higher-order provider](./attributes/use_provider.md) the question is which layer to fix, and the
tool answers it in the shape of the tree. An inner-layer failure shows **two** provider hops:

```text
   = note: root cause: [CGP-E106] missing field `base_area` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `ScaledArea<BaseArea>`
               └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `BaseArea`
                 └─ [CGP-E105] trait impl `HasBaseArea` for `Rectangle`
                   └─ [CGP-E106] missing field `base_area` on `Rectangle`
```

An outer-layer failure shows **one**, with the inner provider never named. Either way the wiring is
correct and the fix is to supply the field; the depth only tells you which layer wanted it.

When a stack is deep enough that this is still awkward, the `#[check_providers(...)]` form of
[`check_components!`](./macros/check_components.md) asserts each layer separately, so a dependency
missing only from the wrapper fails on its line alone.

### When one mistake prints many errors

A dependency several components share, left unmet, fails at every one of them. Raw `cargo check` prints
one block per affected provider — three chained components over one missing field produce **six** — and
only some of them reach the cause.

The tool collapses the whole cascade into one block naming every affected consumer over a single tree:

```text
error[E0277]: [CGP-E001] the consumer traits `CanBaz`, `CanBar`, and `CanFoo` are not implemented for context `App`
   |
85 |         BazComponent,
   |         ^^^^^^^^^^^^
86 |         BarComponent,
   |         ^^^^^^^^^^^^
87 |         FooComponent,
   |         ^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E106] missing field `name` on `App`
```

Fix the one cause and all of it disappears together.

On the raw output the rule for finding that cause is structural rather than positional: **look for a
block whose `help:` names a concrete missing item** — a field, a type — rather than another provider
trait. Blocks naming `SomeProvider: SomeTrait<App>` are consequences. Two habits help besides: check one
suspect component in its own `check_components!` block, or comment entries out until the failure moves.

## Nothing is wired there

A neighbouring failure that reads like a dependency problem but is not: no provider is found *at all*,
rather than one being found whose requirements fail.

The common case is a component registered into a [namespace](./macros/cgp_namespace.md) under a path that
nothing ever binds — a `#[prefix]` routes the lookup somewhere, and no `#[default_impl]`, namespace entry,
or direct wiring puts a provider there:

```text
   = note: root cause: [CGP-E107] context `App` does not contain any delegate entry for `@app.GreeterComponent`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanGreet` for context `App`
             └─ [CGP-E104] redirect lookup to `@app.GreeterComponent` in `App`
               └─ [CGP-E107] context `App` does not contain any delegate entry for `@app.GreeterComponent`
```

The same `[CGP-E107]` leaf covers a component the context simply never wired. The path is resugared to
its `@app.GreeterComponent` form, which is the largest readability win here — raw `rustc` prints it as a
`PathCons<Symbol<3, Chars<..>>, _>` spine.

In the raw output this class is unusual in that the cause **is** the primary error rather than a note
below it: an `E0277` saying a `PathCons<…>` path does not implement the namespace trait. A `Self` that is
a path or a `RedirectLookup` rather than a provider struct always means the lookup failed, not a
provider's dependencies. One landmark reads like a contradiction and is in fact the diagnosis — a `help:`
listing the component *markers* the namespace does resolve, yours among them, tells you the registration
worked and only the leaf binding is missing.

Reached by a method call instead of a check, this hides exactly as any other dependency does.

**The fix** is to bind a provider at the path: a `#[default_impl(@app.GreeterComponent in Namespace)]` on
the provider, a namespace body entry, or a direct line on the context — whichever matches where the
wiring belongs. When the path itself is wrong, reconcile the prefix and the binding so they name the same
one.

## The wiring itself does not compile

These are structural failures. The compiler reports a definite code and points at both offending lines,
so there is no cause to hunt for — the difficulty is only mapping the message back to the wiring
decision.

### Two entries claim one key

The same component wired twice, a generic table overlapping a specific one, an `open` header colliding
with an explicit mapping, or a provider name declared twice. CGP lowers each block independently and
cannot see the collision, so the compiler catches it:

```text
error[E0119]: [CGP-E004] duplicate wiring for component `GreeterComponent` on `Person`
   |
31 |         GreeterComponent: GreetHello,
   |         ---------------- first implementation here
...
38 |         GreeterComponent: GreetGoodbye,
   |         ^^^^^^^^^^^^^^^^ conflicting implementation for `Person`
```

Two carets, two lines to reconcile. Related codes cover the neighbouring shapes: `[CGP-E007]` for an
`open` header colliding with an explicit entry, `[CGP-E008]` for the same key redirected twice.

Two things are worth knowing when reading the raw form. A context-wiring entry generates *two* impls, so
one duplicate prints a **pair** of `E0119`s — one conflict, not two. And a blanket-versus-specific
collision often carries a "downstream crates may implement …" note: that is the compiler explaining that
coherence must allow for impls a future crate could add, not a second problem.

A duplicate **name** rather than a duplicate key is an `E0428` instead, and passes through uncoded — it
already points precisely at both definitions.

### Joining more than one namespace

A context can forward through exactly one namespace. Two `namespace` headers — or one header plus a `for`
loop whose key is bare rather than embedded in a path — each produce a blanket impl covering every key,
and the two overlap:

```text
error[E0119]: [CGP-E006] only one namespace can be used for each target type in `delegate_components!`, but `App` uses both `NamespaceA` and `NamespaceB`
   |
52 |         namespace NamespaceA;
   |                   ---------- first implementation here
53 |         namespace NamespaceB;
   |                   ^^^^^^^^^^ conflicting implementation for `App`
```

**The fix** is inheritance rather than joining: define one namespace that inherits the others
(`cgp_namespace! { new Combined: NamespaceA { … } }`) and join that one. For the `for`-loop form, put the
loop's key inside a path so it keys one route instead of all of them.

### Overriding something a namespace already binds

A namespace's entries are defaults, but only a path it *routes to* without *terminating* can be
overridden. A path the namespace itself binds is already covered by its blanket impl:

```text
error[E0119]: [CGP-E005] `App` cannot wire `@app.GreeterComponent.*` that is already set through `AppNamespace`
   |
55 |         namespace AppNamespace;
   |                   ------------ first implementation here
57 |         @app.GreeterComponent: GreetBye,
   |          ^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `App`
```

**The fix** follows one rule: *a bound namespace entry cannot be overridden.* Leave the leaf path
unclaimed in the namespace so the context can supply it, or change the binding in the namespace itself.

The same mistake one level up — a child namespace redefining a key it inherits — produces a bare
`conflicting implementations of trait ChildNs<_> for type GreeterComponent`, which the tool does not yet
rewrite. The tell is that the conflicting *self type* is a component marker rather than a context. The
fix there is to leave the key unbound in the base so each child can bind it.

### Registering into a namespace you do not own

Rust's orphan rule forbids implementing a foreign trait for a foreign type, and a namespace registration
is exactly such an impl when both the namespace and the key belong to other crates:

```text
error[E0210]: [CGP-E011] cannot register the foreign path `@app.AnnouncerComponent` into the foreign namespace `AppNamespace`
   |
26 | #[cgp_impl(new AnnounceQuietly)]
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: own one end of the wiring: key it on a component defined in this crate, or register it
           from the crate that defines `AppNamespace`
```

**The fix** is in the `help:`: own one end. Key the registration on a local component, register it from
the namespace's own crate, or — to extend a foreign namespace — define a new local namespace that
*inherits* it rather than re-opening it, which is orphan-safe because every emitted impl is then for your
own trait.

### A lookup that never terminates

Wiring a component to [`UseContext`](./providers/use_context.md) when the context's only source of that
component *is* that wiring makes the lookup chase its own tail:

```text
error[E0275]: [CGP-E010] the wiring for the consumer trait `CanGreet` on context `Person` never resolves — the lookup recurses without terminating
   |
   = help: a lookup that recurses usually means the component is delegated back to the context itself —
           e.g. wired to `UseContext` with no direct implementation of the consumer trait — so wire a
           real provider, or implement the consumer trait directly on the context
```

Raw `rustc` gives an `E0275` overflow with a note chain you can read the loop out of, plus a stock
"consider increasing the recursion limit" suggestion. **Ignore that suggestion**: a true cycle does not
terminate at any depth, so no limit is high enough.

Reached by a method call rather than a check, a cycle does not overflow at all — it hides as the ordinary
`E0599`, with nothing to say a cycle was the reason.

**A related cycle sits at namespace definitions rather than at a use site.** Namespaces whose parent
chain loops (`A: B` with `B: A`, or `A: A`) fail *eagerly*, with an `E0275` on each `cgp_namespace!`
block, because the cycle lives in the `where` clause of a generated impl the compiler checks on sight.
Two things are unusual about it. Under `cargo cgp check`'s next-generation solver the code **compiles
clean** and nothing is reported — the one class where the tool shows you *less* than plain `cargo check`.
And the wiring is wrong regardless: break the loop so the parent chain forms a tree, factoring shared
entries into a third base namespace both inherit from.

### A generic that reaches nothing

A per-entry generic must reach the *key*, where `DelegateComponent<Key<…>>` binds it. One that appears
only in the provider value binds nothing:

```text
error[E0207]: the type parameter `T` is not constrained by the impl trait, self type, or predicates
   |
39 |         <T> GreeterComponent: GreetWith<T>,
   |          ^ unconstrained type parameter
```

This passes through unrewritten, and it does not need much — the caret is on the parameter. The message
speaks in impl terms, though, where the actionable reading is a wiring one: **the generic has to appear
in the key** (`<T> SomeKey<T>: …`), not only in the value. Expect the same error twice, once per
generated position; both name the same `<T>`.

## A macro lowered something the compiler rejects

Here the failure is not in the wiring at all but in what a macro turned your input into. These are
generally the clearest errors in the catalog, and most pass through unrewritten because the raw message
already leads with the cause.

### An imported type that does not exist

[`#[use_type]`](./attributes/use_type.md) works by substitution and cannot see which associated types a
trait declares, so a wrong name is lowered faithfully and caught by name resolution:

```text
error[E0576]: cannot find associated type `Scalr` in trait `HasScalarType`
   |
21 |     type Scalar;
   |     ------------ similarly named associated type `Scalar` defined here
...
26 | pub fn get_scalar(&self) -> Scalr {
   |                             ^^^^^
```

The caret lands on the name *you* wrote — the macro copies the identifier's span onto the rewritten path
— and the `help:` usually supplies the spelling. Nothing to decode.

### A name the generated code cannot see

`#[impl_generics]` adds a parameter to the generated *impl* alone, so the generated trait cannot see it.
Naming it in the signature, which stays on the trait, refers to nothing:

```text
error[E0433]: cannot find type `Db` in this scope
   |
37 | pub fn fetch_row(&self, #[implicit] database: &Pool<Db>) -> Db::Row {
   |                                                             ^^ use of undeclared type `Db`
```

The `&Pool<Db>` argument is fine — that one is stripped into a field bound on the impl, where `Db` *is* in
scope. The `Db::Row` return type is not. **The fix** is a choice the macro cannot make for you: promote
the type to an [abstract type](./macros/cgp_type.md) so it can be named everywhere, or keep
`#[impl_generics]` and stop naming it. A bare unresolved name with no path context reports as `E0425`
instead, so grep for both.

A sibling has the opposite cause — the name resolves, to the wrong thing. Naming an abstract type after
the trait that bounds it makes the bound resolve to the type being declared:

```text
error[E0404]: expected trait, found type parameter `Database`
   |
22 | pub trait Database: Sized {
   |           -------- you might have meant to refer to this trait
...
34 |     type Database: Database;
   |          --------  ^^^^^^^^ not a trait
```

Both halves are named, so the message explains itself once read. Name them apart — `Db` reads better for
an abstract database type anyway.

### A field-type shorthand with no rule

The getter and implicit-argument macros recognize a small set of field-type shorthands — `Option<&T>`,
`&[T]`, `&str`. A *combination* they have no rule for is lowered by whichever rule matches first, and the
result can be a type that is not well-formed. `Option<&[u8]>` lowers to a bound naming the unsized
`Option<[u8]>`:

```text
error[E0277]: the size for values of type `[u8]` cannot be known at compilation time
   |
18 | #[cgp_auto_getter]
   | ^^^^^^^^^^^^^^^^^^ doesn't have a size known at compile-time
   |
   = help: the trait `Sized` is not implemented for `[u8]`
note: required by an implicit `Sized` bound in `Option`
```

Two things make this harder to read than it looks. The caret is on the macro attribute rather than on any
type you wrote, because the offending type is synthesized. And a second `E0599` on `as_ref` usually
follows — derived noise from the same unsized type, pointing at nothing new.

**The fix** is a shape a single shorthand supports: `Option<Vec<u8>>`, or `Option<&'static [u8]>`, or a
hand-written getter naming whatever type the field actually has. The shorthands compose only where the
result is `Sized`.

## Using something you did not declare

A group with one cause: the body of a [`#[cgp_fn]`](./macros/cgp_fn.md) or
[`#[cgp_impl]`](./macros/cgp_impl.md) becomes a blanket impl over a generated context parameter, so
anything the body calls must be *declared* as a bound on it. Rust's raw errors here are unhelpful because
they talk about that generated parameter; the tool names the missing declaration instead.

**A capability used without `#[uses]`:**

```text
error[E0599]: [CGP-E012] the capability `GetCount` is used but not declared as a dependency
   |
44 |     format!("{} ({})", self.get_name(), self.get_count())
   |                                              ^^^^^^^^^
   |
   = help: declare it as a dependency with `#[uses(GetCount)]`
```

**An inner provider used without `#[use_provider]`:**

```text
error[E0599]: [CGP-E016] the inner provider `InnerCalculator` is used but not imported
   |
22 |         InnerCalculator::area(self) * scale_factor * scale_factor
   |                          ^^^^
   |
   = help: import it with `#[use_provider(InnerCalculator: AreaCalculator)]`
```

In both cases rustc's own suggestion is worse than useless: it leaks the generated `__Context__`
parameter, and for the second it proposes the *consumer* trait, which is the wrong fix.

### Naming the wrong half of a component

A component has two traits, and a provider implements the *provider* one. Writing the consumer trait
where the provider trait belongs gives it one generic argument too many, because the macro inserts the
context:

```text
error[E0107]: [CGP-E013] `CanCalculateArea` is a consumer trait, but a `#[cgp_impl]` provider must implement its provider trait `AreaCalculator`
  |
9 | impl CanCalculateArea {
  |      ^^^^^^^^^^^^^^^^
  |
  = help: change the impl header to target the provider trait: `impl AreaCalculator` (not `impl CanCalculateArea`)
```

Untranslated, this one mistake produces a burst of `E0425`/`E0107`/`E0186`/`E0207` errors plus a
downstream check failure, none of which names the cause. Two neighbours share the shape: `[CGP-E015]` is
the same confusion in a `#[use_provider]` bound, and `[CGP-E014]` is `#[cgp_impl]` applied to a trait that
is not a component at all — where the fix is to make it one, or to write a plain `impl`.

## The `[CGP-Exxx]` codes

A code tags one class of CGP mistake. The compiler's own code is always kept beside it, so
`rustc --explain` still works and nothing is reclassified away from rustc.

Codes appear in two places and mean different things. A code in the **headline** classifies the error; a
code in the **root-cause tree** labels one link in the chain.

### Headline codes

| Code | Means |
|---|---|
| `CGP-E001` | A context does not implement a consumer trait |
| `CGP-E002` | A provider does not implement its provider trait for the context |
| `CGP-E003` | A field is present but has the wrong type |
| `CGP-E004` | The same key wired twice |
| `CGP-E005` | Wiring a key a namespace already binds |
| `CGP-E006` | More than one namespace joined on one context |
| `CGP-E007` | An `open` redirect colliding with an explicit entry |
| `CGP-E008` | The same key redirected twice |
| `CGP-E009` | A non-component trait — a wrapper, or a `#[cgp_fn]` capability — blocked by a CGP dependency |
| `CGP-E010` | The wiring recurses without terminating |
| `CGP-E011` | An orphan-rule namespace registration |
| `CGP-E012` | A capability used but not declared with `#[uses]` |
| `CGP-E013` | A consumer trait named in a provider impl header |
| `CGP-E014` | `#[cgp_impl]` applied to a trait that is not a component |
| `CGP-E015` | A consumer trait named in an inner-provider bound |
| `CGP-E016` | An inner provider used but not imported |
| `CGP-E017` | An abstract type wired differently from what a provider pinned |

### Dependency-tree codes

| Code | Means |
|---|---|
| `CGP-E101` | A hop through a context's consumer-trait impl |
| `CGP-E102` | A hop through a provider's provider-trait impl |
| `CGP-E104` | A hop through a namespace or `open` redirect |
| `CGP-E105` | A hop through any other trait |
| `CGP-E106` | **Leaf:** a field is genuinely absent |
| `CGP-E107` | **Leaf:** the context wires nothing for a component or path |
| `CGP-E108` | **Leaf:** the field exists but `#[derive(HasField)]` is missing |
| `CGP-E109` | **Leaf:** the field has the wrong type |
| `CGP-E110` | **Leaf:** a provider's own dispatch table is missing an entry |
| `CGP-E111` | **Leaf:** something wired where a provider belongs is not a provider |
| `CGP-E112` | **Leaf:** an associated type differs from what was required |
| `CGP-E201` | The root-cause lead for an ordinary Rust trait bound |

A tree entry that merely passes a non-CGP message through in rustc's own words is left uncoded.
`CGP-E103` was retired rather than reassigned, so the rest stay stable.

## Related constructs

- [`check_components!`](./macros/check_components.md) — turns a hidden failure into a surfaced one, and
  the first thing to reach for.
- [`delegate_components!`](./macros/delegate_components.md) — the wiring nearly every error here is about.
- [`IsProviderFor`](./traits/is_provider_for.md) — the marker that carries a provider's requirements into
  a diagnostic; the reason a checked error names the cause at all.
- [`CanUseComponent`](./traits/can_use_component.md) — what a check asserts.
- [`cargo cgp check`](/docs/cargo-cgp/check) — the tool every rewritten output on this page came from.
- [`Symbol!`](./macros/symbol.md) — the type-level field name whose raw `Chars<…>` spine the tool
  resugars.

The ideas behind it:

- [Checking your wiring](/docs/concepts/check-traits) — why wiring is lazy, and one broken context
  followed through unchecked, checked, and checked through the toolchain.
- [Bypassing coherence](/docs/concepts/coherence) — the mechanism behind the `E0119` and orphan-rule
  failures above.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
