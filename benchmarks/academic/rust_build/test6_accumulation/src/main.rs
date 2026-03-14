#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
fn accumulate(mut readings: f64) -> String {
    let mut temperatures = vec![];
    let mut humidities = vec![];
    let mut i = 0f64;
    while i < readings {
        let temp = 25f64 + (i * 0.0001f64 as f64).sin() * 10f64
            + (i * 0.01f64 as f64).cos() * 5f64 + (i * 3.7f64 as f64).sin() * 2f64;
        temperatures.push(temp);
        let hum = 60f64 - (temp - 25f64) * 2f64 + (i * 0.007f64 as f64).sin() * 15f64;
        humidities.push(hum);
        i = i + 1f64;
    }
    let mut temp_sum = 0f64;
    temperatures
        .iter()
        .cloned()
        .for_each(|t| {
            temp_sum = temp_sum + t;
        });
    let temp_mean = temp_sum / readings;
    let mut temp_var_sum = 0f64;
    temperatures
        .iter()
        .cloned()
        .for_each(|t| {
            let diff = t - temp_mean;
            temp_var_sum = temp_var_sum + diff * diff;
        });
    let temp_std = (temp_var_sum / readings as f64).sqrt();
    let mut anomalies = 0f64;
    temperatures
        .iter()
        .cloned()
        .for_each(|t| {
            if (t - temp_mean).abs() > 2f64 * temp_std {
                anomalies = anomalies + 1f64;
            }
        });
    let mut hum_sum = 0f64;
    humidities
        .iter()
        .cloned()
        .for_each(|h| {
            hum_sum = hum_sum + h;
        });
    let hum_mean = hum_sum / readings;
    return format!(
        "{}{}", format!("{}{}", format!("{}{}", format!("{}{}", (temp_mean).floor()
        .to_string(), String::from(",")), (hum_mean).floor().to_string()),
        String::from(",")), anomalies.to_string()
    );
}
fn main() -> () {
    println!("{}", accumulate(1000000f64));
}

