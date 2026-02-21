---
slug: extensible-datatypes-part-2
title: 'Programming Extensible Data Types in Rust with CGP - Part 2: Modular Interpreters and Extensible Visitors'
authors: [soares]
tags: [deepdive]
---

This is the **second** part of the blog series on **Programming Extensible Data Types in Rust with CGP**. You can read the [first part here](/blog/extensible-datatypes-part-1).

In this second part of the series, we will explore the use of **extensible variants**, by examining how it can be used in an **extensible visitor pattern** to build a modular interpreter for a toy math expression language.

<!-- truncate -->

## Recap

As a recap, we have covered the new release of [**CGP v0.4.2**](https://github.com/contextgeneric/cgp/releases/tag/v0.4.2) which now supports the use of **extensible records and variants**, allowing developers to write code that operates on *any struct containing specific fields* or *any enum containing specific variants*, without needing their concrete definition.

In the first part of the series, [**Modular App Construction and Extensible Builders**](/blog/extensible-datatypes-part-1), we demonstrated an example use of the **extensible builder pattern**, which uses **extensible records** to support modular construction of an application context.

### Discussion

Discuss on [Reddit](https://www.reddit.com/r/rust/comments/1lvgyre/building_modular_interpreters_and_visitors_in/), [GitHub](https://github.com/orgs/contextgeneric/discussions/13) or [Discord](https://discord.gg/Hgk3rCw6pQ).

## Series Overview

[**Part 1: Modular App Construction and Extensible Builders**](/blog/extensible-datatypes-part-1) – In this introductory part, we present a high-level overview of the key features enabled by extensible data types. We then dive into a hands-on demonstration showing how extensible records can be used to build and compose modular builders for real-world applications.

**Part 2: Modular Interpreters and Extensible Visitors** (this post) – This part continues the demonstration by introducing extensible variants. We use them to address the [**expression problem**](https://en.wikipedia.org/wiki/Expression_problem), implementing a set of reusable interpreter components for a small toy language.

[**Part 3: Implementing Extensible Records**](/blog/extensible-datatypes-part-3) – Here, we walk through the internal mechanics behind extensible records. We show how CGP supports the modular builder pattern demonstrated in Part 1 through its underlying type and trait machinery.

[**Part 4: Implementing Extensible Variants**](/blog/extensible-datatypes-part-4) – This part mirrors Part 3, but for extensible variants. We examine how extensible variants are implemented, and compare the differences and similarities between extensible records and variants.

## Extending the Visitor Pattern

Earlier, we explored how CGP uses the extensible builder pattern to enable modular construction of context structs. In this article, we will see how a similar approach can be applied to context **enums**, allowing each variant to be destructured and handled by a flexible, composable set of handlers.

In Rust and many object-oriented languages, this pattern is commonly referred to as the [**visitor pattern**](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html). However, Rust’s powerful `enum` and `match` features often reduce the need for the visitor pattern, especially when the concrete enum type is known. In such cases, developers can simply use `match` expressions to handle each variant explicitly and concisely.

Despite this, the visitor pattern remains useful in situations where the concrete enum type is **unknown** or abstracted away. This is especially true in libraries like [`serde`](https://docs.rs/serde/latest/serde/de/trait.Visitor.html) and [`syn`](https://docs.rs/syn/latest/syn/visit/trait.Visit.html), where visitors are used to traverse abstract syntax trees or serialization payloads without tying the implementation to a specific format or structure. For instance, in `serde`, deserialization is driven by a visitor provided by the target type, which walks through structures like JSON or TOML without coupling the deserializer to any specific data format.

### Limitations of the Traditional Visitor Pattern

While the visitor pattern is useful, it suffers from a major drawback: it is inherently **closed for extension**. All possible variants and visitable types must be declared upfront in the visitor interface, and it is challenging to add or remove variants later without breaking existing implementations.

For example, consider the `Visitor` trait in `serde`, which defines methods for visiting a fixed set of primitive types — up to 128-bit integers. If a developer wants to deserialize a type that contains a [`U256`](https://docs.rs/primitive-types/latest/primitive_types/struct.U256.html) value, there is no way to extend the `Visitor` trait to support native 256-bit integers. Likewise, if someone builds a new serialization format that introduces support for such a type, it cannot cleanly integrate with `serde` because the trait cannot be expanded.

To work around this, `serde` includes a broad set of 26 visitor methods in its core `Visitor` trait to accommodate a wide range of cases. However, this introduces the opposite problem: when a serialization format **does not** support a specific visitor method, the only option is to return a **runtime error**. There is no way to signal at compile time that a type is incompatible with the format, even if it formally implements `Serialize` or `Deserialize`.

This mismatch becomes especially noticeable when using compact formats like [`postcard`](https://docs.rs/postcard) or [`bincode`](https://docs.rs/bincode), which support only a small subset of types compared to JSON. These libraries accept any type implementing `Deserialize`, but compatibility is only verified at runtime — leaving users to discover format mismatches through runtime errors instead of compile-time errors.

In short, the traditional visitor pattern tends to be either too **restrictive** (by enforcing a closed set of operations) or too **permissive** (by relying on runtime errors to reject unsupported operations). What’s needed is a more flexible, composable alternative — one that allows both sides (visitor and visitee) to express precise requirements at compile time.

This is exactly the problem that the **extensible visitor pattern** in CGP aims to solve. It enables open-ended, modular visitors that preserve **type safety** and **extensibility**, without the pitfalls of runtime errors or rigid interfaces.

### The Expression Problem

While it’s theoretically possible to replace `serde`’s visitor pattern with CGP’s extensible alternative, doing so would require significant refactoring and is outside the scope of this post. Instead, we’ll explore a *simpler* but well-known challenge that illustrates the same limitations: the [**expression problem**](https://en.wikipedia.org/wiki/Expression_problem).

Suppose we want to implement an interpreter for a toy arithmetic language in Rust. This language might support basic math expressions like `1 + (2 * 3)`. A typical way to represent such a language is with an enum like this:

```rust
pub enum MathExpr {
    Literal(u64),
    Plus(Box<MathExpr>, Box<MathExpr>),
    Times(Box<MathExpr>, Box<MathExpr>),
}
```

Here, `MathExpr` represents arithmetic expressions. The `Plus` and `Times` variants contain boxed sub-expressions, and `Literal` holds an integer value. The use of `Box` is necessary due to Rust’s size constraints for recursive data structures.

To evaluate these expressions, we can implement a straightforward `eval` function:

```rust
pub fn eval(expr: MathExpr) -> u64 {
    match expr {
        MathExpr::Literal(value) => value,
        MathExpr::Plus(a, b) => eval(*a) + eval(*b),
        MathExpr::Times(a, b) => eval(*a) * eval(*b),
    }
}
```

This works well for small examples. But real-world interpreters quickly grow in complexity. Each evaluation case might span dozens — or hundreds — of lines of code. Additionally, the enum itself might have many more variants. For example, [`syn::Expr`](https://docs.rs/syn/latest/syn/enum.Expr.html), a real-world expression type for Rust, defines over *40 variants*.

Let’s assume our toy `MathExpr` is similarly complex. Now imagine that alongside `eval`, we also want to define other operations, like pretty-printing:

```rust
pub fn expr_to_string(expr: &MathExpr) -> String {
    match expr {
        MathExpr::Literal(value) => value.to_string(),
        MathExpr::Plus(a, b) => format!("({} + {})", expr_to_string(a), expr_to_string(b)),
        MathExpr::Times(a, b) => format!("({} * {})", expr_to_string(a), expr_to_string(b)),
    }
}
```

Here lies the crux of the expression problem: as the language evolves, we frequently need to *add* new expression variants or *remove* old ones. But any modification to the `MathExpr` enum forces us to update *all* pattern-matching functions like `eval`, `expr_to_string`, and others. The enum becomes **tightly coupled** to every function that consumes it.

Worse, this coupling is not easy to break. The recursive nature of `MathExpr` — where variants like `Plus` contain other `MathExpr` values — means even modular helper functions (e.g., `eval_plus`) must still operate on `MathExpr`, perpetuating the tight dependency.

This isn’t a problem unique to interpreters. Many recursive data structures — like JSON `Value` types — suffer from similar issues. A JSON object may contain maps of nested `Value`s, making any function over the type deeply tied to its structure.

Because of this, extending or experimenting with the enum often becomes burdensome. If the type is part of an upstream crate, users may need to submit a **pull request** just to add a variant. And if the maintainer *declines*, downstream users may be forced to **fork** the crate just to gain the flexibility they need.

In the next section, we’ll explore how CGP's extensible visitor pattern addresses this problem — by **decoupling** the implementation of each variant from the concrete enum definition.

## Evaluator Computer

To demonstrate how CGP enables extensible and decoupled evaluation logic, we will now walk through how to implement a small part of the `eval` function — specifically, the logic for handling the `Plus` operator. Rather than tying ourselves to a fixed `MathExpr` enum, we begin by defining `Plus` as an independent struct:

```rust
pub struct Plus<Expr> {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}
```

In this definition, `Plus` is no longer a variant of a hardcoded enum. Instead, it is a *generic* data structure that takes an `Expr` type parameter. This parameter represents the broader expression type and allows `Plus` to be reused in many different expression trees. The `left` and `right` operands are wrapped in `Box` to support recursive structures while still satisfying Rust’s size requirements later on.

To actually evaluate such a sub-expression, CGP introduces the concept of a **Computer** — a CGP component designed for pure computation. It is defined as follows:

```rust
#[cgp_component(Computer)]
pub trait CanCompute<Code, Input> {
    type Output;

    fn compute(&self, _code: PhantomData<Code>, input: Input) -> Self::Output;
}
```

This trait behaves similarly to the `Handler` trait introduced earlier, but with one key distinction: `compute` is a *synchronous* function and does not return a `Result`. It is called a *computer* because it embodies a pure, deterministic *computation* that transforms input into output.

The `Computer` trait serves as the foundation for extensible evaluation. It abstracts the idea of computation away from any specific expression type or evaluation strategy. Using this abstraction, we can implement evaluation logic for each sub-expression in isolation. For example, here is how we define a provider for evaluating the `Plus` struct:

```rust
#[cgp_new_provider]
impl<Context, Code, MathExpr, Output> Computer<Context, Code, Plus<MathExpr>> for EvalAdd
where
    Context: CanCompute<Code, MathExpr, Output = Output>,
    Output: Add<Output = Output>,
{
    type Output = Output;

    fn compute(
        context: &Context,
        code: PhantomData<Code>,
        Plus { left, right }: Plus<MathExpr>,
    ) -> Self::Output {
        let output_a = context.compute(code, *left);
        let output_b = context.compute(code, *right);

        output_a + output_b
    }
}
```

This implementation defines `EvalAdd` as a `Computer` with `Plus<MathExpr>` as input. It works generically over any `Context`, `Code`, and `MathExpr` type, as long as the context knows how to compute `MathExpr` and the resulting output type supports the `Add` trait. In other words, the context must be able to evaluate each operand, and the results must be addable.

By using `context.compute(...)` recursively on the left and right operands, we evaluate each sub-expression and then add the results together. This setup allows us to write clean, modular logic that does not assume anything about the shape of the expression tree or the numeric type being used.

The same approach applies to other arithmetic operations. For example, we can implement a provider for multiplication as follows:

```rust
pub struct Times<Expr> {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[cgp_new_provider]
impl<Context, Code, MathExpr, Output> Computer<Context, Code, Times<MathExpr>> for EvalMultiply
where
    Context: CanCompute<Code, MathExpr, Output = Output>,
    Output: Mul<Output = Output>,
{
    type Output = Output;

    fn compute(
        context: &Context,
        code: PhantomData<Code>,
        Times { left, right }: Times<MathExpr>,
    ) -> Output {
        let output_a = context.compute(code, *left);
        let output_b = context.compute(code, *right);

        output_a * output_b
    }
}
```

Here, we follow the exact same structure. The only difference is that we require the output type to implement `Mul` instead of `Add`, and we use the `*` operator to combine the results.

Finally, we handle literals using the following straightforward implementation:

```rust
pub struct Literal<T>(pub T);

#[cgp_new_provider]
impl<Context, Code, T> Computer<Context, Code, Literal<T>> for EvalLiteral {
    type Output = T;

    fn compute(_context: &Context, _code: PhantomData<Code>, Literal(value): Literal<T>) -> T {
        value
    }
}
```

The `EvalLiteral` provider simply returns the inner value. It doesn’t depend on any context or recursive evaluation, making it the simplest form of a computer.

What’s remarkable about this setup is how each of the providers — `EvalAdd`, `EvalMultiply`, and `EvalLiteral` — is completely **decoupled** from both each other and the concrete expression type. These components can live in separate crates or modules and still be composed together to form a complete evaluator.

This modularity is precisely the power that CGP brings to the table. Instead of forcing every part of your evaluator into a rigid, monolithic structure, you can build each piece independently and combine them later. The result is not only cleaner and more maintainable code, but also an evaluation engine that is fully open for extension — without giving up on compile-time guarantees.

### Evaluating Concrete Expressions

With our evaluation logic defined for individual expression types like `Plus`, `Times`, and `Literal`, the next step is to bring everything together into a fully functional evaluator. To do this, we first define a concrete expression type named `MathExpr`:

```rust
#[derive(Debug, HasFields, FromVariant, ExtractField)]
pub enum MathExpr {
    Plus(Plus<MathExpr>),
    Times(Times<MathExpr>),
    Literal(Literal<u64>),
}
```

Notice that instead of embedding the structure of each expression variant directly inside the enum, we define each variant to wrap one of the standalone structs we previously created. This design is intentional. By keeping each sub-expression — such as `Plus` and `Times` — as its own modular type, we can reuse and compose them in more flexible ways. To complete the recursive structure, we instantiate the generic type parameter `MathExpr` inside each sub-expression, allowing the expression tree to contain arbitrarily nested expressions. For the `Literal` case, we keep things simple by hardcoding the value type to `u64`.

Now that we have our enum, we need to define the context in which evaluation will happen. In CGP, this is done using a `#[cgp_context]` declaration:

```rust
#[cgp_context]
pub struct Interpreter;
```

This `Interpreter` struct will serve as the central context object for our evaluator. In this particular example, we do not require the context to hold any runtime data, so the struct is left empty. Its only purpose is to act as a compile-time container that wires together the correct provider implementations.

The actual wiring is handled through CGP’s powerful delegation system, which allows us to map input types to their corresponding computation logic. Here's how we set it up:

```rust
delegate_components! {
    InterpreterComponents {
        ComputerComponent:
            UseInputDelegate<
                new EvalComponents {
                    Expr: DispatchEval,
                    Plus<MathExpr>: EvalAdd,
                    Times<MathExpr>: EvalMultiply,
                    Literal<u64>: EvalLiteral,
                }
            >,
    }
}
```

In this block, we define the `InterpreterComponents` wiring configuration, which tells CGP how to resolve the `Computer` trait for each expression type. At the heart of this configuration is `UseInputDelegate`, a [**generic dispatcher**](/blog/hypershell-release/#input-based-dispatch) that automatically selects the correct provider based on the input type at compile time.

This dispatcher operates over the inner `EvalComponents` table, which we create on the fly using the `new` keyword. This `EvalComponents` struct maps each expression type to its associated computation provider:

* `MathExpr` is mapped to `DispatchEval`, which acts as a dispatcher that dispatches to one of the sub-expression types based on the variant.
* `Plus<MathExpr>` is evaluated using `EvalAdd`.
* `Times<MathExpr>` is evaluated using `EvalMultiply`.
* `Literal<u64>` is handled by `EvalLiteral`.

Because `UseInputDelegate` operates at the type level, the entire dispatch process is fully type-safe and resolved at compile time. There is no need for `match` statements, no runtime type checks, and no boilerplate glue code. The trait system simply composes itself from the parts we’ve defined.

### Dispatching Eval

With the components for evaluating individual sub-expressions in place, we now turn our attention to the final piece of the puzzle: evaluating the main `MathExpr` enum itself. To accomplish this, we delegate the `MathExpr` type to a special provider named `DispatchEval`, which is defined alongside the `Interpreter` context like so:

```rust
#[cgp_new_provider]
impl<Code> Computer<Interpreter, Code, MathExpr> for DispatchEval {
    type Output = u64;

    fn compute(context: &Interpreter, code: PhantomData<Code>, expr: MathExpr) -> Self::Output {
        <MatchWithValueHandlers>::compute(context, code, expr)
    }
}
```

Here, `DispatchEval` is implemented as a *context-specific* provider. That means it only applies when we are evaluating expressions in the `Interpreter` context, and it handles the concrete `MathExpr` enum as input. Rather than directly writing out how each variant of the enum is evaluated, we delegate that responsibility to a special **visitor dispatcher** called `MatchWithValueHandlers`.

This dispatcher is one of the key tools provided by CGP. It automatically maps each enum variant to the appropriate computation provider we registered earlier in `EvalComponents`. In effect, `MatchWithValueHandlers` performs dispatch on the matching of variants at compile time. The implementation of `DispatchEval` is simply a wrapper around this dispatcher, but that wrapper plays a crucial role.

So why do we need this wrapper in the first place? It comes down to a subtle limitation in Rust’s trait resolution system. If we try to directly wire the `Computer` handler for `MathExpr` to `MatchWithValueHandlers`, the compiler runs into a cyclic dependency: to implement the trait, it needs to evaluate the variant-specific providers like `EvalAdd`, which themselves rely on `MatchWithValueHandlers`. The result is a cryptic “overflowing requirements” error.

By inserting this wrapper layer with `DispatchEval`, we sidestep that issue. Rust is able to mark the trait as implemented before diving into the body of the method, effectively breaking the cycle.

To understand what `MatchWithValueHandlers` is doing under the hood, imagine manually writing out the dispatch logic like this:

```rust
#[cgp_new_provider]
impl<Code> Computer<Interpreter, Code, MathExpr> for DispatchEval {
    type Output = u64;

    fn compute(context: &Interpreter, code: PhantomData<Code>, expr: MathExpr) -> Self::Output {
        match expr {
            Expr::Plus(expr) => context.compute(code, expr),
            Expr::Times(expr) => context.compute(code, expr),
            Expr::Literal(expr) => context.compute(code, expr),
        }
    }
}
```

This is straightforward for a small enum like ours. But once your enum grows beyond a few variants — as is the case with something like `syn::Expr`, which contains over 40 — you quickly run into repetition, verbosity, and maintenance pain.

`MatchWithValueHandlers` avoids all that by performing this logic *generically*. It doesn't rely on macros or hardcoded pattern matching. Instead, it works entirely through traits and type-level programming. That means the same dispatcher can be reused for any enum type that satisfies the required constraints, without knowing anything about the actual enum variants ahead of time.

This is a significant benefit over traditional macro-based approaches, which are more difficult to reason about, harder to debug, and often tightly coupled to specific enum definitions. With CGP, you get a reusable, type-safe visitor implementation that scales cleanly as your codebase grows.

In short, `DispatchEval` and `MatchWithValueHandlers` together make it possible to evaluate complex enums in a clean, declarative, and extensible way — without writing repetitive boilerplate or giving up compile-time guarantees. It’s another example of how CGP turns what would normally be painful and manual trait implementations into something elegant and maintainable.

## Converting to a Lisp Expression

At this point, we’ve implemented a basic arithmetic evaluator using CGP. But interpreting expressions is only one of many possible operations we might want to perform. Often, we want to **transform** the syntax tree — say, converting it into a string, generating code, or emitting tokens for macro expansion.

Although a plain `to_string` implementation could be a compelling use case on its own, it might seem too trivial to justify CGP’s involvement (spoiler: it’s not). So instead, to make things a little more illustrative and practical, we’ll convert our arithmetic expressions into **Lisp expressions** — specifically, into a form inspired by [S-expressions](https://en.wikipedia.org/wiki/S-expression).

### Why Lisp?

The motivation here is similar to the real-world task of converting a Rust syntax tree (like [`syn::Expr`](https://docs.rs/syn/latest/syn/enum.Expr.html)) into a [`TokenStream`](https://docs.rs/proc-macro2/latest/proc_macro2/struct.TokenStream.html). That task typically requires walking a rich enum structure and transforming it into a stream of tokens. Rather than deal with the full complexity of `TokenStream`, we’ll use a simplified representation based on Lisp syntax — concise, nested, and familiar to anyone who’s seen prefix notation.

For example, our arithmetic expression `1 + (2 * 3)` would become the Lisp-like expression: `(+ 1 (* 2 3))`.

To represent this form, we define a general-purpose enum:

```rust
#[derive(HasFields, FromVariant, ExtractField)]
pub enum LispExpr {
    List(List<LispExpr>),
    Literal(Literal<u64>),
    Ident(Ident),
}

pub struct List<Expr>(pub Vec<Box<Expr>>);
pub struct Ident(pub String);
pub struct Literal<T>(pub T);
```

This `LispExpr` enum is broader than our original `MathExpr` — it can represent not just arithmetic but more general symbolic forms. Each `List` is a vector of boxed sub-expressions; `Literal` holds numeric values; and `Ident` wraps identifiers like `"+"` or `"*"`. For simplicity, we use a `Vec` instead of a linked list.

We can manually construct the equivalent of `(+ 1 (* 2 3))` like this:

```rust
let lisp_expr = LispExpr::List(List(vec![
    LispExpr::Ident(Ident("+".to_owned())).into(),
    LispExpr::Literal(Literal(1)).into(),
    LispExpr::List(List(vec![
        LispExpr::Ident(Ident("*".to_owned())).into(),
        LispExpr::Literal(Literal(2)).into(),
        LispExpr::Literal(Literal(3)).into(),
    ])).into()
]));
```

This demonstrates the basic structure. But the real point is this: converting from `Expr` to `LispExpr` **is itself another instance of the expression problem**, just like evaluation. In fact, it's even more subtle — this is a **"double expression problem"**: we want to decouple our logic from both the *source expression type* (`MathExpr`) and the *target type* (`LispExpr`).

So how do we solve it modularly?

### The `ComputerRef` Component

To implement this modular conversion, we’ll use a slightly different CGP trait: `ComputerRef`.

```rust
#[cgp_component(ComputerRef)]
pub trait CanComputeRef<Code, Input> {
    type Output;

    fn compute_ref(&self, _code: PhantomData<Code>, input: &Input) -> Self::Output;
}
```

`ComputerRef` is similar to `Computer`, but it takes a **reference** to the input rather than consuming it. This is especially useful in our case, because we might want to evaluate the expression again after transforming it — something we couldn’t do if we moved it.

While we *could* clone the input or use higher-ranked trait bounds to handle references, `ComputerRef` offers a cleaner, more ergonomic solution. CGP also provides promotion adapters that allow `ComputerRef` implementations to act as `Computer` providers when needed. So the two traits are often interchangeable in practice — use the one that fits your borrowing needs.

For the example, we used `Computer` to implement evaluation, but `ComputerRef` for to-Lisp transformation, to demonstrate the use of both traits. In practice, you might want to use `ComputerRef` for evaluation as well, so that the same expression can still be reused after evaluation.

### Implementing `PlusToLisp`

With our expression types and Lisp target representation in place, we can now implement a CGP provider that transforms a `Plus` expression into its corresponding Lisp representation. Here's what the provider looks like:

```rust
#[cgp_new_provider]
impl<Context, Code, MathExpr, LispExpr> ComputerRef<Context, Code, Plus<MathExpr>> for PlusToLisp
where
    Context:
        HasLispExprType<LispExpr = LispExpr> + CanComputeRef<Code, MathExpr, Output = LispExpr>,
    LispSubExpr<LispExpr>: CanUpcast<LispExpr>,
{
    type Output = LispExpr;

    fn compute_ref(
        context: &Context,
        code: PhantomData<Code>,
        Plus { left, right }: &Plus<MathExpr>,
    ) -> Self::Output {
        let expr_a = context.compute_ref(code, left);
        let expr_b = context.compute_ref(code, right);
        let ident = LispSubExpr::Ident(Ident("+".to_owned())).upcast(PhantomData);

        LispSubExpr::List(List(vec![ident.into(), expr_a.into(), expr_b.into()]))
            .upcast(PhantomData)
    }
}
```

This implementation takes a `Plus<MathExpr>` as input and returns a `LispExpr` as output. The transformation is recursive: each subexpression is converted by delegating to the same `ComputerRef` trait for `MathExpr`. The resulting `LispExpr` values are then combined into a list, with the `"+"` operator represented as an identifier at the head.

Notice that the provider is generic over both the context and the code. It requires that the context knows how to evaluate an `MathExpr` into a `LispExpr`, and that it defines a concrete type for `LispExpr`. This is done via a CGP type trait called `HasLispExprType`:

```rust
#[cgp_type]
pub trait HasLispExprType {
    type LispExpr;
}
```

By relying on this trait, we avoid hardcoding the `LispExpr` type directly into the provider. Instead, the actual type can be supplied later when we wire everything together.

### Constructing Variants with Sub-Enums

While we want to construct a `LispExpr` as the final result, we do not necessarily need access to all of its variants inside this provider. In fact, for converting a `Plus` node, we only need to construct two specific kinds of `LispExpr`: a `List`, and an `Ident` representing `"+"`.

To express this more precisely, we define a *local* enum called `LispSubExpr`:

```rust
#[derive(HasFields, ExtractField, FromVariant)]
enum LispSubExpr<LispExpr> {
    List(List<LispExpr>),
    Ident(Ident),
}
```

This `LispSubExpr` enum includes only the subset of variants required to construct a `Plus` expression in Lisp form. It excludes other variants like `Literal`, which may be needed by other parts of the transformation but are not relevant here. Even though `LispSubExpr` is a reduced version of `LispExpr`, it is still parameterized by the full `LispExpr` type, so that the elements in the list can recursively represent complete expressions.

To use `LispSubExpr` in our transformation, we need a way to convert — or more precisely, *upcast* — from this smaller enum into the full `LispExpr`. This is made possible by implementing the `CanUpcast` trait we [introduced earlier](/blog/extensible-datatypes-part-1#safe-enum-upcasting), which is implemented automatically when we annotate the enum with `#[derive(HasFields, ExtractField, FromVariant)]`. This gives us a safe and type-checked way to promote the constructed value into the broader type expected by the rest of the system.

Inside the method body, we first compute the Lisp representations of the two sub-expressions. Then we create an identifier for the `"+"` symbol and upcast it to `LispExpr`. Finally, we build a `List` containing the operator followed by the two operands, and upcast that list into the final `LispExpr` result.

This pattern demonstrates how CGP’s upcasting mechanism makes it easy to construct enum values in a modular and flexible way. Instead of requiring full knowledge of the target enum’s structure, we work with a small, purpose-specific subset. This keeps our providers focused and easier to reason about, while still interoperating cleanly with the larger system.

In essence, `LispSubExpr` plays a role similar to what `#[cgp_auto_getter]` do for structs in CGP. Just as `#[cgp_auto_getter]` lets you **read** fields from a struct through a derived trait without knowing the whole type, `CanUpcast` lets you **construct** parts of an enum using only the variants you care about — without being tied to the entire definition of the enum.

### Implementing `LiteralToLisp`

The implementation of `TimesToLisp` follows the same pattern as `PlusToLisp`, differing only in that it constructs the `"*"` identifier instead of `"+"`. Since the structure is nearly identical, we will focus instead on a more interesting case: converting literal values into their Lisp representation.

The transformation of a literal is handled by the `LiteralToLisp` provider, which implements the `ComputerRef` trait. The core idea here is to wrap the literal value in a Lisp-compatible enum variant and return it as the final result. Here's the implementation:

```rust
#[cgp_new_provider]
impl<Context, Code, T, LispExpr> ComputerRef<Context, Code, Literal<T>> for LiteralToLisp
where
    Context: HasLispExprType<LispExpr = LispExpr>,
    LispSubExpr<T>: CanUpcast<LispExpr>,
    T: Clone,
{
    type Output = LispExpr;

    fn compute_ref(
        _context: &Context,
        _code: PhantomData<Code>,
        Literal(value): &Literal<T>,
    ) -> Self::Output {
        LispSubExpr::Literal(Literal(value.clone())).upcast(PhantomData)
    }
}
```

In this implementation, we pattern match on a reference to the `Literal<T>` and simply clone the value before constructing a new `Literal` variant inside a helper enum. This enum, `LispSubExpr`, plays the same role here as it did in the `PlusToLisp` provider: it defines a minimal subset of variants sufficient to perform the transformation.

```rust
#[derive(HasFields, ExtractField, FromVariant)]
enum LispSubExpr<T> {
    Literal(Literal<T>),
}
```

What makes this pattern especially powerful is that the `LispSubExpr` and `Literal` enums are completely parameterized over the literal type `T`. This means that the transformation logic does not need to know or care about what kind of value the literal holds. As long as `T` can be cloned, the provider works uniformly for all supported literal types — whether they are numbers, strings, or other values.

There is another subtle but important aspect to this design: the `Literal` type used here is exactly the same as the one used in our arithmetic expression tree. In other words, the same data structure is reused across both the source language (`MathExpr`) and the target language (`LispExpr`). This isn’t just a convenience — it opens the door to reusing logic across very different language expressions.

#### Wiring To-Lisp Handlers

With the Lisp transformation providers now defined, the final step is to integrate them into the interpreter context. This is where the individual pieces — evaluation, transformation, and type configuration — are all connected through CGP’s `delegate_components!` macro. To do this, we update the `InterpreterComponents` definition so that it includes the logic required for converting expressions into their Lisp representations:

```rust
#[derive(Eq, PartialEq, Debug, HasFields, FromVariant, ExtractField)]
pub enum LispExpr {
    List(List<LispExpr>),
    Literal(Literal<u64>),
    Ident(Ident),
}

delegate_components! {
    InterpreterComponents {
        LispExprTypeProviderComponent:
            UseType<LispExpr>,
        ComputerComponent:
            UseInputDelegate<
                new EvalComponents {
                    MathExpr: DispatchEval,
                    Literal<u64>: EvalLiteral,
                    Plus<Expr>: EvalAdd,
                    Times<Expr>: EvalMultiply,
                }
            >,
        ComputerRefComponent:
            UseInputDelegate<
                new ToLispComponents {
                    Expr: DispatchToLisp,
                    Literal<u64>: LiteralToLisp,
                    Plus<Expr>: PlusToLisp,
                    Times<Expr>: TimesToLisp,
                }
            >,
    }
}
```

In this setup, the `LispExprTypeProviderComponent` establishes the concrete `LispExpr` enum as the actual type behind the abstract `LispExpr` used in our providers. This mapping is done through `UseType`, which binds the type parameter required by `HasLispExprType` to the specific enum definition we want to use in the final output.

The `ComputerComponent` remains unchanged from when we configured the system for arithmetic evaluation. It continues to delegate evaluation logic to the appropriate providers, such as `EvalAdd` for addition and `EvalLiteral` for literal values.

The main addition here is the `ComputerRefComponent`, which enables reference-based computations — specifically, the transformation of expression trees into Lisp form without taking ownership of them. This component also uses `UseInputDelegate`, but it connects to a different set of providers: those responsible for generating Lisp output. It includes the transformation logic for `Plus`, `Times`, and `Literal`, each handled by their respective providers.

For the top-level `MathExpr` type, we introduce `DispatchToLisp`, a dedicated dispatcher that routes the various expression variants to their corresponding transformation providers. It is defined as follows:

```rust
#[cgp_new_provider]
impl<Code> ComputerRef<Interpreter, Code, MathExpr> for DispatchToLisp {
    type Output = LispExpr;

    fn compute_ref(context: &Interpreter, code: PhantomData<Code>, expr: &MathExpr) -> Self::Output {
        <MatchWithValueHandlersRef>::compute_ref(context, code, expr)
    }
}
```

This implementation mirrors the earlier `DispatchEval`, but with one key distinction: it uses `MatchWithValueHandlersRef`, a visitor dispatcher designed specifically for reference-based operations. Rather than consuming the input, it operates on borrowed values and dispatches calls to providers that implement the `ComputerRef` trait.

One of the major advantages of this approach is that it is entirely driven by the type system. Because the dispatcher is implemented generically — as a regular Rust `impl` rather than a macro — it benefits fully from the compiler’s ability to check lifetime correctness, trait bounds, and input-output consistency. Mistakes such as passing the wrong reference type, using incompatible trait bounds, or violating borrowing rules are caught immediately at compile time, often with clear and actionable error messages.

If this logic had instead been implemented using traditional Rust macros, many of these issues would only surface later during macro expansion or execution, making them harder to trace and debug. CGP’s generic dispatchers, by contrast, offer the same level of automation while remaining transparent and fully type-checked.

The `MatchWithValueHandlers` and `MatchWithValueHandlersRef` dispatchers are just two examples of CGP’s modular dispatching infrastructure. CGP provides a *family* of such dispatchers, each tuned for a particular use case — whether by value, by reference, or with more specialized patterns. These dispatchers are designed to be extensible and interchangeable, giving you fine-grained control over how your logic is routed while preserving flexibility.

With both evaluation and Lisp transformation now wired into the same interpreter context, the system is able to evaluate expressions to numeric results or convert them into Lisp-style syntax trees, all from the same `MathExpr` type. The modularity, reusability, and compile-time guarantees of this architecture make CGP a powerful and scalable tool for building language runtimes and transformation pipelines in Rust.

## Advanced Techniques

### Binary Operator Provider

When examining the implementations of `PlusToLisp` and `TimesToLisp`, it quickly becomes clear that they follow nearly identical patterns. Aside from the specific operator symbol and the input types, the transformation logic is the same. This duplication presents a perfect opportunity for *further abstraction*.

By extracting the shared structure, we can implement a generalized provider, `BinaryOpToLisp`, that handles both `Plus` and `Times` expressions using a single implementation:

```rust
#[cgp_new_provider]
impl<Context, Code, MathExpr, MathSubExpr, LispExpr, Operator>
    ComputerRef<Context, Code, MathSubExpr> for BinaryOpToLisp<Operator>
where
    Context: HasMathExprType<MathExpr = MathExpr>
        + HasLispExprType<LispExpr = LispExpr>
        + CanComputeRef<Code, MathExpr, Output = LispExpr>,
    MathSubExpr: BinarySubExpression<MathExpr>,
    Operator: Default + Display,
    LispSubExpr<LispExpr>: CanUpcast<LispExpr>,
{
    type Output = LispExpr;

    fn compute_ref(context: &Context, code: PhantomData<Code>, expr: &MathSubExpr) -> Self::Output {
        let expr_a = context.compute_ref(code, expr.left());
        let expr_b = context.compute_ref(code, expr.right());

        let ident = LispSubExpr::Ident(Ident(Operator::default().to_string())).upcast(PhantomData);

        LispSubExpr::List(List(vec![ident.into(), expr_a.into(), expr_b.into()]))
            .upcast(PhantomData)
    }
}
```

This provider introduces a generic `Operator` type, which is expected to represent the binary operator as a type-level string, such as `"+"` or `"*"`. To support this, `Operator` must implement both `Default` and `Display`. These traits allow the provider to convert the operator type into a string during execution, which is then used to create a `LispSubExpr::Ident` variant representing the operation.

The input to this provider is any `MathSubExpr` — which could be `Plus`, `Times`, or any other binary expression type — that implements the `BinarySubExpression` trait:

```rust
#[cgp_auto_getter]
pub trait BinarySubExpression<Expr> {
    fn left(&self) -> &Box<Expr>;
    fn right(&self) -> &Box<Expr>;
}
```

By annotating this trait with `#[cgp_auto_getter]`, CGP can automatically implement it for any struct that contains `left` and `right` fields of type `Box<Expr>`. This removes the need to manually implement the trait for each binary operator type and allows the generic provider to access subexpressions in a uniform way.

To connect this trait to the right expression type, we introduce the `HasMathExprType` trait:

```rust
#[cgp_type]
pub trait HasMathExprType {
    type MathExpr;
}
```

This trait plays a similar role to `HasLispExprType`, allowing us to define the abstract `MathExpr` type outside of the generic parameters of the provider. It ensures that the right type is used consistently throughout the system and helps avoid ambiguity when specifying the generic parameter for `BinarySubExpression`.

The body of `compute_ref` mirrors the logic we saw earlier. We evaluate both the left and right subexpressions recursively, construct a Lisp identifier by calling `Operator::default().to_string()`, and then build a list containing the operator followed by the operands. The resulting Lisp structure is then upcast into the final `LispExpr` type.

With this reusable provider in place, we can now eliminate the separate implementations for `PlusToLisp` and `TimesToLisp`, and wire both operators through a single generic provider in our component configuration:

```rust
delegate_components! {
    InterpreterComponents {
        ...
        MathExprTypeProviderComponent:
            UseType<MathExpr>,
        ComputerRefComponent:
            UseInputDelegate<
                new ToLispComponents {
                    MathExpr: DispatchToLisp,
                    Literal<Value>: LiteralToLisp,
                    Plus<MathExpr>: BinaryOpToLisp<Symbol!("+")>,
                    Times<MathExpr>: BinaryOpToLisp<Symbol!("*")>,
                }
            >,
    }
}
```

Here, we map `Plus<MathExpr>` and `Times<MathExpr>` to the same `BinaryOpToLisp` provider, each with a different `Symbol!` type-level string.

Thanks to CGP’s expressive delegation system and powerful match-based dispatching via `MatchWithValueHandlersRef`, this setup allows us to write reusable, composable transformation logic. Rather than duplicating the same structure across multiple providers, we define it once in a generic form and let the type system handle the rest.

### Code-Based Dispatching

Earlier, we explored the difference between the `Computer` and `ComputerRef` traits and saw how `ComputerRef` offers a cleaner and more efficient interface for computations that don’t require ownership of the input. This naturally applies to our evaluation logic as well — after all, an evaluator only needs to borrow the expression, not consume it.

However, once we refactor our `EvalAdd`, `EvalMultiply`, and other evaluation providers to use `ComputerRef`, we run into a challenge: we’ve already wired `ComputerRefComponent` for the purpose of transforming expressions to Lisp. How do we now support *both* evaluation and transformation using the same trait?

This is where the `Code` parameter in the `ComputerRef` trait comes into play. If you’ve read about [Hypershell’s design](/blog/hypershell-release/#generic-dispatcher), you’ll recognize `Code` can be used to build type-level DSLs to encode the kind of operation we want to perform. In our interpreter, we can apply the same idea to distinguish between different computation *intentions* — for example, evaluation vs. conversion.

Let’s begin by defining two types that represent our operations at the type level:

```rust
pub struct Eval;
pub struct ToLisp;
```

These act as “statements” in our interpreter DSL. `Eval` represents program evaluation, while `ToLisp` represents conversion into Lisp syntax. This gives us a lightweight and expressive way to route logic based on the kind of computation we want to perform.

With that in place, we can define a single provider for handling `Plus` expressions, where the behavior is determined by the `Code`:

```rust
delegate_components! {
    new HandlePlus {
        ComputerRefComponent: UseDelegate<
            new PlusHandlers {
                Eval: EvalAdd,
                ToLisp: BinaryOpToLisp<Symbol!("+")>,
            }>
    }
}
```

Here, we use `delegate_components!` to define a new provider called `HandlePlus`. Inside it, we delegate the `ComputerRefComponent` implementation to `UseDelegate`, which performs `Code`-based dispatching based on the newly created dispatch table `PlusHandlers`. If the `Code` is `Eval`, it uses `EvalAdd`. If it’s `ToLisp`, it uses `BinaryOpToLisp<Symbol!("+")>`.

Thanks to CGP’s blanket implementations, `HandlePlus` automatically becomes a valid `ComputerRef` provider for `Plus<Expr>` — delegating to the appropriate providers based on `Code`.

We could also have achieved similar functionality by writing two separate `impl` blocks for `HandlePlus`, like this:

```rust
pub struct HandlePlus;

#[cgp_provider]
impl<Context, MathExpr> Computer<Context, Eval, Plus<MathExpr>> for HandlePlus {
    ...
}

#[cgp_provider]
impl<Context, MathExpr> ComputerRef<Context, ToLisp, Plus<MathExpr>> for HandlePlus {
    ...
}
```

This works too, but it introduces friction. Each `impl` must be written in the same crate as the type it targets (in this case, `HandlePlus`). That restricts how you organize your code. If you wanted to group all `Eval` logic into one crate and all `ToLisp` logic into another, this approach would make it more challenging to separate the implementations.

Using `delegate_components!`, on the other hand, gives you complete modularity. You can define `EvalAdd` and `BinaryOpToLisp` in completely separate places, and only compose them when building the actual interpreter.

We follow this same pattern for `Times` and `Literal`:

```rust
delegate_components! {
    new HandleTimes {
        ComputerRefComponent: UseDelegate<
            new TimesHandlers {
                Eval: EvalMultiply,
                ToLisp: BinaryOpToLisp<Symbol!("*")>,
            }>
    }
}

delegate_components! {
    new HandleLiteral {
        ComputerRefComponent: UseDelegate<
            new LiteralHandlers {
                Eval: EvalLiteral,
                ToLisp: LiteralToLisp,
            }>
    }
}
```

Finally, we define a top-level dispatcher that handles the `MathExpr` enum itself:

```rust
delegate_components! {
    new HandleMathExpr {
        ComputerRefComponent: UseDelegate<
            new MathExprHandlers {
                Eval: DispatchEval,
                ToLisp: DispatchToLisp,
            }>
    }
}
```

Now that we’ve defined these composed providers, we can plug them into the interpreter’s wiring:

```rust
delegate_components! {
    InterpreterComponents {
        MathExprTypeProviderComponent:
            UseType<MathExpr>,
        LispExprTypeProviderComponent:
            UseType<LispExpr>,
        ComputerRefComponent:
            UseInputDelegate<
                new ExprComputerComponents {
                    MathExpr: HandleMathExpr,
                    Literal<u64>: HandleLiteral,
                    Plus<MathExpr>: HandlePlus,
                    Times<MathExpr>: HandleTimes,
                }
            >,
    }
}
```

At this point, we’ve created a *two-layer* dispatch system. The first layer selects a handler based on the *input type* — e.g., `Plus<MathExpr>`. The second layer selects a handler based on the *code type* — e.g., `Eval` vs. `ToLisp`.

This approach is flexible and composable. You could just as easily reverse the order, grouping logic by `Code` first and dispatching on the input type second. That may make more sense if you’re organizing your project by functionality (say, all evaluation logic in one crate, all Lisp transformation logic in another).

Importantly, the dispatch ordering is entirely compile-time and has **no impact on performance**. CGP uses Rust’s type system and monomorphization to resolve all this dispatch at compile time, so whether you dispatch by `Input` first or `Code` first, the result is the same: fast, zero-cost, strongly typed behavior.

This layered dispatch model is one of CGP’s superpowers. It enables you to write simple, focused components and compose them in flexible, scalable ways — without macros, runtime reflection, or boilerplate.

## Extending `MathExpr`

With the basic interpreter in place, supporting addition and multiplication, it’s natural to explore how we can extend the language further. To demonstrate the modularity and flexibility of CGP, let’s add two new features: *subtraction* and *negation*. These are simple but meaningful enhancements that allow us to test how well our interpreter handles incremental language growth.

Now, one could argue that subtraction and negation are not strictly necessary in the core language. After all, both operations can be expressed using multiplication by `-1`. But while this may be theoretically sound, practical language design often involves more than minimalism. By promoting these features to first-class status, we can greatly improve the ergonomics of writing and reading programs in the language.

This kind of design decision mirrors broader discussions in language evolution. Consider CGP itself. While we’ve built everything so far using CGP purely as a Rust library, it’s conceivable to imagine CGP becoming a *native* feature of the language. From a purist’s perspective, native support might not seem essential — after all, we’ve shown that powerful, compile-time generics-based programming is already achievable today. But once a tool like CGP becomes central to how systems are built, native support brings significant benefits: smoother integration, better diagnostics, and a lower learning curve.

In fact, if Rust were implemented using CGP from the start, it would be much easier to extend the language with features like CGP itself. There would be no need to fork the compiler or jump through macro-related hoops. Extensions could be introduced as structured additions to the language, just as we are now extending our interpreter with new syntax.

### Defining the `MathPlusExpr` Expression Type

To see how CGP enables modular language extension, let’s define a new expression type — `MathPlusExpr` — that expands on our original `MathExpr`. Crucially, this new enum does not replace the old one. Instead, it lives *alongside* it, allowing us to demonstrate how CGP supports language variants and extensions without duplicating logic or entangling implementations.

```rust
#[derive(Debug, HasFields, FromVariant, ExtractField)]
pub enum MathPlusExpr {
    Plus(Plus<MathPlusExpr>),
    Times(Times<MathPlusExpr>),
    Minus(Minus<MathPlusExpr>),
    Negate(Negate<MathPlusExpr>),
    Literal(Literal<i64>),
}
```

At a glance, `MathPlusExpr` looks much like `MathExpr`, but it includes two new variants: `Minus` for subtraction and `Negate` for unary negation. For the original variants — addition, multiplication, and literals — we continue to use the same generic sub-expression types as before, now instantiated with `MathPlusExpr`.

We’ve also changed the numeric type for `Literal` from `u64` to `i64`, enabling the representation of negative values. This change may seem minor, but it highlights an important feature of the system: the ability to evolve types naturally without breaking compatibility. Thanks to CGP’s generic approach, providers like `EvalAdd` and `EvalMultiply` still work seamlessly. Since `i64` also implements `Add` and `Mul`, the existing evaluators remain fully reusable without modification.

For the two new variants, we define their associated sub-expression types just as we did before:

```rust
pub struct Minus<Expr> {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

pub struct Negate<Expr>(pub Box<Expr>);
```

### Implementing Eval Providers

With our extended expression language in place, the next step is to implement evaluation logic for the new constructs. We begin with subtraction. The evaluator for `Minus` is straightforward and closely mirrors what we’ve already done for addition and multiplication. The only real difference is that we now use the `Sub` trait to handle the subtraction operation.

```rust
#[cgp_new_provider]
impl<Context, Code, MathExpr, Output> ComputerRef<Context, Code, Minus<MathExpr>> for EvalSubtract
where
    Context: CanComputeRef<Code, MathExpr, Output = Output>,
    Output: Sub<Output = Output>,
{
    type Output = Output;

    fn compute_ref(
        context: &Context,
        code: PhantomData<Code>,
        Minus { left, right }: &Minus<MathExpr>,
    ) -> Self::Output {
        let output_a = context.compute_ref(code, left);
        let output_b = context.compute_ref(code, right);

        output_a - output_b
    }
}
```

We follow a similar pattern for the `Negate` expression. Since negation is a unary operation, its implementation is even simpler. We compute the value of the inner expression, then apply the `Neg` trait to produce the final result.

```rust
#[cgp_new_provider]
impl<Context, Code, MathExpr, Output> ComputerRef<Context, Code, Negate<MathExpr>> for EvalNegate
where
    Context: CanComputeRef<Code, MathExpr, Output = Output>,
    Output: Neg<Output = Output>,
{
    type Output = Output;

    fn compute_ref(
        context: &Context,
        code: PhantomData<Code>,
        Negate(expr): &Negate<MathExpr>,
    ) -> Self::Output {
        -context.compute_ref(code, expr)
    }
}
```

### Wiring of `InterpreterPlus`

To complete the extension, we define a new context called `InterpreterPlus`. This context wires together the evaluation logic for our extended expression language, including subtraction and negation.

```rust
#[cgp_context]
pub struct InterpreterPlus;

delegate_components! {
    InterpreterPlusComponents {
        ComputerRefComponent:
            UseDelegate<new CodeComponents {
                Eval: UseInputDelegate<new EvalComponents {
                    MathPlusExpr: DispatchEval,
                    Plus<MathPlusExpr>: EvalAdd,
                    Times<MathPlusExpr>: EvalMultiply,
                    Minus<MathPlusExpr>: EvalSubtract,
                    Negate<MathPlusExpr>: EvalNegate,
                    Literal<i64>: EvalLiteral,
                }>,
            }>
    }
}

#[cgp_new_provider]
impl ComputerRef<InterpreterPlus, Eval, MathPlusExpr> for DispatchEval {
    type Output = i64;

    fn compute_ref(
        context: &InterpreterPlus,
        code: PhantomData<Eval>,
        expr: &MathPlusExpr,
    ) -> Self::Output {
        <MatchWithValueHandlersRef>::compute_ref(context, code, expr)
    }
}
```

Thanks to CGP’s modular design, implementing `InterpreterPlus` requires only a few dozen lines of code. The core task here is to dispatch each sub-expression type to its corresponding provider. We also define a context-specific wrapper implementation that enables recursive evaluation through `MatchWithValueHandlersRef`. This approach highlights how CGP makes it easy to extend and organize language features cleanly and efficiently.

### Omitting To-Lisp Implementations

At this stage, you might assume that supporting to-Lisp conversion for `MathPlusExpr` is necessary before proceeding further. However, when rapidly prototyping new language extensions, it is often desirable to *skip* implementing less critical features like to-Lisp conversion and focus solely on the core logic, such as evaluation.

This is where CGP shines. You can choose **not to** implement certain language features — like to-Lisp conversion for `MathPlusExpr` — and still have your evaluation code compile and work perfectly without any extra effort.

This stands in stark contrast to typical Rust designs that rely on **heavyweight traits** with many methods. In those cases, introducing a new type like `MathPlusExpr` usually forces you to provide boilerplate implementations, often filled with `unimplemented!()`, just to satisfy the compiler. This can quickly become cumbersome and confusing, making it hard to know which methods are truly essential for an initial prototype.

With CGP, the minimal trait design and lazy wiring mean that components are only checked for implementation when they are actually *used*. As a result, you can safely defer adding to-Lisp conversion for `Minus` and `Negate` without worrying about subtle runtime panics or crashes caused by missing implementations.

Thanks to CGP’s flexibility and strong compile-time guarantees, once your code compiles, you can trust that missing non-essential features won’t break your core functionality — allowing you to focus on what matters most in early development.

## Conclusion

By now, we’ve seen how extensible variants and the CGP visitor pattern open up a new frontier in modular interpreter design. You can find the full source code of the examples in this article at our [GitHub repository](https://github.com/contextgeneric/cgp-examples/tree/main/expression).

Rather than tying our logic to rigid enums or bloated visitor traits, we’ve been able to deconstruct and evaluate expressions with reusable, decoupled components — all backed by strong compile-time guarantees. Whether we’re evaluating arithmetic, transforming into Lisp, or handling richer variants down the line, each operation remains isolated, composable, and safe.

This is more than a workaround for the expression problem — it’s a foundational shift in how we think about data structures and operations in Rust. With CGP, you no longer need to trade off between extensibility and type safety. You can add new variants without touching existing code, and build interpreters or transformers that evolve organically with your domain.

In [Part 3 of this series, **Implementing Extensible Records**](/blog/extensible-datatypes-part-3), we will dive into the *underlying* implementation details of **extensible records**, and how the extensible builder pattern is built on top of it. We will cover the concepts of **partial records**, and the use of traits such as `BuildField` and `FinalizeField` to represent *row constraints*.
