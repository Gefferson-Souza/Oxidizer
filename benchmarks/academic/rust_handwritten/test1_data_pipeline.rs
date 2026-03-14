// Hand-optimized Rust: Data Pipeline 500K
// Key optimizations vs Tyrus:
// 1. Vec::with_capacity (avoid reallocations)
// 2. No .clone().into_iter() chains (zero-copy iterator)
// 3. Single-pass filter+map+reduce (no intermediate Vec allocations)
// 4. usize loop counter (integer increment vs f64)

fn data_pipeline(size: usize) -> f64 {
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        let fi = i as f64;
        let seasonal = (fi * 0.0001).sin() * 50.0 + 50.0;
        let daily = (fi * 0.01).cos() * 20.0;
        let noise = (fi * 7.31 + 0.5).sin() * 10.0;
        data.push(seasonal + daily + noise);
    }

    // Single iterator chain — no intermediate Vec allocations
    data.iter()
        .filter(|&&score| score > 30.0)
        .map(|&score| score * score + score.sqrt() * 10.0)
        .sum::<f64>()
        .floor()
}

fn main() {
    println!("{}", data_pipeline(500_000) as i64);
}
