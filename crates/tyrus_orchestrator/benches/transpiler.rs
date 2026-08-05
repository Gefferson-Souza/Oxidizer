#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::hint::black_box;
use std::io::Write;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
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
// Tier 1: Basic TypeScript (primitives, functions, control flow)
// ---------------------------------------------------------------------------

const TIER1_FUNCTIONS: &str = r"
function square(x: number): number {
    return x * x;
}

function isPositive(n: number): boolean {
    return n > 0;
}

function formatUser(name: string, age: number): string {
    return `${name} is ${age} years old`;
}
";

const TIER1_CONTROL_FLOW: &str = r#"
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

function classify(x: number): string {
    if (x > 0) {
        return "positive";
    } else if (x < 0) {
        return "negative";
    } else {
        return "zero";
    }
}
"#;

// ---------------------------------------------------------------------------
// Tier 2: Intermediate (interfaces, classes, arrays)
// ---------------------------------------------------------------------------

const TIER2_INTERFACES: &str = r"
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
";

const TIER2_CLASS: &str = r"
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
";

const TIER2_ARRAYS: &str = r"
function doubleAll(nums: number[]): number[] {
    return nums.map((n: number) => n * 2);
}

function evens(nums: number[]): number[] {
    return nums.filter((n: number) => n % 2 === 0);
}

function sum(nums: number[]): number {
    let total: number = 0;
    nums.forEach((n: number) => {
        total = total + n;
    });
    return total;
}
";

// ---------------------------------------------------------------------------
// Tier 3: Advanced (generics, stdlib, methods)
// ---------------------------------------------------------------------------

const TIER3_STDLIB: &str = r"
function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
}

function roundToTwo(n: number): number {
    return Math.round(n * 100) / 100;
}

function findFirst(nums: number[]): number {
    const found: number = nums.find((n: number) => n > 10) ?? 0;
    return found;
}

function hasLarge(nums: number[]): boolean {
    return nums.some((n: number) => n > 100);
}

function allPositive(nums: number[]): boolean {
    return nums.every((n: number) => n > 0);
}
";

// ---------------------------------------------------------------------------
// Tier 4: NestJS / Enterprise (decorators, DI, controllers)
// ---------------------------------------------------------------------------

const TIER4_NESTJS: &str = r#"
import { Controller, Get, Post, Body } from "@nestjs/common";
import { Injectable } from "@nestjs/common";

interface CreateUserDto {
    name: string;
    email: string;
}

interface User {
    id: number;
    name: string;
    email: string;
}

@Injectable()
class UsersService {
    private users: User[];

    constructor() {
        this.users = [];
    }

    findAll(): User[] {
        return this.users;
    }

    create(dto: CreateUserDto): User {
        const user: User = {
            id: this.users.length + 1,
            name: dto.name,
            email: dto.email,
        };
        this.users.push(user);
        return user;
    }
}

@Controller("/users")
class UsersController {
    constructor(private usersService: UsersService) {}

    @Get("/")
    findAll(): User[] {
        return this.usersService.findAll();
    }

    @Post("/")
    create(@Body() dto: CreateUserDto): User {
        return this.usersService.create(dto);
    }
}
"#;

// ---------------------------------------------------------------------------
// Combined: all tiers in one file (scalability test)
// ---------------------------------------------------------------------------

const COMBINED_ALL_TIERS: &str = r"
function square(x: number): number { return x * x; }
function isPositive(n: number): boolean { return n > 0; }
function formatUser(name: string, age: number): string {
    return `${name} is ${age} years old`;
}

function max(a: number, b: number): number {
    if (a > b) { return a; } else { return b; }
}

function countdown(n: number): number {
    let result: number = 0;
    let i: number = n;
    while (i > 0) { result = result + i; i = i - 1; }
    return result;
}

interface Config {
    host: string;
    port: number;
    debug: boolean;
}

interface ApiResponse {
    data: string[];
    total: number;
    success: boolean;
}

class Counter {
    private count: number;
    constructor() { this.count = 0; }
    increment(): number { this.count = this.count + 1; return this.count; }
    getCount(): number { return this.count; }
}

function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
}

function roundToTwo(n: number): number {
    return Math.round(n * 100) / 100;
}
";

// ---------------------------------------------------------------------------
// Benchmark Group 1: Full pipeline by tier (parse → analyze → codegen → format)
// ---------------------------------------------------------------------------

fn bench_full_pipeline_by_tier(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    let tiers: &[(&str, &str)] = &[
        ("tier1_functions", TIER1_FUNCTIONS),
        ("tier1_control_flow", TIER1_CONTROL_FLOW),
        ("tier2_interfaces", TIER2_INTERFACES),
        ("tier2_class", TIER2_CLASS),
        ("tier2_arrays", TIER2_ARRAYS),
        ("tier3_stdlib", TIER3_STDLIB),
        ("tier4_nestjs", TIER4_NESTJS),
        ("combined_all", COMBINED_ALL_TIERS),
    ];

    for (name, source) in tiers {
        let (_tmp, path) = ts_temp_file(source);

        group.bench_with_input(BenchmarkId::new("transpile", name), source, |b, _| {
            b.iter(|| {
                let result = tyrus_orchestrator::build(black_box(&path));
                assert!(result.is_ok());
                result
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark Group 2: Individual pipeline stages
// ---------------------------------------------------------------------------

fn bench_pipeline_stages(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_stages");

    // Use combined input for stage-level benchmarks
    let (_tmp, path) = ts_temp_file(COMBINED_ALL_TIERS);

    // Stage 1: Parse only
    group.bench_function("1_parse", |b| {
        b.iter(|| {
            let program = tyrus_parser::parse(black_box(path.as_ref()));
            assert!(program.is_ok());
            program
        });
    });

    // Stage 2: Parse + Analyze
    group.bench_function("2_parse_analyze", |b| {
        b.iter(|| {
            let program = tyrus_parser::parse(black_box(path.as_ref())).unwrap();
            let source = std::fs::read_to_string(path.as_ref()).unwrap();
            let file_name = path.as_ref().to_string_lossy().to_string();
            tyrus_analyzer::Analyzer::analyze(&program, source, file_name)
        });
    });

    // Stage 3: Parse + Codegen (skip analyzer for isolation)
    group.bench_function("3_parse_codegen", |b| {
        b.iter(|| {
            let program = tyrus_parser::parse(black_box(path.as_ref())).unwrap();
            tyrus_codegen::generate(&program, false)
        });
    });

    // Stage 4: Full pipeline (parse + codegen + format)
    group.bench_function("4_full_pipeline", |b| {
        b.iter(|| {
            let result = tyrus_orchestrator::build(black_box(&path));
            assert!(result.is_ok());
            result
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark Group 3: Scalability (increasing code size)
// ---------------------------------------------------------------------------

fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    // Generate inputs of increasing size
    let sizes: &[usize] = &[1, 5, 10, 20];

    for &count in sizes {
        let fns: Vec<String> = (0..count)
            .map(|i| format!("function func_{i}(x: number): number {{ return x * {i}; }}"))
            .collect();
        let source = format!("{}\n", fns.join("\n"));

        let label = format!("{count}_functions");
        let (_tmp, path) = ts_temp_file(&source);

        group.bench_with_input(BenchmarkId::new("scale", &label), &count, |b, _| {
            b.iter(|| {
                let result = tyrus_orchestrator::build(black_box(&path));
                assert!(result.is_ok());
                result
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion harness
// ---------------------------------------------------------------------------

criterion_group!(
    transpiler_benches,
    bench_full_pipeline_by_tier,
    bench_pipeline_stages,
    bench_scalability,
);
criterion_main!(transpiler_benches);
