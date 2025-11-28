#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
pub struct CreatePaymentDto {
    pub amount: f64,
    pub currency: String,
    pub targetAccount: String,
}
impl CreatePaymentDto {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_di() -> Self {
        Self::default()
    }
}
