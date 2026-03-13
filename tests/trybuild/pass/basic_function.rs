// Trybuild pass test: verifies basic transpiled function compiles
fn add(a: f64, b: f64) -> f64 {
    return a + b;
}

fn main() {
    let result = add(2.0, 3.0);
    println!("{}", result);
}
