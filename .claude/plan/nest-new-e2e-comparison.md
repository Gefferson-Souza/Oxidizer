# Plan: nest new → Tyrus E2E Comparison

## Task Type
- [x] Backend (→ Claude direct implementation)

## Technical Solution

Create a real `nest new` project, transpile it with Tyrus, and verify both produce identical HTTP responses. This is the ultimate proof-of-concept.

## Gap Analysis Result

| File | Status | Notes |
|------|--------|-------|
| `app.service.ts` | FULLY SUPPORTED | Transpiles correctly |
| `app.controller.ts` | FULLY SUPPORTED | DI, @Get, @Controller all work |
| `app.module.ts` | REPLACED | Tyrus DI graph replaces NestJS runtime DI |
| `main.ts` | REPLACED | scaffold.rs generates Axum main.rs |

**Conclusion:** A `nest new` default project is transpilable TODAY.

## Implementation Steps

### Step 1: Install NestJS CLI and create project
```bash
npm i -g @nestjs/cli
nest new tyrus-test-app --skip-git --package-manager npm
```

### Step 2: Start NestJS server and verify
```bash
cd tyrus-test-app
npm run start
curl http://localhost:3100/  # → "Hello World!"
```

### Step 3: Transpile with Tyrus
```bash
tyrus compile tyrus-test-app/src --output /tmp/tyrus-rust-app
```

### Step 4: Start Rust server and verify
```bash
cd /tmp/tyrus-rust-app
cargo run --release
curl http://localhost:3100/  # → should also return "Hello World!"
```

### Step 5: Compare responses
- GET / → both should return "Hello World!"
- Both should return HTTP 200

### Step 6: Performance comparison (bonus)
```bash
# NestJS
wrk -t4 -c100 -d10s http://localhost:3100/

# Rust
wrk -t4 -c100 -d10s http://localhost:3101/
```

## Risks and Mitigation

| Risk | Mitigation |
|------|------------|
| NestJS CLI not installed | Install via npm |
| Port conflict | Use 3100 for NestJS, 3101 for Rust |
| main.ts not transpiled | Expected — scaffold.rs replaces it |
| app.module.ts empty class | Expected — DI graph replaces it |
| @Controller() with no path | Already handled (defaults to "/") |

## Key Files

| File | Operation | Description |
|------|-----------|-------------|
| tests/fixtures/ | Create | nest new output |
| tests/tests/tier4_tests.rs | Modify | Add E2E comparison test |
