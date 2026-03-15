#[derive(Default, Debug, Clone)]
pub struct AppService {
    pub next_id: std::sync::Arc<std::sync::Mutex<f64>>,
}
impl AppService {
    pub fn new() -> Self {
        Self {
            next_id: std::sync::Arc::new(std::sync::Mutex::new(1f64)),
        }
    }
    pub fn new_di() -> Self {
        Self {
            next_id: std::sync::Arc::new(std::sync::Mutex::new(Default::default())),
        }
    }
    pub fn get_health(&self) -> String {
        return String::from("ok");
    }
    pub fn add(&self, a: f64, b: f64) -> String {
        let result = a + b;
        return result.to_string();
    }
    pub fn subtract(&self, a: f64, b: f64) -> String {
        let result = a - b;
        return result.to_string();
    }
    pub fn multiply(&self, a: f64, b: f64) -> String {
        let result = a * b;
        return result.to_string();
    }
    pub fn divide(&self, a: f64, b: f64) -> String {
        if b == 0f64 {
            return String::from("error: division by zero");
        }
        let result = a / b;
        return result.to_string();
    }
    pub fn power(&self, base: f64, exp: f64) -> String {
        let result = (base as f64).powf(exp as f64);
        return result.to_string();
    }
    pub fn square_root(&self, n: f64) -> String {
        let result = (n as f64).sqrt();
        return result.to_string();
    }
    pub fn to_upper_case(&self, text: String) -> String {
        return text.to_uppercase();
    }
    pub fn to_lower_case(&self, text: String) -> String {
        return text.to_lowercase();
    }
    pub fn repeat_text(&self, text: String, times: f64) -> String {
        return text.repeat(times as usize);
    }
    pub fn trim_text(&self, text: String) -> String {
        return text.trim().to_string();
    }
    pub fn create_user(&self, name: String, email: String) -> String {
        let id = *self.next_id.lock().unwrap_or_else(|e| e.into_inner());
        {
            let __new_val = *self.next_id.lock().unwrap_or_else(|e| e.into_inner())
                + 1f64;
            *self.next_id.lock().unwrap_or_else(|e| e.into_inner()) = __new_val;
        };
        return format!(
            "{}{}{}{}{}{}{}", String::from("User #"), id.to_string(), String::from(": "),
            name, String::from(" ("), email, String::from(")")
        );
    }
    pub fn greet(&self, name: String) -> String {
        return format!(
            "{}{}{}", String::from("Hello, "), name, String::from("! Welcome to Tyrus.")
        );
    }
    pub fn get_length(&self, text: String) -> String {
        let len = text.len() as f64;
        return len.to_string();
    }
}
