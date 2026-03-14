// Hand-optimized Rust: Statistics 2M
// Key optimizations vs Tyrus:
// 1. Vec::with_capacity (pre-allocate)
// 2. for loop with usize (vs while with f64)
// 3. Single-pass stats where possible
// 4. Direct iteration (no .iter().cloned())

fn statistics(size: usize) -> String {
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        let fi = i as f64;
        data.push((fi * 0.001).sin() * 100.0 + (fi * 0.002).cos() * 50.0);
    }

    // Pass 1: sum
    let sum: f64 = data.iter().sum();
    let mean = sum / size as f64;

    // Pass 2: variance + min/max + threshold count
    let mut variance_sum = 0.0_f64;
    let mut above_threshold = 0_usize;
    for &v in &data {
        let diff = v - mean;
        variance_sum += diff * diff;
        if v > 150.0 { above_threshold += 1; }
    }
    let std_dev = (variance_sum / size as f64).sqrt();

    // Pass 3: within 1 std dev
    let within_1sd = data.iter()
        .filter(|&&v| v > mean - std_dev && v < mean + std_dev)
        .count();

    let pct_within = (within_1sd * 100) / size;
    format!("{},{},{}", mean.floor() as i64, std_dev.floor() as i64, pct_within)
}

fn main() {
    println!("{}", statistics(2_000_000));
}
