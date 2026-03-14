// Hand-optimized Rust: Text Processing 200K
// Key optimizations vs Tyrus:
// 1. &str instead of String::from() for literals
// 2. write!() macro instead of nested format!()
// 3. String::with_capacity for pre-allocation
// 4. Direct &str comparison instead of String == String
// 5. usize counters instead of f64

use std::fmt::Write;

fn text_processing(entries: usize) -> String {
    let mut error_count = 0_usize;
    let mut warn_count = 0_usize;
    let mut total_length = 0_usize;

    let mut line = String::with_capacity(80);

    for i in 0..entries {
        line.clear();
        let hour = i % 24;
        let minute = i % 60;
        let user_id = i % 1000;
        let method = match i % 3 { 0 => "GET", 1 => "POST", _ => "PUT" };
        let status = if i % 7 == 0 { "500" } else if i % 5 == 0 { "404" } else { "200" };

        write!(line, "2026-03-14T{}:{} {} /api/v1/users/{} {}",
            hour, minute, method, user_id, status).unwrap();

        // Analyze without allocating
        total_length += line.len();

        if status == "500" { error_count += 1; }
        if status == "404" { warn_count += 1; }
        if line.contains("/users/42") { /* noop */ }

        let trimmed = line.trim();
        if trimmed.starts_with("2026") { total_length += 1; }
    }

    format!("{},{},{}", error_count, warn_count, total_length)
}

fn main() {
    println!("{}", text_processing(200_000));
}
