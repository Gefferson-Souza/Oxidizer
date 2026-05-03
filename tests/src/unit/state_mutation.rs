//! Regression tests for #129 — `std::sync::Mutex` re-entrance deadlocks
//! when state fields are read or written multiple times in one statement.
//!
//! Two failure modes both rooted in `MutexGuard` temporary lifetime:
//!   1. Compound assigns (`this.x += N`) silently miscompile to plain `=`
//!      (the read-then-write split previously only handled `=`).
//!   2. Reading the same state field twice in one statement (struct literal
//!      arg, function call args) keeps two guards live → deadlock at runtime.
//!
//! These tests assert the generated Rust no longer contains the
//! double-lock-in-one-statement pattern and that it compiles cleanly.

use crate::helpers::transpile;
use tyrus_test_utils::assert_rust_compiles;

const SERVICE_TEMPLATE: &str = r#"
import { Injectable } from "@nestjs/common";

interface User {
    id: number;
    name: string;
}

@Injectable()
class UsersService {
    counter: number;
    users: User[];

    constructor() {
        this.counter = 0;
        this.users = [];
    }

    PLACEHOLDER_BODY
}
"#;

fn render_service(method_body: &str) -> String {
    SERVICE_TEMPLATE.replace("PLACEHOLDER_BODY", method_body)
}

/// `*self.counter.lock()...! += 1` is the broken inline pattern; if any
/// generated method body contains it, the binary deadlocks at runtime.
fn assert_no_inline_compound_on_lock(rust: &str) {
    for op in ["+=", "-=", "*=", "/=", "%="] {
        let pattern = format!(".lock().unwrap_or_else(|e| e.into_inner()) {op} ");
        assert!(
            !rust.contains(&pattern),
            "Generated Rust contains broken inline compound on locked guard: {pattern}\n--- code ---\n{rust}"
        );
    }
}

#[test]
fn compound_addassign_emits_read_then_write() {
    let ts = render_service(
        r#"
    bumpCounter(): void {
        this.counter += 7;
    }"#,
    );
    let rust = transpile(&ts);

    assert_no_inline_compound_on_lock(&rust);
    assert!(
        rust.contains("let __new_val")
            || rust.contains("let __next")
            || rust.contains("__new_val ="),
        "Expected a temporary binding before the write-back, got: {rust}"
    );
    // Discriminator vs silent miscompile to plain `=`: the literal `7` must
    // appear with an addition. Without the operator, `this.counter += 7`
    // would compile to `*lock = 7f64` (effectively `=`), losing the bump.
    assert!(
        rust.contains("+ 7f64") || rust.contains("+ 7.0") || rust.contains("+ 7)"),
        "Operator dropped — `+= 7` silently became `= 7`. Got:\n{rust}"
    );
}

#[test]
fn compound_subassign_emits_read_then_write() {
    let ts = render_service(
        r#"
    drain(amount: number): void {
        this.counter -= amount;
    }"#,
    );
    let rust = transpile(&ts);
    assert_no_inline_compound_on_lock(&rust);
    assert!(
        rust.contains("- amount") || rust.contains("- amount)"),
        "Operator dropped — `-= amount` silently became `= amount`. Got:\n{rust}"
    );
}

#[test]
fn update_expr_postincrement_emits_safe_form() {
    let ts = render_service(
        r#"
    next(): void {
        this.counter++;
    }"#,
    );
    let rust = transpile(&ts);
    // The unsafe form would be `*self.counter.lock(...) += 1f64` (single statement,
    // single guard, currently safe) — but for parity with compound assigns we
    // also route this through the read-then-write split. Either way, the broken
    // pattern from `assert_no_inline_compound_on_lock` must not appear.
    assert_no_inline_compound_on_lock(&rust);
}

#[test]
fn dual_read_in_struct_literal_uses_guarded_blocks() {
    let ts = render_service(
        r#"
    snapshot(): User {
        const u: User = { id: this.counter, name: "row-" + this.counter };
        return u;
    }"#,
    );
    let rust = transpile(&ts);

    let lock_count = rust.matches(".lock()").count();
    let guarded_block_count = rust.matches("let __g = ").count();

    assert!(
        guarded_block_count >= 2 || rust.contains("let __counter_read"),
        "Expected guarded-block reads for state field appearing twice in one statement. \
         Found {guarded_block_count} guarded block(s) and {lock_count} raw .lock() calls.\n--- code ---\n{rust}"
    );
}

#[test]
fn dual_read_in_call_args_uses_guarded_blocks() {
    let ts = render_service(
        r#"
    range(): number {
        return Math.max(this.counter, this.counter);
    }"#,
    );
    let rust = transpile(&ts);

    let lock_count = rust.matches(".lock()").count();
    let guarded_block_count = rust.matches("let __g = ").count();

    assert!(
        guarded_block_count >= 2 || lock_count == 0,
        "Expected guarded-block reads when state field is read twice in same statement. \
         Found {guarded_block_count} guarded block(s).\n--- code ---\n{rust}"
    );
}

#[test]
fn state_compound_assign_compiles() {
    let ts = render_service(
        r#"
    bumpCounter(): void {
        this.counter += 1;
        this.counter -= 1;
        this.counter *= 2;
    }"#,
    );
    let rust = transpile(&ts);
    assert_rust_compiles(&rust);
}

#[test]
fn state_dual_read_compiles() {
    let ts = render_service(
        r#"
    snapshot(): User {
        const u: User = { id: this.counter, name: "row-" + this.counter };
        return u;
    }"#,
    );
    let rust = transpile(&ts);
    assert_rust_compiles(&rust);
}

#[test]
fn state_update_expr_compiles() {
    let ts = render_service(
        r#"
    next(): void {
        this.counter++;
    }
    prev(): void {
        this.counter--;
    }"#,
    );
    let rust = transpile(&ts);
    assert_rust_compiles(&rust);
}
