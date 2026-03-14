#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn sort_benchmark(mut size: f64) -> String {
    let mut data = vec![];
    let mut i = 0f64;
    while i < size {
        data.push(((i * 1f64 as f64).sin() * 1000000f64).floor());
        i = i + 1f64;
    }
    {
        data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    };
    let mut first = 0f64;
    let mut last = 0f64;
    data.iter()
        .cloned()
        .for_each(|v| {
            if first == 0f64 {
                first = v;
            }
            last = v;
        });
    return format!(
        "{}{}", format!("{}{}", first.to_string(), String::from(",")), last.to_string()
    );
}
fn main() -> () {
    println!("{}", sort_benchmark(100000f64));
}

