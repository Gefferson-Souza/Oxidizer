#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn text_processing(mut iterations: f64) -> f64 {
    let mut count = 0f64;
    let mut i = 0f64;
    while i < iterations {
        let name = format!("{}{}", String::from("user_"), i.to_string());
        let upper = name.to_uppercase();
        let has_prefix = upper.starts_with(&String::from("USER_1") as &str);
        if has_prefix {
            count = count + 1f64;
        }
        let trimmed = (format!(
            "{}{}", format!("{}{}", String::from("  "), name), String::from("  ")
        ))
            .trim()
            .to_string();
        let replaced = trimmed
            .replacen(&String::from("user"), &String::from("account"), 1);
        if replaced.contains(&String::from("account") as &str) {
            count = count + 1f64;
        }
        i = i + 1f64;
    }
    return count;
}
fn main() -> () {
    println!("{}", text_processing(50000f64));
}

