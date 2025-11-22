# 🗺️ Oxidizer Roadmap: The Path to Production

> **Goal:** Turn a full Node.js/TypeScript backend project into a compilable Rust workspace with a single command.

---

## 🟢 Phase 1: Core Syntax (COMPLETED - 40%)
*The engine is ready. We can translate logic, data structures, and async calls.*
- [x] Interfaces -> Structs
- [x] Classes -> Impl blocks
- [x] Async/Await -> Tokio
- [x] Functions & Math
- [x] Basic HTTP (Axios/Fetch -> Reqwest)
- [x] Linter (Analyzer)

---

## 🟡 Phase 2: The Project Architect (NEXT UP - 60%)
*Focus: Files, Folders, and Modules. Making files talk to each other.*
- [ ] **File System Walker:** Recursively read `src/**/*.ts`.
- [ ] **Module Resolution (The Hard Part):**
    - [ ] Translate `import/export` to `use/pub mod`.
    - [ ] Auto-generate `mod.rs` files for folders.
    - [ ] Handle relative paths (`../../utils`) conversion to crate paths (`crate::utils`).
- [ ] **Output Mirroring:** Replicate the input folder structure in the output directory.

---

## 🟠 Phase 3: The Package Manager (75%)
*Focus: Dependencies and Configuration.*
- [ ] **Manifest Converter:** Parse `package.json`.
- [ ] **Dependency Mapper:** Map common NPM packages to Crates.io equivalents.
    - `uuid` -> `uuid`
    - `dotenv` -> `dotenvy`
    - `winston` -> `tracing`
    - `zod` -> `validator`
- [ ] **Cargo Gen:** Generate a valid `Cargo.toml` with the correct dependencies features.

---

## 🔴 Phase 4: Server & Database (90%)
*Focus: The "Backend" part (Frameworks & DB).*
- [ ] **Web Framework Adapter:**
    - Detect `express` or `fastify` or `NestJS`.
    - Transpile routes to `axum` (recommended) or `actix-web`.
    - Handle Middleware conversion (Hard).
- [ ] **Database Adapter:**
    - Detect `Prisma` schema or `TypeORM` entities.
    - Suggest or generate `SeaORM` entities (closest to generic ORMs).

---

## 🔵 Phase 5: Safety & Polish (100%)
*Focus: Error Handling and Refactoring.*
- [ ] **Error Propagation:** Convert `try/catch` blocks into `Result` handling.
- [ ] **Refactoring Hints:** Identify patterns that represent technical debt in Rust (e.g., excessive cloning) and suggest fixes.
- [ ] **CI/CD Generation:** Auto-generate GitHub Actions for the new Rust project.