#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn matrix_compute(mut n: f64) -> f64 {
    let mut total_distance = 0f64;
    let mut comparisons = 0f64;
    let mut i = 0f64;
    while i < n {
        let xi = (i * 0.1f64 as f64).sin() * 100f64;
        let yi = (i * 0.15f64 as f64).cos() * 100f64;
        let mut j = i + 1f64;
        while j < n {
            let xj = (j * 0.1f64 as f64).sin() * 100f64;
            let yj = (j * 0.15f64 as f64).cos() * 100f64;
            let dx = xi - xj;
            let dy = yi - yj;
            let dist = (dx * dx + dy * dy as f64).sqrt();
            total_distance = total_distance + dist;
            comparisons = comparisons + 1f64;
            j = j + 1f64;
        }
        i = i + 1f64;
    }
    return (total_distance / comparisons).floor();
}
fn main() -> () {
    println!("{}", matrix_compute(4000f64));
}

