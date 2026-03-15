use super::app_controller::AppController;
use super::app_service::AppService;
#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppModule {}
impl AppModule {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_di() -> Self {
        Self::default()
    }
}
