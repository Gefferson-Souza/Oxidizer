#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn accumulate(mut iterations: f64) -> f64 {
    let mut results = vec![];
    let mut i = 0f64;
    while i < iterations {
        let value = ((i * 1f64 as f64).powf(1.5f64 as f64)).floor()
            + ((i * 1f64 as f64).sin() * 100f64).abs();
        results.push(value);
        i = i + 1f64;
    }
    let mut sum = 0f64;
    results
        .iter()
        .cloned()
        .for_each(|v| {
            sum = sum + v;
        });
    return (sum / iterations).floor();
}
fn main() -> () {
    println!("{}", accumulate(500000f64));
}

