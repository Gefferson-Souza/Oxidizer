#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn data_pipeline(mut size: f64) -> f64 {
    let mut data = vec![];
    let mut i = 0f64;
    while i < size {
        let seasonal = (i * 0.0001f64 as f64).sin() * 50f64 + 50f64;
        let daily = (i * 0.01f64 as f64).cos() * 20f64;
        let noise = (i * 7.31f64 + 0.5f64 as f64).sin() * 10f64;
        data.push(seasonal + daily + noise);
        i = i + 1f64;
    }
    let result = data
        .clone()
        .into_iter()
        .filter(|score| {
            let score = score.clone();
            score > 30f64
        })
        .collect::<Vec<_>>()
        .clone()
        .into_iter()
        .map(|score| score * score + (score as f64).sqrt() * 10f64)
        .collect::<Vec<_>>()
        .iter()
        .cloned()
        .fold(0f64, |acc, val| acc + val);
    return (result).floor();
}
fn main() -> () {
    println!("{}", data_pipeline(500000f64));
}

