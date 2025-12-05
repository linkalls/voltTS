# VoltTS Improvement Roadmap

This roadmap captures near-term enhancements discussed for VoltTS, focusing on parser architecture, diagnostics, language features, and supporting tooling.

## Frontend and Parser Architecture
- Transition the string-based parser to a structured AST so later passes (type checking, optimization, code generation) can share a stable representation.
- Evaluate parser generators such as `pest` or `logos` + hand-rolled parser; pick an approach that yields clear grammar files and maintainable tokens.
- Preserve source spans (line/column) on AST nodes to enable high-quality diagnostics and formatting.

## Diagnostics and Error Handling
- Centralize error types with `thiserror` (or `anyhow` for application-level flow) to standardize error reporting.
- Adopt `miette`/`codespan-reporting` for colorful, source-annotated diagnostics with line/column highlights and notes.
- Ensure parse/type errors carry spans from the AST and emit actionable messages.

## Type System Foundations
- Introduce explicit type annotations (`number`, `string`, etc.) and type inference where unambiguous.
- Implement type-checked function signatures, return types, and simple generics-ready plumbing.
- Add `Result`/`Option`-like semantics to prepare for error-handling constructs.

## Control Flow and Expressions
- Extend control flow with `if`/`else if`/`else`, `for` (range/iterator), `while`, and a `match` expression inspired by Rust.
- Model the constructs directly in the AST and ensure code generation emits predictable C output.

## Collections and Data Structures
- Support array and object literals with corresponding type information and basic indexing/property access.
- Define how these map onto the C runtime (structs/arrays) to keep generated code straightforward.

## Standard Library Growth
- Phase 1: `std/json` for parse/stringify.
- Phase 2: `std/net` HTTP server/client helpers plus `std/testing` primitives for assertions and test discovery.
- Phase 3: Additional packages (`std/crypto`, `std/path`, `std/env`, `std/process`, `std/strings`, `std/fmt`, `std/encoding`) implemented incrementally as the runtime stabilizes.

## Code Generation and Optimization
- Prune unused variables/functions and perform dead-code elimination based on AST reachability.
- Inline small helper functions where safe; keep C output readable for debugging.
- Ensure optimizations preserve source-span mapping so diagnostics remain accurate.

## Tooling and Testing
- Keep the Rust CLI as the canonical test surface via `cargo test`.
- When JS-based harnesses are required, prefer Bun (`bun test`) instead of Node.js to align with the project’s DX goals.

## Suggested Execution Order
1. Strengthen diagnostics (central error types + miette/codespan integration) and record spans in the parser.
2. Land the AST-based parser and migrate existing passes to consume it.
3. Add minimal type checking for variables/functions and integrate `Result`/`Option` patterns.
4. Expand control flow constructs and collection literals; update formatter/linter accordingly.
5. Ship `std/json` and baseline `std/testing`, followed by `std/net` HTTP helpers.
6. Iterate on codegen optimizations (dead-code elimination, inlining, unused-variable pruning).
