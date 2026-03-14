#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn data_pipeline(mut size: f64) -> f64 {
    let mut data = vec![];
    let mut i = 0f64;
    while i < size {
        data.push(i * 1f64);
        i = i + 1f64;
    }
    let result = data
        .clone()
        .into_iter()
        .filter(|n| {
            let n = n.clone();
            n % 3f64 != 0f64
        })
        .collect::<Vec<_>>()
        .clone()
        .into_iter()
        .map(|n| n * n + (n as f64).sqrt())
        .collect::<Vec<_>>()
        .iter()
        .cloned()
        .fold(0f64, |acc, n| acc + n);
    return (result).floor();
}
fn main() -> () {
    println!("{}", data_pipeline(100000f64));
}

