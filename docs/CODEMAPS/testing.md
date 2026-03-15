<!-- Generated: 2026-03-15 | Files scanned: 28 | Token estimate: ~600 -->

# Testing Codemap

## Test Distribution (195 total, 1 skipped)

```
tests/
├── src/                          # Main test crate (integration_tests)
│   ├── equivalence/ (81 tests)   # TS↔Rust identical stdout
│   │   ├── basic.rs        (12)  arithmetic, unary, control flow
│   │   ├── strings.rs      (14)  16 string methods
│   │   ├── arrays.rs       (11)  15 array methods
│   │   ├── math.rs         (11)  15 math functions
│   │   ├── spread.rs        (5)  array + object spread
│   │   ├── nestjs_logic.rs  (5)  service DI, create sequence
│   │   ├── error_handling.rs(4)  try-catch, throw
│   │   ├── control_flow.rs  (4)  for-of, do-while, switch, ternary
│   │   ├── getters_setters.rs(3) get/set accessor methods
│   │   ├── top_level.rs     (3)  const, let, mixed
│   │   ├── console.rs       (3)  log formatting
│   │   ├── type_features.rs (2)  as Type, numeric enums
│   │   ├── static_members.rs(2)  static methods + call-site
│   │   └── inheritance.rs   (2)  extends, super(), override
│   ├── unit/ (49 tests)          # Fast codegen function tests
│   │   ├── expr.rs         (10)
│   │   ├── stmt.rs         (10)
│   │   ├── tier4_nestjs.rs (14)  NestJS decorators, guards
│   │   ├── types.rs         (9)
│   │   ├── tier3.rs         (6)
│   │   └── stdlib.rs        (4)  string/array/math
│   ├── snapshot/ (20 tests)      # insta snapshot tests
│   │   ├── tier1.rs         (6)
│   │   ├── tier2.rs         (5)
│   │   ├── tier3.rs         (7)
│   │   └── tier4_nestjs.rs  (2)
│   ├── compilation/ (9)          # cargo check per tier
│   ├── ir/ (21)                  # IR lowering
│   ├── cli.rs (7)                # CLI commands
│   └── trybuild (1)             # compile-verification
├── tests/
│   └── tier4_tests.rs (5)       # E2E: DI extraction, build, HTTP
└── fixtures/
    ├── tier1-4/                  # TS fixtures per complexity
    ├── reference_nestjs/         # Full NestJS project
    └── tier4_multi_module/       # Multi-directory project
```

## Test Helper Chain

```
assert_output_equivalent(ts_code)
  1. run_node(ts_code)      → Node.js stdout
  2. transpile(ts_code)     → Rust source
  3. compile_and_run_rust() → Rust stdout
  4. assert_eq!(ts_out, rust_out)
```

## E2E HTTP Test

```
test_http_equivalence_rust_server
  1. build_project(fixture, output)
  2. patch port → 3100
  3. cargo build --release
  4. spawn server process
  5. poll health endpoint (30 attempts)
  6. curl GET / → verify "ok"
  7. curl GET /users → verify "[]"
  8. kill server
```
