# 🗺️ Oxidizer Roadmap

## 🏁 Milestone 0: The Foundation (Current)
- [x] Project Scaffolding (Cargo Workspace)
- [x] CLI Basic Structure (`clap`)
- [x] **Tracer Bullet:** Connect CLI to Parser (Read a file and print "Success")
- [ ] CI/CD Pipeline Setup (GitHub Actions)

## � Milestone 1: The Analyzer (Complete)
- [x] **Analyzer Core:** Implement `ox_analyzer` crate
- [x] **Linter Rules:**
    - [x] Block `any`
    - [x] Block `eval`
    - [x] Block `var`
- [x] **Refactoring:** Compliance with Guidelines (Newtype, Tracing, Tests)
- [x] Implement basic Lints:
    - [x] Ban `any` type
    - [x] Ban `eval`
    - [x] Check for `var` usage
- [ ] Error Reporting with `miette` (Visual spans)

## 🏁 Milestone 2: The Type Transpiler (Complete)
- [x] Convert TS `interface` -> Rust `struct`
- [x] Map primitive types (`string` -> `String`, `number` -> `f64`)
- [x] Auto-derive `Serialize, Deserialize` (Serde)
- [x] Output `.rs` files using `quote!`

## 🏁 Milestone 3: Logic & Functions (Complete)
- [x] Convert simple `fn` declarations
- [x] Basic control flow (`if`, `return`)
- [x] Binary expressions (Math)

## 🚀 Milestone 4: The Modern Stack (Complete)
- [x] **Async/Await Support:**
    - [x] Convert `async function` -> `pub async fn`
    - [x] Unwrap `Promise<T>` return types
    - [x] Convert `await expr` -> `expr.await`
- [x] **Class Support:**
    - [x] Split `class` into `struct` (properties) + `impl` (methods)
    - [x] Convert `constructor` -> `pub fn new() -> Self`
    - [x] Convert `this.prop` -> `self.prop`
    - [x] Add `&self` to instance methods
- [x] **HTTP Client mapping** (`axios` & `fetch` -> `reqwest`)
- [x] **Standard Library Mapping:**
    - [x] Math (max, min, round, floor, ceil, abs, random)
    - [x] String (includes, replace, split, toUpperCase, toLowerCase, trim)
    - [x] Array (push, map, filter, join)
    - [x] JSON (stringify, parse)
    - [x] Console (log, error)
- [x] **Variable Declarations:**
    - [x] `const`/`let` -> `let` bindings
    - [x] Variable initialization support

## ✅ QA & Compliance (Complete)
- [x] **Code Quality:**
    - [x] Fix all compiler warnings
    - [x] Clippy compliance
- [x] **Guidelines.md Compliance:**
    - [x] Newtype Pattern (`FilePath`)
    - [x] Visitor Pattern (AST traversal)
    - [x] Rich error handling (miette)
- [x] **Testing Infrastructure:**
    - [x] Unit tests (8 passing)
    - [x] Snapshot tests (insta)
    - [x] Compilation tests (rustc validation)
    - [x] Complex E2E fixtures

## 📚 Phase 1.5: Standard Library Compliance (The "Shim" Layer)
*Goal: 100% coverage of essential JS/TS APIs mapped to Rust equivalents.*

### 🧮 Math & Numbers
- [ ] **Math Object:**
    - `Math.max/min` -> `f64::max/min` (or `std::cmp`)
    - `Math.round/floor/ceil` -> `.round()/.floor()/.ceil()`
    - `Math.abs` -> `.abs()`
    - `Math.random()` -> `rand::random()` (Requires `rand` crate)
- [ ] **Number Parsing:**
    - `parseInt(x)` -> `x.parse::<i32>()`
    - `parseFloat(x)` -> `x.parse::<f64>()`

### 🧵 Strings
- [ ] **Query:** `.includes()` -> `.contains()`, `.startsWith()`, `.endsWith()`
- [ ] **Transformation:**
    - `.toUpperCase/LowerCase()` -> `.to_uppercase/lowercase()`
    - `.replace(a, b)` -> `.replace(a, b)`
    - `.trim()` -> `.trim()`
    - `.split(sep)` -> `.split(sep).collect::<Vec<_>>()`
- [ ] **Template Literals:** `${var}` -> `format!("{}", var)`

### 📦 Arrays & Iterators
- [ ] **Transformation (Lazy):** `map`, `filter` -> `iter().map()...`
- [ ] **Aggregation:** `reduce` -> `fold`
- [ ] **Search:** `find` -> `iter().find()`, `some` -> `iter().any()`, `every` -> `iter().all()`
- [ ] **Mutation:** `push` -> `push`, `pop` -> `pop`
- [ ] **Utility:** `.length` -> `.len()`, `.join()` -> `.join()`

### 📅 Dates & JSON
- [ ] **JSON:**
    - `JSON.stringify` -> `serde_json::to_string`
    - `JSON.parse` -> `serde_json::from_str`
- [ ] **Date:**
    - `new Date()` -> `chrono::Utc::now()` (Requires `chrono` crate)
    - `.toISOString()` -> `.to_rfc3339()`

### 📝 Compatibility Matrix
- [ ] Create `COMPATIBILITY.md` to track exactly which methods are supported vs unsupported.