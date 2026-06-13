# Expression Type Checker

**Type checking** is the static analysis process that ensures every operation in an expression tree receives operands of compatible types — preventing runtime errors like adding a number to a boolean, or concatenating a string with an integer. This crate implements a **Hindley-Milner-style type checker** for a small expression language with `Number`, `Bool`, `String`, and `Unit` types.

## Why It Matters

Type errors are the most common class of bugs in dynamically-typed languages. By checking types **before execution**, we eliminate entire categories of runtime failures: `1 + true`, `"hello" < 3`, or `if 42 then ... else ...`. Production systems from TypeScript's type checker to Rust's borrow checker to SQL query planners all perform type analysis as a correctness gate. This crate demonstrates the core algorithm: a **syntax-directed checker** that walks the AST, maintains a type environment for variables, and reports type mismatches with precise error messages.

## How It Works

The type checker performs a **post-order traversal** of the AST: it first determines the types of sub-expressions, then checks whether the operator's type signature accepts them.

### Type System

```
Type ::= Number | Bool | String | Unit
```

Each binary operator has a **type signature** — a mapping from operand types to result type:

| Operator | Left | Right | Result |
|----------|------|-------|--------|
| `+ - * /` | Number | Number | Number |
| `< >` | Number | Number | Bool |
| `== !=` | T | T | Bool |
| `&& \|\|` | Bool | Bool | Bool |
| `++` (concat) | String | String | String |

Unary operators follow similar rules: `Neg(Number) → Number`, `Not(Bool) → Bool`, `Len(String) → Number`.

### Type Environment

Variable bindings are tracked in a `HashMap<String, Type>`. The `Let` construct scopes variables: entering a `let x = e1 in e2` inserts `x`'s type, evaluates `e2`, then removes `x` — implementing **lexical scoping**.

### Conditional Expressions

For `if c then t else e`:
1. Condition `c` must have type `Bool`
2. Both branches must have the **same type** (no implicit coercion)
3. If no `else` branch, the result is `Unit`

### Error Reporting

The checker distinguishes five error kinds: type mismatch, undefined variable, non-boolean condition, branch type divergence, and unsupported operator/type combination.

### Complexity

Each node is visited once: **O(n)** in AST size. Environment operations are O(1) average for HashMap lookups.

## Quick Start

```rust
use std::collections::HashMap;

// The TypeChecker walks the AST and returns Result<Type, TypeError>
let mut tc = TypeChecker::new();

// Valid: 1 + 2 : Number
let expr = Expr::Binary {
    op: BinOp::Add,
    left: Box::new(Expr::LitNum(1.0)),
    right: Box::new(Expr::LitNum(2.0)),
};
assert_eq!(tc.check(&expr), Ok(Type::Number));

// Invalid: 1 + true — type error!
let bad = Expr::Binary {
    op: BinOp::Add,
    left: Box::new(Expr::LitNum(1.0)),
    right: Box::new(Expr::LitBool(true)),
};
assert!(tc.check(&bad).is_err());
```

## API

| Type / Function | Description |
|----------------|-------------|
| `TypeChecker` | Holds the type environment; main `check(&Expr) → Result<Type, TypeError>` method |
| `Type` | `Number`, `Bool`, `String`, `Unit` |
| `TypeError` | Variants: `Mismatch`, `UndefinedVar`, `NonBoolCond`, `BranchMismatch`, `UnsupportedOp`, `UnsupportedUnOp` |
| `Expr` | AST: `LitNum`, `LitBool`, `LitStr`, `Var`, `Binary`, `Unary`, `If`, `Let` |
| `BinOp` / `UnOp` | Operator enums with type-level signatures |

## Architecture Notes

This crate is the **validation stage** of the SuperInstance expression pipeline. After `expr-parser` produces an AST, `expr-typecheck` verifies type correctness before `expr-optimize` simplifies the tree. This implements **γ + η = C**: γ (type safety) ensures that all optimizations preserve semantic correctness.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for the full system design.

## References

1. Pierce, B. C. *Types and Programming Languages*. MIT Press, 2002. Chapters 9–11.
2. Damas, L., Milner, R. "Principal type-schemes for functional programs." *POPL 1982*.
3. Oswald, C., Campora, J., Chen, S., Hermann, B. "Type Error Diagnosis for Improving the Developer Experience." *ICSE 2024*.

## License

MIT
