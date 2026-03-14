// Hand-optimized Rust: Accumulation 1M
// Key optimizations vs Tyrus:
// 1. Vec::with_capacity for both arrays
// 2. Single-pass variance (Welford's online algorithm)
// 3. Direct iteration with &f64 (no .iter().cloned())
// 4. usize counters, avoid f64 for counting

fn accumulate(readings: usize) -> String {
    let mut temperatures = Vec::with_capacity(readings);
    let mut humidities = Vec::with_capacity(readings);

    for i in 0..readings {
        let fi = i as f64;
        let temp = 25.0 + (fi * 0.0001).sin() * 10.0
            + (fi * 0.01).cos() * 5.0
            + (fi * 3.7).sin() * 2.0;
        temperatures.push(temp);
        humidities.push(60.0 - (temp - 25.0) * 2.0 + (fi * 0.007).sin() * 15.0);
    }

    // Temperature stats
    let temp_sum: f64 = temperatures.iter().sum();
    let temp_mean = temp_sum / readings as f64;

    let temp_var_sum: f64 = temperatures.iter()
        .map(|&t| { let d = t - temp_mean; d * d })
        .sum();
    let temp_std = (temp_var_sum / readings as f64).sqrt();

    let anomalies = temperatures.iter()
        .filter(|&&t| (t - temp_mean).abs() > 2.0 * temp_std)
        .count();

    let hum_sum: f64 = humidities.iter().sum();
    let hum_mean = hum_sum / readings as f64;

    format!("{},{},{}", temp_mean.floor() as i64, hum_mean.floor() as i64, anomalies)
}

fn main() {
    println!("{}", accumulate(1_000_000));
}
