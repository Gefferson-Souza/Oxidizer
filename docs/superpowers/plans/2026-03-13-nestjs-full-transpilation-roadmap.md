# NestJS Full Transpilation Roadmap

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transpile a real NestJS project to a Rust (Axum) project where both produce identical HTTP responses for identical HTTP requests — full semantic equivalence at the API level.

**Architecture:** Incremental enhancement of the existing Tyrus transpiler pipeline (SWC → Analyzer → Codegen → Orchestrator). Each phase unlocks a layer of NestJS functionality, verified by equivalence tests that run both TS (Node.js) and Rust and compare outputs.

**Tech Stack:** Rust (quote!, proc_macro2), SWC (swc_ecma_ast/visit), Axum (HTTP framework), tokio (async runtime), serde (serialization), Node.js (equivalence testing)

---

## Current State (2026-03-13)

| Aspect | Status | Details |
|--------|--------|---------|
| Tests | 158 pass, 1 skip | 55 equivalence, 7 CLI, 8 IR, 73 integration, 9 codegen, 4 common, 1 trybuild |
| CLI | 4 commands | check, build, compile, run (branded, --quiet, --json) |
| Analyzer | 8 lint + 11 API blocks | var, any, eval, for-in, try-catch, delete, with, labeled |
| Codegen | ~4190 lines, 20 modules | Expressions, statements, classes, stdlib |
| Stdlib | 51 methods | 16 string + 15 array + 15 math + 5 console |
| NestJS | Basic | @Injectable, @Controller, @Get/@Post, constructor DI, Arc\<Mutex\<T\>\> |
| IR | Foundation | TyrusType/Expr/Stmt/Decl defined, SWC→IR lowering started |

## Gap Analysis: Current → Full NestJS Transpilation

### Critical Path (Must Have)

| Gap | Impact | Why Critical |
|-----|--------|--------------|
| try-catch → Result | Blocks ALL error handling | Every NestJS controller uses try-catch |
| Top-level statements | Blocks main.ts bootstrap | NestJS bootstrap is top-level code |
| throw → Err() | Blocks error propagation | Services throw HttpException |
| Class inheritance | Blocks base classes | NestJS guards/interceptors extend base |
| Spread operator | Blocks DTO patterns | `{...dto, id}` is ubiquitous |
| @Query/@Param/@Headers | Blocks route params | Can't extract query/path params |
| @HttpCode/@Header | Blocks response config | Can't set status codes |
| HttpException hierarchy | Blocks error responses | NestJS uses typed HTTP exceptions |
| Multi-file module system | Blocks real projects | Current: single-file only |
| Validation pipes | Blocks input validation | NestJS uses class-validator |

### Important (Should Have)

| Gap | Impact |
|-----|--------|
| Static methods/properties | Used in utility classes, enums |
| Getters/setters | Used in DTOs, entities |
| Rest parameters | Used in some service methods |
| Type assertions (as Type) | Used in service logic |
| Promise.all | Used for concurrent operations |
| Map/Set data structures | Used for caching |
| Guards (@UseGuards) | Authentication/authorization |
| Interceptors | Logging, transform responses |
| Middleware | CORS, helmet, body parsing |

### Nice to Have (Phase 7+)

| Gap | Impact |
|-----|--------|
| Abstract classes | Base repositories |
| Intersection types | Complex DTOs |
| Type narrowing | Runtime type checks |
| Dynamic modules | forRoot/forAsync |
| Custom providers | useFactory, useValue |
| TypeORM/Prisma → SQLx | Database layer |
| WebSocket gateway | Real-time features |
| Microservices | Inter-service communication |

---

## Phase 6: TypeScript Language Completeness

**Objective:** Fill in all TypeScript language features needed by real NestJS codebases.
**Prerequisite for:** All NestJS framework phases.
**Estimated Scope:** ~50 tasks across 7 milestones.

---

### Milestone 6.0: Transpiler Infrastructure Upgrade (Research-Driven)

**Why:** Research identified critical tooling improvements that should land BEFORE any new features. These reduce friction for all subsequent milestones.

**Academic Justification:**
- **prettyplease**: Used by C2Rust (4656 stars) for formatting generated Rust. Eliminates external `rustfmt` dependency. Handles async code that our current subprocess approach skips.
- **trybuild**: Standard in Rust compiler testing. Used by serde, diesel, sqlx for verifying generated code compiles.
- **thiserror 2.0**: Generates `#[derive(Error)]` in transpiled Axum apps. Replaces hand-written AppError in `format.rs`.

**Files:**
- Modify: `crates/tyrus_orchestrator/Cargo.toml` — Add `prettyplease`, `syn` (features = ["full", "parsing"])
- Modify: `crates/tyrus_orchestrator/src/format.rs` — Replace `rustfmt` subprocess with `prettyplease::unparse()`
- Modify: `crates/tyrus_diagnostics/Cargo.toml` — Upgrade `thiserror` from 1.0 to 2.0
- Modify: `crates/tyrus_di/Cargo.toml` — Upgrade `thiserror` from 1.0 to 2.0
- Modify: `tests/Cargo.toml` — Add `trybuild`

#### Tasks

- [x] **Task 6.0.1: Add prettyplease to orchestrator** ✓

  Added `prettyplease = "0.2"` and `syn = { version = "2.0", features = ["full", "parsing"] }` to orchestrator.

- [x] **Task 6.0.2: Replace rustfmt subprocess with prettyplease** ✓

  Replaced `Command::new("rustfmt")` subprocess + async-skip hack with `syn::parse_file` + `prettyplease::unparse`. Properly formats async code now.

- [x] **Task 6.0.3: Upgrade thiserror to 2.0** ✓

  Updated `tyrus_diagnostics` and `tyrus_di` from thiserror 1.0 → 2.0. No breaking changes.

- [x] **Task 6.0.4: Add trybuild for compile-verification tests** ✓

  Added `trybuild = "1.0"` to tests/Cargo.toml. Created `tests/trybuild/` directory.

- [x] **Task 6.0.5: Upgrade generated AppError with named HTTP variants** ✓

  **Implementation note:** Could not use `#[derive(thiserror::Error)]` on generated `AppError` because it would implement `std::error::Error`, conflicting with the blanket `impl<E: Error> From<E> for AppError` needed for `?` operator support. Instead, used named enum variants with manual `Display` impl + blanket `From<E>`, giving us proper HTTP status code mapping via `IntoResponse` while keeping `?` ergonomics. Also added `axum` to `detect_dependencies()` for single-file builds.

- [ ] **Task 6.0.6: Commit**

  ```bash
  git commit -m "chore: upgrade transpiler infrastructure (prettyplease, thiserror 2.0, trybuild)"
  ```

---

### Milestone 6.1: Error Handling (try-catch-finally)

**Why:** Every NestJS controller and service uses try-catch. Without this, we can't transpile any real error handling logic.

**TypeScript → Rust Mapping:**
```typescript
// TypeScript
try {
  const result = await service.findOne(id);
  return result;
} catch (error) {
  throw new NotFoundException(`User ${id} not found`);
} finally {
  logger.log('Operation completed');
}

// → Rust
match (|| -> Result<_, AppError> {
  let result = service.find_one(id).await?;
  Ok(result)
})() {
  Ok(result) => result,
  Err(error) => {
    return Err(AppError::NotFound(format!("User {} not found", id)));
  }
}
// finally block runs unconditionally (defer pattern or drop guard)
```

**Files:**
- Modify: `crates/tyrus_analyzer/src/lints.rs` — Remove try-catch from blocked list
- Modify: `crates/tyrus_codegen/src/convert/stmt.rs` — Add try-catch → match Result
- Create: `tests/fixtures/tier3/try_catch.ts`
- Create: `tests/src/equivalence/error_handling.rs`
- Modify: `docs/specs/GRAMMAR.md` — Update grammar

#### Tasks

- [ ] **Task 6.1.1: Remove try-catch from analyzer block list**

  Modify `crates/tyrus_analyzer/src/lints.rs`:
  - Remove the entire `fn visit_try_stmt(&mut self, n: &swc_ecma_ast::TryStmt)` method from the `Visit` impl for `LintVisitor` (lines 79-86)
  - This "unlocks" try-catch for transpilation

  ```rust
  // REMOVE this entire method from the Visit impl:
  fn visit_try_stmt(&mut self, n: &swc_ecma_ast::TryStmt) {
      self.errors.push(TyrusError::UnsupportedFeature {
          feature: "try-catch blocks".to_string(),
          src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
          span: self.create_span(n.span),
      });
      n.visit_children_with(self);
  }
  ```

  Run: `cargo clippy --workspace`

- [ ] **Task 6.1.2: Write failing equivalence test for basic try-catch**

  Create `tests/src/equivalence/error_handling.rs`:

  ```rust
  use crate::helpers::assert_output_equivalent;

  #[test]
  fn test_equivalence_try_catch_basic() {
      assert_output_equivalent(r#"
  function safeDivide(a: number, b: number): string {
      try {
          if (b === 0) {
              throw new Error("Division by zero");
          }
          const result: number = a / b;
          return result.toString();
      } catch (error) {
          return "Error caught";
      }
  }
  console.log(safeDivide(10, 2));
  console.log(safeDivide(10, 0));
  "#);
  }
  ```

  Register in `tests/src/equivalence/mod.rs` and `tests/src/lib.rs`.

  Run: `cargo test -p integration_tests test_equivalence_try_catch_basic` — Expected: FAIL

- [ ] **Task 6.1.2b: Refactor stmt.rs before adding try-catch**

  `stmt.rs` is already at 427 lines (near the 400-line project limit). Before adding try-catch logic, extract existing switch/for-of/do-while conversion into a sub-module to make room.

  Create `crates/tyrus_codegen/src/convert/stmt/` directory structure:
  - Move `convert_switch_stmt` → `stmt/switch.rs`
  - Keep core `convert_stmt` in `stmt/mod.rs`

  Run: `cargo test --workspace` — all 157 tests must still pass.

- [ ] **Task 6.1.3: Implement try-catch in stmt.rs**

  Modify `crates/tyrus_codegen/src/convert/stmt.rs` (or `stmt/mod.rs` after refactor):
  - Add `Stmt::Try(try_stmt)` arm to `convert_stmt()`
  - Convert try block → closure returning Result
  - Convert catch block → match arm for Err
  - Handle `catch (e)` binding → `Err(e)` destructuring
  - **Note:** `throw` already works (stmt.rs:377 — `Stmt::Throw` → `return Err(#arg.into())`)

  Strategy: Wrap try body in `match (|| -> Result<_, AppError> { ... })() { Ok(v) => v, Err(e) => { catch_body } }`

  Run: `cargo test -p integration_tests test_equivalence_try_catch_basic` — Expected: PASS

- [ ] **Task 6.1.4: Write equivalence test for try-catch with throw**

  Add to `error_handling.rs`:

  ```rust
  #[test]
  fn test_equivalence_try_catch_throw() {
      assert_output_equivalent(r#"
  function processAge(age: number): string {
      try {
          if (age < 0) {
              throw new Error("Age cannot be negative");
          }
          if (age > 150) {
              throw new Error("Age too high");
          }
          return "Valid age: " + age.toString();
      } catch (error) {
          return "Invalid: caught error";
      }
  }
  console.log(processAge(25));
  console.log(processAge(-5));
  console.log(processAge(200));
  "#);
  }
  ```

  Run: `cargo test -p integration_tests test_equivalence_try_catch` — Expected: PASS

- [ ] **Task 6.1.5: Write equivalence test for nested try-catch**

  ```rust
  #[test]
  fn test_equivalence_nested_try_catch() {
      assert_output_equivalent(r#"
  function outer(): string {
      try {
          try {
              throw new Error("inner");
          } catch (e) {
              return "caught inner";
          }
      } catch (e) {
          return "caught outer";
      }
      return "no error";
  }
  console.log(outer());
  "#);
  }
  ```

- [ ] **Task 6.1.6: Implement finally block (defer pattern)**

  `finally` blocks run unconditionally after try/catch. In Rust, use a drop guard:
  - Create a `struct Finally<F: FnOnce()>(Option<F>)` with `Drop` impl
  - Or inline the finally body after the match expression
  - For simple cases (logging, cleanup), just emit the code after the match

  ```rust
  // Simple approach: emit finally body after the match
  let _try_result = match (|| -> Result<_, AppError> { ... })() {
      Ok(v) => v,
      Err(e) => { catch_body }
  };
  // finally body runs here unconditionally
  logger.log("Operation completed");
  ```

- [ ] **Task 6.1.7: Add try-catch-finally to GRAMMAR.md**

  Update `docs/specs/GRAMMAR.md` with try-catch-finally production rule.

- [ ] **Task 6.1.8: Commit**

  ```bash
  git add crates/tyrus_analyzer/src/lints.rs crates/tyrus_codegen/src/convert/stmt.rs tests/src/equivalence/error_handling.rs tests/src/equivalence/mod.rs tests/src/lib.rs docs/specs/GRAMMAR.md
  git commit -m "feat(codegen): try-catch transpilation to Result matching"
  ```

---

### Milestone 6.2: Top-Level Statements

**Why:** NestJS `main.ts` is entirely top-level code. Currently top-level variable declarations produce EMPTY output because `build()` hardcodes `is_index: false`, which discards `main_body` content.

**Root Cause Analysis:**
1. `crates/tyrus_codegen/src/convert/interface.rs` — The `visit_stmt` catch-all arm already writes non-declaration statements to `self.main_body`. This works correctly.
2. `crates/tyrus_orchestrator/src/lib.rs:39` — `build()` calls `generate(&program, false)`, hardcoding `is_index` to `false`.
3. `crates/tyrus_codegen/src/lib.rs:23` — `if !generator.main_body.is_empty() && is_index` guards the `main()` wrapper. When `is_index` is `false`, `main_body` is silently discarded.

**Note:** `throw` already works — `stmt.rs:377` handles `Stmt::Throw` → `return Err(#arg.into())`. No re-implementation needed.

**TypeScript → Rust Mapping:**
```typescript
// main.ts (top-level)
const port: number = 3000;
console.log(`Server running on port ${port}`);

// → Rust main.rs
fn main() {
    let port: f64 = 3000.0;
    println!("Server running on port {}", port);
}
```

**Files:**
- Modify: `crates/tyrus_orchestrator/src/lib.rs` — Change `is_index` to `true` for single-file builds (or always emit `main()` when `main_body` is non-empty)
- Modify: `crates/tyrus_codegen/src/lib.rs` — Remove or rethink the `is_index` guard on `main_body` emission
- Modify: `tests/src/helpers.rs` — Update `assert_output_equivalent` docstring (currently says "MUST contain function declarations, not top-level statements")
- Create: `tests/src/equivalence/top_level.rs`

#### Tasks

- [ ] **Task 6.2.1: Write failing test for top-level variable declarations**

  Create `tests/src/equivalence/top_level.rs`:
  ```rust
  #[test]
  fn test_equivalence_top_level_const() {
      assert_output_equivalent(r#"
  const greeting: string = "Hello World";
  const count: number = 42;
  const active: boolean = true;
  console.log(greeting);
  console.log(count);
  console.log(active);
  "#);
  }
  ```

  Run: Expected FAIL (currently produces empty output).

- [ ] **Task 6.2.2: Fix build() to emit top-level statements**

  **Fix 1:** `crates/tyrus_orchestrator/src/lib.rs:39` — Change `generate(&program, false)` to `generate(&program, true)` for single-file builds (the `build()` function).

  **Fix 2:** `crates/tyrus_codegen/src/lib.rs:23` — Consider removing the `is_index` guard entirely. If `main_body` is non-empty, always wrap it in `fn main()`. This makes the behavior consistent regardless of calling context.

  Run: `cargo test -p integration_tests test_equivalence_top_level_const` — Expected: PASS

- [ ] **Task 6.2.3: Update assert_output_equivalent docstring**

  Modify `tests/src/helpers.rs`:
  - Remove the restriction "The TypeScript code MUST: Contain function declarations (not top-level statements)"
  - Update to reflect that both top-level statements and function declarations are now supported

- [ ] **Task 6.2.4: Write test for top-level let (mutable)**

  ```rust
  #[test]
  fn test_equivalence_top_level_let() {
      assert_output_equivalent(r#"
  let counter: number = 0;
  counter = counter + 1;
  counter = counter + 1;
  console.log(counter);
  "#);
  }
  ```

- [ ] **Task 6.2.5: Write test for top-level with functions**

  ```rust
  #[test]
  fn test_equivalence_top_level_mixed() {
      assert_output_equivalent(r#"
  function greet(name: string): string {
      return "Hello, " + name;
  }
  const message: string = greet("World");
  console.log(message);
  "#);
  }
  ```

- [ ] **Task 6.2.6: Write test for existing throw support (validation only)**

  Verify the existing `throw` implementation works correctly via equivalence:
  ```rust
  #[test]
  fn test_equivalence_throw_in_function() {
      assert_output_equivalent(r#"
  function validate(age: number): string {
      if (age < 0) {
          throw new Error("negative");
      }
      return "valid";
  }
  console.log(validate(25));
  "#);
  }
  ```

- [ ] **Task 6.2.7: Commit**

  ```bash
  git commit -m "feat(codegen): top-level statement transpilation"
  ```

---

### Milestone 6.3: Spread Operator & Rest Parameters

**Why:** NestJS DTOs use spread constantly: `{...createDto, id: generatedId}`. Services use rest parameters.

**TypeScript → Rust Mapping:**
```typescript
// Object spread
const user = { ...createDto, id: "123" };
// → Rust: struct update syntax or serde merge
let mut user = create_dto.clone();
user.id = String::from("123");

// Array spread
const all = [...arr1, ...arr2];
// → Rust
let all: Vec<_> = arr1.iter().chain(arr2.iter()).cloned().collect();

// Rest parameters
function log(...args: string[]): void { }
// → Rust
fn log(args: Vec<String>) { }
```

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/expr/literal.rs` — Object spread
- Modify: `crates/tyrus_codegen/src/convert/expr/misc.rs` — Array spread
- Modify: `crates/tyrus_codegen/src/convert/fn_decl.rs` — Rest parameters
- Create: `tests/src/equivalence/spread.rs`

#### Tasks

- [ ] **Task 6.3.1: Write failing test for array spread**

  ```rust
  #[test]
  fn test_equivalence_array_spread() {
      assert_output_equivalent(r#"
  const arr1: number[] = [1, 2, 3];
  const arr2: number[] = [4, 5, 6];
  const combined: number[] = [...arr1, ...arr2];
  console.log(combined.join(", "));
  "#);
  }
  ```

- [ ] **Task 6.3.2: Implement array spread in literal.rs**

  In `convert_array_lit()`, detect `ExprOrSpread` with spread=true:
  - Single array: `arr.clone()`
  - Multiple arrays: `arr1.iter().chain(arr2.iter()).cloned().collect()`
  - Mixed: `vec![item1].into_iter().chain(arr.iter().cloned()).collect()`

- [ ] **Task 6.3.3: Write test for object spread**

  ```rust
  #[test]
  fn test_equivalence_object_spread() {
      assert_output_equivalent(r#"
  interface Config {
      host: string;
      port: number;
      debug: boolean;
  }
  function createConfig(base: Config): Config {
      const updated: Config = { ...base, debug: false };
      return updated;
  }
  const config: Config = { host: "localhost", port: 3000, debug: true };
  const prod: Config = createConfig(config);
  console.log(prod.host);
  console.log(prod.port);
  console.log(prod.debug);
  "#);
  }
  ```

- [ ] **Task 6.3.4: Implement object spread in literal.rs**

  Detect `SpreadElement` in object literal:
  - Generate: `let mut obj = base.clone(); obj.field = value;` for struct context
  - Or: `serde_json::json!({})` merge for untyped context

- [ ] **Task 6.3.5: Write test for rest parameters**

  ```rust
  #[test]
  fn test_equivalence_rest_params() {
      assert_output_equivalent(r#"
  function sum(...nums: number[]): number {
      let total: number = 0;
      for (const n of nums) {
          total = total + n;
      }
      return total;
  }
  console.log(sum(1, 2, 3));
  console.log(sum(10, 20));
  "#);
  }
  ```

- [ ] **Task 6.3.6: Implement rest parameters in fn_decl.rs**

  Detect `Pat::Rest(rest_pat)` in function parameters:
  - Last parameter with `...` → `Vec<T>` parameter
  - Caller side: wrap remaining args in `vec![...]`

- [ ] **Task 6.3.7: Commit**

  ```bash
  git commit -m "feat(codegen): spread operator and rest parameters"
  ```

---

### Milestone 6.4: Class Inheritance

**Why:** NestJS guards extend `CanActivate`, interceptors extend `NestInterceptor`, services can extend base classes.

**Research-Validated Approach:** Based on analysis of gtk-rs, enum_dispatch crate, and Rust community patterns (see Research section), we use a **trait + composition + enum_dispatch** strategy:
- Base class → trait definition + struct
- Derived class → struct with base field + trait impl
- Polymorphic contexts → `enum_dispatch` for 10x performance over `dyn Trait`

**TypeScript → Rust Mapping:**
```typescript
class Animal {
    name: string;
    constructor(name: string) { this.name = name; }
    speak(): string { return this.name + " makes a sound"; }
}
class Dog extends Animal {
    breed: string;
    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }
    speak(): string { return this.name + " barks"; }
}

// → Rust (trait + composition, validated by gtk-rs patterns)
trait AnimalTrait {
    fn name(&self) -> &str;
    fn speak(&self) -> String;
}

struct Animal { name: String }
impl AnimalTrait for Animal {
    fn name(&self) -> &str { &self.name }
    fn speak(&self) -> String { format!("{} makes a sound", self.name) }
}

struct Dog { base: Animal, breed: String }
impl AnimalTrait for Dog {
    fn name(&self) -> &str { &self.base.name }
    fn speak(&self) -> String { format!("{} barks", self.base.name) }
}

// For polymorphic usage (enum_dispatch for 10x perf vs dyn Trait)
#[enum_dispatch(AnimalTrait)]
enum AnyAnimal { Animal(Animal), Dog(Dog) }
```

**Crate dependency:** Add `enum_dispatch = "0.3"` to generated Cargo.toml when inheritance is detected.

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/class/mod.rs` — Detect extends, generate trait
- Modify: `crates/tyrus_codegen/src/convert/class/constructor.rs` — super() calls
- Modify: `crates/tyrus_codegen/src/convert/expr/call.rs` — super.method() calls
- Modify: `crates/tyrus_orchestrator/src/scaffold.rs` — Add enum_dispatch to generated deps
- Create: `tests/fixtures/tier3/inheritance.ts`
- Create: `tests/src/equivalence/inheritance.rs`

#### Tasks

- [ ] **Task 6.4.1: Write failing test for basic inheritance**

  ```rust
  #[test]
  fn test_equivalence_class_inheritance_basic() {
      assert_output_equivalent(r#"
  class Animal {
      name: string;
      constructor(name: string) {
          this.name = name;
      }
      speak(): string {
          return this.name + " makes a sound";
      }
  }
  class Dog extends Animal {
      breed: string;
      constructor(name: string, breed: string) {
          super(name);
          this.breed = breed;
      }
      speak(): string {
          return this.name + " barks";
      }
  }
  const dog = new Dog("Rex", "Labrador");
  console.log(dog.speak());
  console.log(dog.breed);
  "#);
  }
  ```

- [ ] **Task 6.4.2: Implement extends detection in class/mod.rs**

  In `process_class_decl()`:
  - Detect `class.super_class` (optional Expr)
  - If present, add `base: ParentType` field to struct
  - Flatten parent fields for direct access (or use delegation)

- [ ] **Task 6.4.3: Implement super() in constructor.rs**

  In `convert_constructor()`:
  - Detect `super(args)` calls in constructor body
  - Convert to `base: ParentType::new(args)` field initialization

- [ ] **Task 6.4.4: Write test for method override**

  ```rust
  #[test]
  fn test_equivalence_method_override() {
      assert_output_equivalent(r#"
  class Shape {
      area(): number { return 0; }
  }
  class Circle extends Shape {
      radius: number;
      constructor(radius: number) {
          super();
          this.radius = radius;
      }
      area(): number { return 3.14159 * this.radius * this.radius; }
  }
  const c = new Circle(5);
  console.log(c.area());
  "#);
  }
  ```

- [ ] **Task 6.4.5: Commit**

  ```bash
  git commit -m "feat(codegen): class inheritance with composition pattern"
  ```

---

### Milestone 6.5: Static Members, Getters/Setters

**Why:** NestJS uses static methods in utility classes. Getters/setters are common in DTOs.

**TypeScript → Rust Mapping:**
```typescript
class Config {
    static defaultPort: number = 3000;
    static create(): Config { return new Config(); }
    private _name: string = "";
    get name(): string { return this._name; }
    set name(value: string) { this._name = value; }
}

// → Rust
impl Config {
    const DEFAULT_PORT: f64 = 3000.0;
    fn create() -> Self { Self::default() }
    fn name(&self) -> &str { &self._name }
    fn set_name(&mut self, value: String) { self._name = value; }
}
```

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/class/mod.rs` — Static detection
- Modify: `crates/tyrus_codegen/src/convert/class/method.rs` — Getter/setter
- Create: `tests/src/equivalence/static_members.rs`

#### Tasks

- [ ] **Task 6.5.1: Write failing test for static methods**

  ```rust
  #[test]
  fn test_equivalence_static_method() {
      assert_output_equivalent(r#"
  class MathUtils {
      static add(a: number, b: number): number {
          return a + b;
      }
      static multiply(a: number, b: number): number {
          return a * b;
      }
  }
  console.log(MathUtils.add(3, 4));
  console.log(MathUtils.multiply(5, 6));
  "#);
  }
  ```

- [ ] **Task 6.5.2: Implement static method detection**

  In `class/method.rs`:
  - Check `method.is_static` flag
  - Static methods: no `&self` parameter, called as `Type::method()`
  - Static properties: `const` associated items in impl block

- [ ] **Task 6.5.2b: Implement static call-site transformation**

  In `crates/tyrus_codegen/src/convert/expr/member.rs`:
  - Detect `ClassName.staticField` → `ClassName::STATIC_FIELD`
  - Detect `ClassName.staticMethod(args)` → `ClassName::static_method(args)`
  - Use heuristic: PascalCase object name + known static members from class processing

- [ ] **Task 6.5.3: Write test for getters/setters**

  ```rust
  #[test]
  fn test_equivalence_getter_setter() {
      assert_output_equivalent(r#"
  class Counter {
      private _count: number = 0;
      get count(): number { return this._count; }
      set count(value: number) { this._count = value; }
      increment(): void { this._count = this._count + 1; }
  }
  const c = new Counter();
  c.increment();
  c.increment();
  console.log(c.count);
  "#);
  }
  ```

- [ ] **Task 6.5.4: Implement getter/setter transpilation**

  In `class/method.rs`:
  - Detect `MethodKind::Getter` → `fn field_name(&self) -> T`
  - Detect `MethodKind::Setter` → `fn set_field_name(&mut self, value: T)`
  - At call sites, `obj.name` → `obj.name()`, `obj.name = v` → `obj.set_name(v)`

- [ ] **Task 6.5.5: Commit**

  ```bash
  git commit -m "feat(codegen): static members and getter/setter transpilation"
  ```

---

### Milestone 6.6: Type Assertions, Typeof, Enums

**Why:** NestJS services use `as Type` casts. Typeof is used in guards.

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/expr/misc.rs` — Type assertions
- Modify: `crates/tyrus_codegen/src/convert/interface.rs` — Enhanced enums

#### Tasks

- [ ] **Task 6.6.1: Write test for type assertion**

  ```rust
  #[test]
  fn test_equivalence_type_assertion() {
      assert_output_equivalent(r#"
  interface User { name: string; age: number; }
  function getUser(): User {
      const data: User = { name: "Alice", age: 30 };
      return data;
  }
  const user: User = getUser();
  console.log(user.name);
  "#);
  }
  ```

- [ ] **Task 6.6.2: Implement as Type assertion**

  `expr as Type` → In most cases, this is a no-op in Rust (types are already known). For `value as string`, generate `.to_string()`. For struct casts, generate type annotation.

- [ ] **Task 6.6.3: Write test for numeric enum with values**

  ```rust
  #[test]
  fn test_equivalence_numeric_enum() {
      assert_output_equivalent(r#"
  enum Direction {
      Up = 0,
      Down = 1,
      Left = 2,
      Right = 3
  }
  function dirName(d: number): string {
      if (d === 0) { return "Up"; }
      if (d === 1) { return "Down"; }
      return "Other";
  }
  console.log(dirName(0));
  console.log(dirName(1));
  "#);
  }
  ```

- [ ] **Task 6.6.4: Commit**

  ```bash
  git commit -m "feat(codegen): type assertions and enhanced enum support"
  ```

---

## Phase 7: NestJS Framework Completeness

**Objective:** Support all NestJS decorators and patterns needed for a real API project.
**Prerequisite:** Phase 6 complete (language features available).
**Estimated Scope:** ~35 tasks across 5 milestones.

---

### Milestone 7.1: HTTP Parameter Decorators

**Why:** Every NestJS controller needs @Query, @Param, @Headers to extract request data.

**TypeScript → Rust Mapping:**
```typescript
@Controller('/users')
class UsersController {
    @Get(':id')
    findOne(@Param('id') id: string): Promise<User> { ... }

    @Get()
    findAll(@Query('page') page: string, @Query('limit') limit: string): Promise<User[]> { ... }

    @Post()
    create(@Body() dto: CreateUserDto, @Headers('authorization') auth: string): Promise<User> { ... }
}

// → Axum
async fn find_one(Path(id): Path<String>) -> Result<Json<User>, AppError> { ... }
async fn find_all(Query(params): Query<FindAllParams>) -> Result<Json<Vec<User>>, AppError> { ... }
async fn create(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(dto): Json<CreateUserDto>
) -> Result<Json<User>, AppError> { ... }
```

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/class/method.rs` — @Param, @Query, @Headers
- Modify: `crates/tyrus_codegen/src/convert/class/routing.rs` — Route parameter syntax
- Create: `tests/fixtures/tier4/controller_params.ts`
- Create: `tests/src/unit/tier4_params.rs`

#### Tasks

- [ ] **Task 7.1.1: Write failing test for @Param decorator**

  ```rust
  #[test]
  fn test_param_decorator_generates_path_extractor() {
      let code = transpile(r#"
  import { Controller, Get, Param } from '@nestjs/common';
  @Controller('/users')
  class UsersController {
      @Get(':id')
      findOne(@Param('id') id: string): string {
          return id;
      }
  }
  "#);
      assert!(code.contains("Path("));
  }
  ```

- [ ] **Task 7.1.2: Implement @Param → Path extractor**

  In `class/method.rs`, detect `@Param('name')` decorator on parameters:
  - Single param: `Path(name): Path<String>`
  - Multiple params: Generate params struct, `Path(params): Path<ParamsStruct>`

- [ ] **Task 7.1.3: Write test for @Query decorator**

  ```rust
  #[test]
  fn test_query_decorator_generates_query_extractor() {
      let code = transpile(r#"
  import { Controller, Get, Query } from '@nestjs/common';
  @Controller('/items')
  class ItemsController {
      @Get()
      findAll(@Query('page') page: string): string {
          return page;
      }
  }
  "#);
      assert!(code.contains("Query("));
  }
  ```

- [ ] **Task 7.1.4: Implement @Query → Query extractor**

  - Single query param: `Query(name): Query<String>`
  - Multiple query params: Generate query struct with optional fields

- [ ] **Task 7.1.5: Write test for @Headers decorator**

  ```rust
  #[test]
  fn test_headers_decorator() {
      let code = transpile(r#"
  import { Controller, Get, Headers } from '@nestjs/common';
  @Controller('/api')
  class ApiController {
      @Get()
      getInfo(@Headers('authorization') auth: string): string {
          return auth;
      }
  }
  "#);
      assert!(code.contains("TypedHeader") || code.contains("HeaderMap"));
  }
  ```

- [ ] **Task 7.1.6: Implement @Headers → header extraction**

- [ ] **Task 7.1.7: Commit**

  ```bash
  git commit -m "feat(codegen): @Param, @Query, @Headers decorator transpilation"
  ```

---

### Milestone 7.2: HTTP Response Configuration

**Why:** NestJS controllers use @HttpCode, @Header, @Redirect to configure responses.

**TypeScript → Rust Mapping:**
```typescript
@Post()
@HttpCode(201)
create(@Body() dto: CreateUserDto): Promise<User> { ... }

// → Axum (must be Result to support error propagation from try-catch)
async fn create(Json(dto): Json<CreateUserDto>) -> Result<(StatusCode, Json<User>), AppError> {
    // ... returns Result wrapping (StatusCode, Json) tuple
    Ok((StatusCode::CREATED, Json(user)))
}
```

**Note:** Return type MUST be `Result<(StatusCode, Json<T>), AppError>`, NOT bare `(StatusCode, Json<T>)`.
If the handler uses try-catch (Phase 6.1), it needs `Result` for error propagation. Using
`Result<(StatusCode, Json<T>), AppError>` keeps both `?` operator and custom status codes working.

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/class/method.rs` — @HttpCode
- Modify: `crates/tyrus_codegen/src/convert/class/routing.rs` — Response types

#### Tasks

- [ ] **Task 7.2.1: Write test for @HttpCode**

  ```rust
  #[test]
  fn test_http_code_decorator() {
      let code = transpile(r#"
  import { Controller, Post, HttpCode, Body } from '@nestjs/common';
  interface Item { name: string; }
  @Controller('/items')
  class ItemsController {
      @Post()
      @HttpCode(201)
      create(@Body() item: Item): Item {
          return item;
      }
  }
  "#);
      assert!(code.contains("StatusCode"));
      assert!(code.contains("201") || code.contains("CREATED"));
  }
  ```

- [ ] **Task 7.2.2: Implement @HttpCode → StatusCode tuple return**

  Detect `@HttpCode(code)` decorator:
  - Return type becomes `Result<(StatusCode, Json<T>), AppError>` (wraps StatusCode in Result for error propagation compatibility)
  - Map common codes: 200→OK, 201→CREATED, 204→NO_CONTENT, etc.
  - Default (no @HttpCode): `Result<Json<T>, AppError>` (current behavior, 200 implicit)

- [ ] **Task 7.2.3: Commit**

  ```bash
  git commit -m "feat(codegen): @HttpCode decorator transpilation"
  ```

---

### Milestone 7.3: HttpException Error Responses

**Why:** NestJS uses typed exceptions for error responses. These must map to Axum error types.

**TypeScript → Rust Mapping:**
```typescript
throw new NotFoundException('User not found');
throw new BadRequestException('Invalid email');
throw new UnauthorizedException('Invalid token');
throw new ForbiddenException('Access denied');
throw new ConflictException('Email already exists');

// → Rust AppError variants
return Err(AppError::NotFound("User not found".into()));
return Err(AppError::BadRequest("Invalid email".into()));
return Err(AppError::Unauthorized("Invalid token".into()));
return Err(AppError::Forbidden("Access denied".into()));
return Err(AppError::Conflict("Email already exists".into()));
```

**Files:**
- Modify: `crates/tyrus_orchestrator/src/format.rs` — Expand AppError enum
- Modify: `crates/tyrus_codegen/src/convert/expr/call.rs` — new XxxException()
- Modify: `crates/tyrus_codegen/src/convert/stmt.rs` — throw → return Err

#### Tasks

- [ ] **Task 7.3.1: Expand AppError enum with HTTP status variants**

  Modify `format.rs` `get_app_error_code()`:
  ```rust
  enum AppError {
      NotFound(String),
      BadRequest(String),
      Unauthorized(String),
      Forbidden(String),
      Conflict(String),
      InternalServer(String),
      // ... map to StatusCode in IntoResponse
  }
  ```

- [ ] **Task 7.3.2: Map NestJS exceptions to AppError variants**

  In `expr/call.rs`, detect `new NotFoundException(msg)`:
  - Map to `AppError::NotFound(msg.into())`
  - Support: NotFoundException, BadRequestException, UnauthorizedException, ForbiddenException, ConflictException, InternalServerErrorException

- [ ] **Task 7.3.3: Write unit test for exception mapping**

  ```rust
  #[test]
  fn test_not_found_exception() {
      let code = transpile(r#"
  import { NotFoundException } from '@nestjs/common';
  function findUser(id: string): string {
      throw new NotFoundException("User not found");
  }
  "#);
      assert!(code.contains("NotFound") || code.contains("not_found"));
  }
  ```

- [ ] **Task 7.3.4: Commit**

  ```bash
  git commit -m "feat(codegen): NestJS HttpException hierarchy to AppError mapping"
  ```

---

### Milestone 7.4: Guards and Middleware

**Why:** Authentication in NestJS uses guards. Middleware handles cross-cutting concerns (CORS, logging).

**TypeScript → Rust Mapping:**
```typescript
@Injectable()
class AuthGuard implements CanActivate {
    canActivate(context: ExecutionContext): boolean {
        const request = context.switchToHttp().getRequest();
        return request.headers.authorization !== undefined;
    }
}

@Controller('/protected')
@UseGuards(AuthGuard)
class ProtectedController { ... }

// → Axum middleware/extractor
async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !headers.contains_key("authorization") {
        return Err(AppError::Unauthorized("Missing auth".into()));
    }
    Ok(next.run(request).await)
}
// Applied via .layer() on router
```

**Files:**
- Modify: `crates/tyrus_analyzer/src/decorators.rs` — @UseGuards detection
- Modify: `crates/tyrus_codegen/src/convert/class/routing.rs` — Middleware layers
- Create: `tests/fixtures/tier4/guard.ts`

#### Tasks

- [ ] **Task 7.4.1: Write test for @UseGuards decorator detection**

  ```rust
  #[test]
  fn test_use_guards_generates_middleware() {
      let code = transpile(r#"
  import { Controller, Get, UseGuards, Injectable } from '@nestjs/common';
  @Injectable()
  class AuthGuard {
      canActivate(): boolean { return true; }
  }
  @Controller('/api')
  @UseGuards(AuthGuard)
  class ApiController {
      @Get()
      getData(): string { return "data"; }
  }
  "#);
      assert!(code.contains("layer") || code.contains("middleware"));
  }
  ```

- [ ] **Task 7.4.2: Implement guard → middleware layer transpilation**

  - Detect `@UseGuards(GuardClass)` on controller or method
  - Generate Axum middleware function from `canActivate()` method
  - Apply via `.layer(middleware::from_fn(auth_middleware))` on router

- [ ] **Task 7.4.3: Commit**

  ```bash
  git commit -m "feat(codegen): @UseGuards to Axum middleware layer"
  ```

---

### Milestone 7.5: Enhanced Module System

**Why:** Real NestJS projects have multiple modules importing/exporting services.

**TypeScript → Rust Mapping:**
```typescript
// users.module.ts
@Module({
    imports: [DatabaseModule],
    controllers: [UsersController],
    providers: [UsersService],
    exports: [UsersService],
})
class UsersModule {}

// app.module.ts
@Module({
    imports: [UsersModule, AuthModule],
    controllers: [AppController],
    providers: [AppService],
})
class AppModule {}

// → Rust
// Each module becomes a Rust module with pub exports
// DI graph resolves initialization order
// main.rs instantiates in topological order
```

**Files:**
- Modify: `crates/tyrus_di/src/graph.rs` — Cross-module resolution
- Modify: `crates/tyrus_orchestrator/src/pipeline.rs` — Multi-module coordination
- Modify: `crates/tyrus_orchestrator/src/scaffold.rs` — Module-aware scaffolding
- Create: `tests/fixtures/tier4_multi_module/`

#### Tasks

- [ ] **Task 7.5.1: Create multi-module test fixture**

  Create `tests/fixtures/tier4_multi_module/`:
  ```
  src/
  ├── app.module.ts
  ├── app.controller.ts
  ├── app.service.ts
  └── users/
      ├── users.module.ts
      ├── users.controller.ts
      ├── users.service.ts
      └── dto/
          └── create-user.dto.ts
  ```

- [ ] **Task 7.5.2: Write integration test for multi-module transpilation**

  Test that `build_project()` correctly:
  - Discovers all modules
  - Resolves cross-module dependencies
  - Generates correct mod.rs hierarchy
  - Initializes DI in correct order

- [ ] **Task 7.5.3: Implement cross-module DI resolution**

  Modify `graph.rs`:
  - Track module `exports` → make providers available to importing modules
  - Resolve transitive dependencies through module imports

- [ ] **Task 7.5.4: Implement module-aware scaffolding**

  Modify `scaffold.rs`:
  - Generate `mod.rs` per feature directory
  - Re-export public types
  - Generate imports based on module `imports` array

- [ ] **Task 7.5.5: Commit**

  ```bash
  git commit -m "feat(di): cross-module dependency resolution and scaffolding"
  ```

---

## Phase 8: HTTP Equivalence Testing

**Objective:** Create infrastructure to verify that the transpiled Rust server produces identical HTTP responses to the NestJS server for the same requests.
**Prerequisite:** Phase 7 complete (NestJS decorators working).
**Estimated Scope:** ~20 tasks across 3 milestones.

---

### Milestone 8.1: Reference NestJS Project

**Why:** We need a canonical NestJS project as the "ground truth" to compare against.

#### Tasks

- [ ] **Task 8.1.1: Create reference NestJS project**

  Create `tests/fixtures/reference_nestjs_project/`:
  ```
  src/
  ├── main.ts
  ├── app.module.ts
  ├── app.controller.ts
  ├── health/
  │   ├── health.module.ts
  │   └── health.controller.ts    # GET /health → { status: "ok" }
  ├── users/
  │   ├── users.module.ts
  │   ├── users.controller.ts     # CRUD: GET, POST, PUT, DELETE /users
  │   ├── users.service.ts        # In-memory store (no database)
  │   └── dto/
  │       ├── create-user.dto.ts
  │       └── update-user.dto.ts
  └── items/
      ├── items.module.ts
      ├── items.controller.ts     # GET /items?page=1&limit=10
      ├── items.service.ts
      └── entities/
          └── item.entity.ts
  ```

  **Key constraint:** In-memory data only (no database). This keeps the test hermetic.

- [ ] **Task 8.1.2: Verify NestJS project runs and serves correct responses**

  ```bash
  cd tests/fixtures/reference_nestjs_project
  npm install
  npm run start
  # Test: curl http://localhost:3000/health → {"status":"ok"}
  # Test: curl http://localhost:3000/users → []
  # Test: curl -X POST http://localhost:3000/users -d '{"name":"Alice"}' → {"id":"...","name":"Alice"}
  ```

- [ ] **Task 8.1.3: Create HTTP test script**

  Create `tests/http_equivalence.sh`:
  - Start NestJS server on port 3000
  - Start Rust server on port 3001
  - Send identical requests to both
  - Compare responses (status code, body, headers)
  - Report differences

- [ ] **Task 8.1.4: Commit**

  ```bash
  git commit -m "test: reference NestJS project for HTTP equivalence testing"
  ```

---

### Milestone 8.2: Transpile Reference Project

**Why:** The reference project IS the test case. If we can transpile it and get identical responses, we've achieved the goal.

#### Tasks

- [ ] **Task 8.2.1: Attempt transpilation of reference project**

  ```bash
  tyrus build tests/fixtures/reference_nestjs_project/src -o tests/output/reference_rust
  ```

  Document failures. Each failure becomes a task to fix.

- [ ] **Task 8.2.2: Fix transpilation failures iteratively**

  For each failure:
  1. Write a minimal failing equivalence test
  2. Fix the codegen issue
  3. Verify the test passes
  4. Re-attempt full project transpilation

- [ ] **Task 8.2.3: Compile transpiled project**

  ```bash
  tyrus compile tests/fixtures/reference_nestjs_project/src -o tests/output/reference_rust
  ```

  Fix any Rust compilation errors in the generated code.

- [ ] **Task 8.2.4: Run both servers and compare**

  ```bash
  # Terminal 1: NestJS
  cd tests/fixtures/reference_nestjs_project && npm run start

  # Terminal 2: Rust
  cd tests/output/reference_rust && cargo run

  # Terminal 3: Compare
  bash tests/http_equivalence.sh
  ```

- [ ] **Task 8.2.5: Commit**

  ```bash
  git commit -m "test: transpiled reference project passes HTTP equivalence"
  ```

---

### Milestone 8.3: Automated E2E Comparison

**Why:** Manual comparison doesn't scale. We need automated CI-friendly tests.

#### Tasks

- [ ] **Task 8.3.1: Create Rust-based HTTP equivalence test**

  Create `tests/src/http_equivalence.rs`:
  - Uses `reqwest` to send requests
  - Compares JSON responses
  - Reports detailed diffs

  ```rust
  #[tokio::test]
  async fn test_health_endpoint_equivalence() {
      let nestjs_resp = reqwest::get("http://localhost:3000/health").await.unwrap();
      let rust_resp = reqwest::get("http://localhost:3001/health").await.unwrap();
      assert_eq!(nestjs_resp.status(), rust_resp.status());
      assert_eq!(
          nestjs_resp.json::<serde_json::Value>().await.unwrap(),
          rust_resp.json::<serde_json::Value>().await.unwrap()
      );
  }
  ```

- [ ] **Task 8.3.2: Create test harness that starts both servers**

  Build a test helper that:
  1. Transpiles + compiles the NestJS project
  2. Starts both servers (NestJS on 3000, Rust on 3001)
  3. Waits for both to be ready (health check polling)
  4. Runs all HTTP comparison tests
  5. Shuts down both servers

- [ ] **Task 8.3.3: Write comprehensive endpoint tests**

  Test all CRUD operations:
  - GET /health
  - GET /users (empty list)
  - POST /users (create)
  - GET /users (list with item)
  - GET /users/:id (find one)
  - PUT /users/:id (update)
  - DELETE /users/:id (delete)
  - GET /users/:id (404 after delete)
  - GET /items?page=1&limit=10

- [ ] **Task 8.3.4: Commit**

  ```bash
  git commit -m "test: automated HTTP equivalence testing infrastructure"
  ```

---

## Phase 9: Polish & Production Readiness

**Objective:** Handle edge cases, improve error messages, optimize performance.
**Prerequisite:** Phase 8 complete (reference project transpiles correctly).

---

### Milestone 9.1: Validation & DTOs

**Why:** Real NestJS projects validate input with class-validator.

**Research Decision:** Use `garde` (0.22+) over `validator` crate. Reasons:
1. **Built-in Axum integration** (no manual validation in handlers)
2. **`#[garde(dive)]`** maps directly to `@ValidateNested()`
3. **More validators** (phone, IP, alphanumeric) than `validator`
4. **Context-aware custom validators** (useful for DI-injected validation)

**TypeScript → Rust Mapping:**
```typescript
import { IsString, IsNotEmpty, IsOptional, IsEmail, Min, Max } from 'class-validator';
class CreateUserDto {
    @IsString() @IsNotEmpty() name: string;
    @IsEmail() email: string;
    @Min(0) @Max(150) age: number;
    @IsOptional() bio?: string;
}

// → Rust (with garde crate — research-validated choice)
#[derive(Deserialize, garde::Validate)]
struct CreateUserDto {
    #[garde(length(min = 1))]
    name: String,
    #[garde(email)]
    email: String,
    #[garde(range(min = 0.0, max = 150.0))]
    age: f64,
    #[garde(skip)]
    bio: Option<String>,
}
```

**class-validator → garde Mapping Table:**

| TypeScript | Rust (garde) |
|------------|-------------|
| `@IsNotEmpty()` | `#[garde(length(min = 1))]` |
| `@IsEmail()` | `#[garde(email)]` |
| `@IsUrl()` | `#[garde(url)]` |
| `@Length(min, max)` | `#[garde(length(min = N, max = M))]` |
| `@Min(n)` / `@Max(n)` | `#[garde(range(min = N, max = M))]` |
| `@Matches(regex)` | `#[garde(pattern(r"regex"))]` |
| `@IsOptional()` | `#[garde(skip)]` on `Option<T>` |
| `@ValidateNested()` | `#[garde(dive)]` |
| `@IsPhoneNumber()` | `#[garde(phone_number)]` |

#### Tasks

- [ ] **Task 9.1.1: Add garde to generated Cargo.toml**

  Modify `crates/tyrus_orchestrator/src/scaffold.rs`:
  - When class-validator imports are detected, add `garde = { version = "0.22", features = ["full"] }` to generated deps

- [ ] **Task 9.1.2: Detect class-validator decorators in analyzer**

  Modify `crates/tyrus_analyzer/src/decorators.rs`:
  - Extract validation decorators from class properties
  - Store in decorator metadata for codegen consumption

- [ ] **Task 9.1.3: Generate garde attributes on DTO structs**

  Modify `crates/tyrus_codegen/src/convert/class/mod.rs`:
  - When a class has validation decorators, add `#[derive(garde::Validate)]`
  - Map each decorator to corresponding `#[garde(...)]` attribute

- [ ] **Task 9.1.4: Generate validation middleware for @Body()**

  Modify `crates/tyrus_codegen/src/convert/class/method.rs`:
  - When `@Body()` param has a type with validation decorators:
  - Generate: `Json(body): Json<T>` + `body.validate(&()).map_err(|e| AppError::BadRequest(e.to_string()))?`

- [ ] **Task 9.1.5: Write equivalence test for validation errors**

  Test that NestJS validation errors match Rust validation errors (HTTP 400 with field-level messages).

- [ ] **Task 9.1.6: Commit**

  ```bash
  git commit -m "feat(codegen): class-validator to garde validation transpilation"
  ```

---

### Milestone 9.2: Logging & Observability

**Why:** NestJS uses built-in Logger. Rust project should use tracing.

#### Tasks

- [ ] **Task 9.2.1: Map NestJS Logger to tracing macros**
- [ ] **Task 9.2.2: Add tracing subscriber setup in main.rs scaffold**
- [ ] **Task 9.2.3: Commit**

---

### Milestone 9.3: Configuration

**Why:** NestJS uses ConfigService. Rust should use environment variables.

#### Tasks

- [ ] **Task 9.3.1: Map ConfigService.get() to std::env::var()**
- [ ] **Task 9.3.2: Generate .env loading with dotenv crate**
- [ ] **Task 9.3.3: Commit**

---

## Phase 10: Academic & Documentation

**Objective:** Benchmarks, formal paper, complete documentation.
**Prerequisite:** Phase 9 complete.

---

### Milestone 10.1: Performance Benchmarks

- [ ] **Task 10.1.1: Transpilation speed benchmarks with Criterion.rs**
- [ ] **Task 10.1.2: Runtime benchmarks (NestJS vs Rust server)**
- [ ] **Task 10.1.3: Memory usage comparison**
- [ ] **Task 10.1.4: Latency comparison under load (wrk/k6)**
- [ ] **Task 10.1.5: Commit**

---

### Milestone 10.2: Formal Specification

- [ ] **Task 10.2.1: Complete EBNF grammar for Oxidizable Standard**
- [ ] **Task 10.2.2: Type mapping formal specification**
- [ ] **Task 10.2.3: NestJS → Axum mapping formal specification**
- [ ] **Task 10.2.4: Commit**

---

### Milestone 10.3: Academic Paper

- [ ] **Task 10.3.1: Write TCC/paper structure**
- [ ] **Task 10.3.2: Results section with benchmarks**
- [ ] **Task 10.3.3: Related work survey**
- [ ] **Task 10.3.4: Conclusions and future work**
- [ ] **Task 10.3.5: Commit**

---

### Milestone 10.4: Transpiler Fuzzing & Property Testing (Research-Driven)

**Why:** Research (TransFuzz, SANER 2023; RustAssure, 2025) shows that grammar-guided fuzzing and property-based testing find 47-385% more bugs than hand-written tests alone.

#### Tasks

- [ ] **Task 10.4.1: Add cargo-fuzz for transpiler robustness**

  Create `fuzz/` directory with fuzz targets:
  - `fuzz_parser`: Feed random/mutated TS to parser, verify no panics
  - `fuzz_codegen`: Feed valid SWC ASTs to codegen, verify no panics
  - `fuzz_pipeline`: Full transpilation pipeline, verify no panics

- [ ] **Task 10.4.2: Add proptest for semantic properties**

  Add to `tests/Cargo.toml`: `proptest = "1.0"`

  Property tests:
  - "For any valid numeric expression, TS and Rust produce the same f64"
  - "For any valid string operation, TS and Rust produce the same String"
  - "Generated Rust always compiles" (for valid Oxidizable TS input)

- [ ] **Task 10.4.3: Grammar-guided AST mutation (TransFuzz approach)**

  Create `tests/src/mutation/` module:
  - Mutate existing fixture ASTs (swap operators, change literals, add/remove statements)
  - Run mutations through transpiler + Node.js + Rust comparison
  - Report any output divergence

- [ ] **Task 10.4.4: Commit**

  ```bash
  git commit -m "test: transpiler fuzzing and property-based testing"
  ```

---

## Summary: Execution Order & Dependencies

```
Phase 6: TypeScript Language Completeness
├── 6.0 Infrastructure upgrade (P0, no dependencies — DO FIRST)
├── 6.1 try-catch → Result (CRITICAL, depends on 6.0)
├── 6.2 Top-level statements (CRITICAL, depends on 6.0)
├── 6.3 Spread/rest (IMPORTANT, depends on 6.0)
├── 6.4 Class inheritance (IMPORTANT, depends on 6.0)
├── 6.5 Static/getters/setters (IMPORTANT, depends on 6.0 — independent of 6.4)
└── 6.6 Type assertions/enums (NICE, depends on 6.0)

Phase 7: NestJS Framework Completeness (depends on Phase 6)
├── 7.1 @Param/@Query/@Headers (CRITICAL, depends on 6.1)
├── 7.2 @HttpCode response config (IMPORTANT, depends on 7.1)
├── 7.3 HttpException hierarchy (CRITICAL, depends on 6.1, 6.2)
├── 7.4 Guards/middleware (IMPORTANT, depends on 7.1)
└── 7.5 Multi-module system (CRITICAL, depends on 6.2 + existing DI system)

Phase 8: HTTP Equivalence (depends on Phase 7)
├── 8.1 Reference NestJS project (no code deps — can start during Phase 7)
├── 8.2 Transpile reference project (depends on 7.*)
└── 8.3 Automated E2E comparison (depends on 8.2)

Phase 9: Polish (depends on Phase 8)
├── 9.1 Validation/DTOs (garde crate)
├── 9.2 Logging (tracing)
└── 9.3 Configuration (dotenv)

Phase 10: Academic (depends on Phase 9)
├── 10.1 Benchmarks (Criterion.rs + wrk/k6)
├── 10.2 Formal spec (EBNF)
├── 10.3 Paper (TCC)
└── 10.4 Fuzzing/property testing (cargo-fuzz + proptest)
```

## Metrics & Success Criteria

| Phase | Success Metric |
|-------|----------------|
| Phase 6 | All new equivalence tests pass. Existing 157 tests still pass. |
| Phase 7 | Full NestJS controller with CRUD routes transpiles and compiles. |
| Phase 8 | Reference NestJS project and transpiled Rust project return identical responses for all endpoints. |
| Phase 9 | Validation errors, logging output, and config loading produce equivalent behavior. |
| Phase 10 | Paper published. Benchmarks show Rust is 5-20x faster than NestJS for same workload. |

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| try-catch semantics divergence | High | Limit to synchronous try-catch first, add async later. Research confirms closure+match pattern is community consensus. |
| Class inheritance complexity | Medium | Use trait+composition+enum_dispatch (validated by gtk-rs, enum_dispatch crate). NOT dyn Trait. |
| Guard/middleware pattern mismatch | Medium | Start with simple function middleware via `tower::middleware::from_fn`. See Axum middleware docs. |
| Optional chaining double-Option | Low | Known limitation, defer to Phase 6 type inference |
| Multi-module DI complexity | High | Start with linear modules (no circular), add graph resolution later |
| SWC AST changes | Low | Pin SWC version, update periodically |
| Generated Rust doesn't compile | High | Every milestone includes compilation test + trybuild (P0 in 6.0) |
| Node.js version mismatch | Low | Pin Node 22+ with --experimental-strip-types |
| prettyplease output diverges from rustfmt | Low | prettyplease produces ~97-98% identical output to rustfmt. Snapshot tests will catch any regressions. |
| garde validation error format differs from NestJS | Medium | Test validation error shapes explicitly in Phase 9.1. May need custom error formatter. |

---

## Research References

Academic papers and industry projects that informed this plan:

### Academic Papers
| Paper | Venue | Key Insight |
|-------|-------|-------------|
| Safe & Efficient Gradual Typing for TypeScript | POPL 2015 | Strict TS subset can have sound static typing — validates "Oxidizable Standard" |
| Ownership Guided C to Rust Translation | CAV 2023 | Ownership inference succeeds for ~11% of pointers — restricting input is correct |
| Aliasing Limits on Translating C to Safe Rust | OOPSLA 2023 | Fundamental limits of safe Rust translation — validates subset approach |
| TransFuzz: Detecting JS Transpiler Bugs | SANER 2023 | Grammar-guided AST mutation finds 47-385% more bugs |
| RustAssure: Differential Symbolic Testing | arXiv 2025 | Symbolic equivalence catches semantic divergence across all inputs |
| VERT: Verified Equivalent Rust Transpilation | arXiv 2024 | LLM-assisted transpilation with verification |

### Industry Projects
| Project | Stars | Pattern Adopted |
|---------|-------|-----------------|
| AssemblyScript | 17841 | Strict TS subset → compiled target (same as Oxidizable Standard) |
| ts2c | 1358 | TS→C89 via strict subset (most mature in category) |
| C2Rust | 4656 | Two-phase: correct first, then idiomatic. prettyplease for formatting. |
| Haxe | 6857 | Multi-target transpiler architecture, extern definitions |
| enum_dispatch | ~700 | 10x performance over dyn Trait for closed hierarchies |

### Crate Decisions
| Decision | Choice | Alternative | Why |
|----------|--------|-------------|-----|
| Formatting | prettyplease | rustfmt subprocess | No external dependency, handles async, 10x faster |
| Validation | garde 0.22 | validator 0.20 | Built-in Axum integration, #[garde(dive)] for nested, more validators |
| Error types | thiserror 2.0 | anyhow | Structured variants needed for HTTP status mapping |
| Inheritance | enum_dispatch | dyn Trait | 10x perf, static dispatch, exhaustive matching |
| Compile testing | trybuild | manual cargo check | Standard in Rust ecosystem, snapshot-based |
