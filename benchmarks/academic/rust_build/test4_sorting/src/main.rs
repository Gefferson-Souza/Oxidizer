#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn sort_benchmark(mut size: f64) -> String {
    let mut data = vec![];
    let mut i = 0f64;
    while i < size {
        let base = ((i * 0.317f64 as f64).sin()).abs() * 100f64;
        let spike = ((i * 0.0013f64 as f64).cos()).abs()
            * ((i * 0.0007f64 as f64).sin()).abs() * 10000f64;
        data.push((base + spike).floor());
        i = i + 1f64;
    }
    {
        data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    };
    let mut below_100 = 0f64;
    let mut mid_range = 0f64;
    let mut above_5000 = 0f64;
    data.iter()
        .cloned()
        .for_each(|v| {
            if v < 100f64 {
                below_100 = below_100 + 1f64;
            }
            if v >= 100f64 {
                if v < 5000f64 {
                    mid_range = mid_range + 1f64;
                }
            }
            if v >= 5000f64 {
                above_5000 = above_5000 + 1f64;
            }
        });
    return format!(
        "{}{}", format!("{}{}", format!("{}{}", format!("{}{}", below_100.to_string(),
        String::from(",")), mid_range.to_string()), String::from(",")), above_5000
        .to_string()
    );
}
fn main() -> () {
    println!("{}", sort_benchmark(500000f64));
}

