#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn statistics(mut size: f64) -> String {
    let mut data = vec![];
    let mut i = 0f64;
    while i < size {
        data.push(
            (i * 0.001f64 as f64).sin() * 100f64 + (i * 0.002f64 as f64).cos() * 50f64,
        );
        i = i + 1f64;
    }
    let mut sum = 0f64;
    data.iter()
        .cloned()
        .for_each(|v| {
            sum = sum + v;
        });
    let mean = sum / size;
    let mut variance_sum = 0f64;
    data.iter()
        .cloned()
        .for_each(|v| {
            let diff = v - mean;
            variance_sum = variance_sum + diff * diff;
        });
    let variance = variance_sum / size;
    let std_dev = (variance as f64).sqrt();
    return format!(
        "{}{}", format!("{}{}", (mean).floor().to_string(), String::from(",")), (std_dev)
        .floor().to_string()
    );
}
fn main() -> () {
    println!("{}", statistics(1000000f64));
}

