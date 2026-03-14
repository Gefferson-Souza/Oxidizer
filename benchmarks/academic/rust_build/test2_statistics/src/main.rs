#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn statistics(mut size: f64) -> String {
    let mut data = vec![];
    let mut i = 1f64;
    while i <= size {
        let base = 50f64;
        let spike = ((i * 0.0073f64 as f64).sin() * (i * 0.0031f64 as f64).cos()).abs()
            * 200f64;
        let jitter = (i * 3.17f64 as f64).sin() * 10f64;
        data.push(base + spike + jitter);
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
    let mut min_val = data[0usize].clone();
    let mut max_val = data[0usize].clone();
    let mut above_threshold = 0f64;
    data.iter()
        .cloned()
        .for_each(|v| {
            let diff = v - mean;
            variance_sum = variance_sum + diff * diff;
            if v < min_val {
                min_val = v;
            }
            if v > max_val {
                max_val = v;
            }
            if v > 150f64 {
                above_threshold = above_threshold + 1f64;
            }
        });
    let std_dev = (variance_sum / size as f64).sqrt();
    let mut within_1sd = 0f64;
    data.iter()
        .cloned()
        .for_each(|v| {
            if v > mean - std_dev {
                if v < mean + std_dev {
                    within_1sd = within_1sd + 1f64;
                }
            }
        });
    let pct_within = (within_1sd * 100f64 / size).floor();
    return format!(
        "{}{}", format!("{}{}", format!("{}{}", format!("{}{}", (mean).floor()
        .to_string(), String::from(",")), (std_dev).floor().to_string()),
        String::from(",")), pct_within.to_string()
    );
}
fn main() -> () {
    println!("{}", statistics(2000000f64));
}

