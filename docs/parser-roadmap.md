# Parser and Frontend Roadmap

This document tracks the near-term parser/frontend improvements needed for the VoltTS prototype. It summarizes gaps discovered while exercising the current line-based parser and suggests concrete next steps.

## Current limitations
- **Line-based parsing only**: multi-line constructs (e.g. `if` blocks, array/object literals) are not recognized reliably because parsing is split on lines.
- **No tokenizer**: the parser operates on raw strings, making string/number handling and future grammar extensions fragile.
- **Stmt-only AST**: expressions are folded into statements, which prevents variable bindings, assignments, and binary operations.
- **Missing control flow/variables**: `if`/`else` and `let` bindings are not emitted, constraining user programs to straight-line calls.
- **Sparse diagnostics**: parse errors lack file/line/column spans, so users cannot pinpoint failures.

## Proposed building blocks
- **Tokenization**: introduce a `Token` enum (keywords, identifiers, literals, punctuation) and a `tokenize(&str)` helper to normalize whitespace and delimiters before parsing.
- **Expr/Stmt split**: add an `Expr` enum (literals, identifiers, calls, binary ops, `await`) alongside a richer `Stmt` enum (`Let`, `Assign`, `Return`, expression statements, `If`).
- **Basic control flow**: emit `if (cond) { ... } else { ... }` in generated C, alongside local variable declarations mapped from `let` bindings.
- **Collections**: start with array/object literals lowered to simple C arrays/structs to unblock common patterns.
- **Spans and typed errors**: add a `Span { file, line, col }` plus a `VoltError` enum to carry structured diagnostics.

## Suggested order of work
1. Tokenizer scaffolding to decouple lexing from parsing.
2. Expr/Stmt refactor to unlock variables and arithmetic.
3. `let` bindings and `if`/`else` codegen for minimal control flow.
4. Span-aware errors to improve developer feedback.
5. Collection literals and expanded standard library (env/process/math/json/http) once the syntax surface is stable.

Keeping these steps small and iterative should let us evolve the language without disrupting the existing CLI/codegen pipeline.
