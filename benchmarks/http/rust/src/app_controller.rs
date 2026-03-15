use super::app_service::AppService;
#[derive(Default, Debug, Clone)]
pub struct AppController {
    pub app_service: std::sync::Arc<AppService>,
}
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AppController
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<std::sync::Arc<Self>>()
            .cloned()
            .map(|arc| arc.as_ref().clone())
            .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}
impl AppController {
    pub fn new(app_service: std::sync::Arc<AppService>) -> Self {
        Self { app_service: app_service }
    }
    pub fn new_di(app_service: std::sync::Arc<AppService>) -> Self {
        Self { app_service: app_service }
    }
    #[doc = concat!("Route: ", "GET", " ", "/")]
    pub async fn get_health(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(state.app_service.clone().get_health().into());
    }
    #[doc = concat!("Route: ", "GET", " ", "/greet")]
    pub async fn greet(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(state.app_service.clone().greet(String::from("Gefferson")).into());
    }
    #[doc = concat!("Route: ", "GET", " ", "/calc/add")]
    pub async fn add(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(state.app_service.clone().add(3f64, 7f64).into());
    }
    #[doc = concat!("Route: ", "GET", " ", "/calc/subtract")]
    pub async fn subtract(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(state.app_service.clone().subtract(100f64, 37f64).into());
    }
    #[doc = concat!("Route: ", "GET", " ", "/calc/multiply")]
    pub async fn multiply(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(state.app_service.clone().multiply(6f64, 7f64).into());
    }
    #[doc = concat!("Route: ", "GET", " ", "/calc/divide")]
    pub async fn divide(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(state.app_service.clone().divide(355f64, 113f64).into());
    }
    #[doc = concat!("Route: ", "GET", " ", "/calc/power")]
    pub async fn power(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(state.app_service.clone().power(2f64, 10f64).into());
    }
    #[doc = concat!("Route: ", "GET", " ", "/calc/sqrt")]
    pub async fn square_root(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(state.app_service.clone().square_root(144f64).into());
    }
    #[doc = concat!("Route: ", "POST", " ", "/format/uppercase")]
    pub async fn to_upper_case(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(
            state
                .app_service
                .clone()
                .to_upper_case(String::from("hello world from tyrus"))
                .into(),
        );
    }
    #[doc = concat!("Route: ", "POST", " ", "/format/lowercase")]
    pub async fn to_lower_case(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(
            state
                .app_service
                .clone()
                .to_lower_case(String::from("TYRUS COMPILES NESTJS TO RUST"))
                .into(),
        );
    }
    #[doc = concat!("Route: ", "PUT", " ", "/format/trim")]
    pub async fn trim_text(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(
            state
                .app_service
                .clone()
                .trim_text(String::from("   spaces around   "))
                .into(),
        );
    }
    #[doc = concat!("Route: ", "PATCH", " ", "/format/repeat")]
    pub async fn repeat_text(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(
            state.app_service.clone().repeat_text(String::from("ha"), 5f64).into(),
        );
    }
    #[doc = concat!("Route: ", "GET", " ", "/format/length")]
    pub async fn get_length(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(
            state.app_service.clone().get_length(String::from("Tyrus Transpiler")).into(),
        );
    }
    #[doc = concat!("Route: ", "POST", " ", "/users/alice")]
    pub async fn create_alice(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(
            state
                .app_service
                .clone()
                .create_user(String::from("Alice"), String::from("alice@tyrus.dev"))
                .into(),
        );
    }
    #[doc = concat!("Route: ", "POST", " ", "/users/bob")]
    pub async fn create_bob(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(
            state
                .app_service
                .clone()
                .create_user(String::from("Bob"), String::from("bob@tyrus.dev"))
                .into(),
        );
    }
    #[doc = concat!("Route: ", "DELETE", " ", "/reset")]
    pub async fn reset_all(
        axum::extract::State(state): axum::extract::State<std::sync::Arc<Self>>,
    ) -> Result<String, crate::AppError> {
        return Ok(String::from("System reset complete").into());
    }
    pub fn router(state: std::sync::Arc<Self>) -> axum::Router {
        axum::Router::new()
            .route("/", axum::routing::get(Self::get_health))
            .route("/greet", axum::routing::get(Self::greet))
            .route("/calc/add", axum::routing::get(Self::add))
            .route("/calc/subtract", axum::routing::get(Self::subtract))
            .route("/calc/multiply", axum::routing::get(Self::multiply))
            .route("/calc/divide", axum::routing::get(Self::divide))
            .route("/calc/power", axum::routing::get(Self::power))
            .route("/calc/sqrt", axum::routing::get(Self::square_root))
            .route("/format/uppercase", axum::routing::post(Self::to_upper_case))
            .route("/format/lowercase", axum::routing::post(Self::to_lower_case))
            .route("/format/trim", axum::routing::put(Self::trim_text))
            .route("/format/repeat", axum::routing::patch(Self::repeat_text))
            .route("/format/length", axum::routing::get(Self::get_length))
            .route("/users/alice", axum::routing::post(Self::create_alice))
            .route("/users/bob", axum::routing::post(Self::create_bob))
            .route("/reset", axum::routing::delete(Self::reset_all))
            .with_state(state)
    }
}
