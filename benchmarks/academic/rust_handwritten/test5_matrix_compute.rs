// Hand-optimized Rust: Matrix 4Kx4K
// Key optimizations vs Tyrus:
// 1. Pre-compute all coordinates (cache-friendly)
// 2. usize counters (integer arithmetic)
// 3. Direct f64 math (no `as f64` casts on already-f64 values)
// 4. Avoid redundant trig calls (store points in array)

fn matrix_compute(n: usize) -> i64 {
    // Pre-compute all points (cache locality)
    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        let fi = i as f64;
        points.push(((fi * 0.1).sin() * 100.0, (fi * 0.15).cos() * 100.0));
    }

    let mut total_distance = 0.0_f64;
    let mut comparisons = 0_usize;

    for i in 0..n {
        let (xi, yi) = points[i];
        for j in (i + 1)..n {
            let (xj, yj) = points[j];
            let dx = xi - xj;
            let dy = yi - yj;
            total_distance += (dx * dx + dy * dy).sqrt();
            comparisons += 1;
        }
    }

    (total_distance / comparisons as f64).floor() as i64
}

fn main() {
    println!("{}", matrix_compute(4_000));
}
