#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::Write;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::NamedTempFile;
use tyrus_common::fs::FilePath;

/// Write TypeScript source to a temporary file and return the file handle
/// and its corresponding `FilePath`. The temp file stays alive as long as
/// the returned `NamedTempFile` is in scope.
fn ts_temp_file(source: &str) -> (NamedTempFile, FilePath) {
    let mut tmp = tempfile::Builder::new()
        .suffix(".ts")
        .tempfile()
        .expect("failed to create temp file");
    tmp.write_all(source.as_bytes())
        .expect("failed to write ts source");
    tmp.flush().expect("failed to flush temp file");
    let path: FilePath = tmp.path().to_path_buf().into();
    (tmp, path)
}

// ---------------------------------------------------------------------------
// TypeScript source snippets
// ---------------------------------------------------------------------------

const SIMPLE_FUNCTION_TS: &str = r#"
function square(x: number): number {
    return x * x;
}

function isPositive(n: number): boolean {
    return n > 0;
}

function formatUser(name: string, age: number): string {
    return `${name} is ${age} years old`;
}
"#;

const INTERFACE_TS: &str = r#"
interface User {
    name: string;
    age: number;
    email: string;
    active: boolean;
}

interface Product {
    id: number;
    title: string;
    price: number;
    description?: string;
}

interface ApiResponse {
    data: User[];
    total: number;
    success: boolean;
}
"#;

const CLASS_TS: &str = r#"
class Calculator {
    private result: number;

    constructor() {
        this.result = 0;
    }

    add(value: number): number {
        this.result = this.result + value;
        return this.result;
    }

    getResult(): number {
        return this.result;
    }

    reset(): void {
        this.result = 0;
    }
}
"#;

const COMBINED_TS: &str = r#"
function max(a: number, b: number): number {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

function countdown(n: number): number {
    let result: number = 0;
    let i: number = n;
    while (i > 0) {
        result = result + i;
        i = i - 1;
    }
    return result;
}

interface Config {
    host: string;
    port: number;
    debug: boolean;
}

class Counter {
    private count: number;

    constructor() {
        this.count = 0;
    }

    increment(): number {
        this.count = this.count + 1;
        return this.count;
    }

    getCount(): number {
        return this.count;
    }
}
"#;

// ---------------------------------------------------------------------------
// Benchmark functions
// ---------------------------------------------------------------------------

fn bench_simple_function(c: &mut Criterion) {
    let (_tmp, path) = ts_temp_file(SIMPLE_FUNCTION_TS);

    c.bench_function("transpile_simple_functions", |b| {
        b.iter(|| {
            let result = tyrus_orchestrator::build(black_box(&path));
            assert!(result.is_ok());
            result
        });
    });
}

fn bench_interface(c: &mut Criterion) {
    let (_tmp, path) = ts_temp_file(INTERFACE_TS);

    c.bench_function("transpile_interfaces", |b| {
        b.iter(|| {
            let result = tyrus_orchestrator::build(black_box(&path));
            assert!(result.is_ok());
            result
        });
    });
}

fn bench_class(c: &mut Criterion) {
    let (_tmp, path) = ts_temp_file(CLASS_TS);

    c.bench_function("transpile_class_with_methods", |b| {
        b.iter(|| {
            let result = tyrus_orchestrator::build(black_box(&path));
            assert!(result.is_ok());
            result
        });
    });
}

fn bench_combined(c: &mut Criterion) {
    let (_tmp, path) = ts_temp_file(COMBINED_TS);

    c.bench_function("transpile_combined_project", |b| {
        b.iter(|| {
            let result = tyrus_orchestrator::build(black_box(&path));
            assert!(result.is_ok());
            result
        });
    });
}

// ---------------------------------------------------------------------------
// Criterion harness
// ---------------------------------------------------------------------------

criterion_group!(
    transpiler_benches,
    bench_simple_function,
    bench_interface,
    bench_class,
    bench_combined,
);
criterion_main!(transpiler_benches);
