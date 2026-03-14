#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn matrix_compute(mut n: f64) -> f64 {
    let mut total = 0f64;
    let mut i = 0f64;
    while i < n {
        let mut j = 0f64;
        while j < n {
            total = total + ((i * j + 1f64 as f64).sqrt()).floor();
            j = j + 1f64;
        }
        i = i + 1f64;
    }
    return total;
}
fn main() -> () {
    println!("{}", matrix_compute(3000f64));
}

