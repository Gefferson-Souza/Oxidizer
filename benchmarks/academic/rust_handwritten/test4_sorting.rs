// Hand-optimized Rust: Sorting 500K
// Key optimizations vs Tyrus:
// 1. Vec::with_capacity
// 2. sort_unstable_by (faster than sort_by — no stability guarantee needed)
// 3. usize loop + direct indexing
// 4. partial_cmp with unwrap_or (vs Tyrus default comparator)

fn sort_benchmark(size: usize) -> String {
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        let fi = i as f64;
        let base = (fi * 0.317).sin().abs() * 100.0;
        let spike = (fi * 0.0013).cos().abs() * (fi * 0.0007).sin().abs() * 10000.0;
        data.push((base + spike).floor());
    }

    data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut below_100 = 0_usize;
    let mut mid_range = 0_usize;
    let mut above_5000 = 0_usize;
    for &v in &data {
        if v < 100.0 { below_100 += 1; }
        else if v < 5000.0 { mid_range += 1; }
        else { above_5000 += 1; }
    }

    format!("{},{},{}", below_100, mid_range, above_5000)
}

fn main() {
    println!("{}", sort_benchmark(500_000));
}
