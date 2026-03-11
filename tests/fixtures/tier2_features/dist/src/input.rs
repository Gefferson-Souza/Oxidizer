#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: f64,
    pub name: Option<String>,
    pub config: Option<Config>,
    pub tags: Option<Vec<String>>,
}
#[derive(Default, Debug, Clone, PartialEq, serde :: Serialize, serde :: Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub theme: Option<String>,
    pub retries: Option<f64>,
}
fn process_user(user: User) -> String {
    let mut theme = user.config.as_ref().and_then(|__v| __v.theme.clone());
    let mut retries = user
        .config
        .as_ref()
        .and_then(|__v| __v.retries.clone())
        .unwrap_or(3f64);
    let mut calc = 1f64 + 2f64 * 3f64;
    let __destructured = user;
    let mut id = __destructured.id.clone();
    let mut name = __destructured
        .name
        .clone()
        .unwrap_or(String::from("Anonymous"));
    let mut list = vec![String::from("a"), String::from("b"), String::from("c")];
    let __arr_destructured = list;
    let mut first = __arr_destructured[0usize].clone();
    let mut second = __arr_destructured[1usize].clone();
    return format!(
        "User {} ({}): Theme {}, Retries {}, Calc {}, List {}-{}",
        id,
        name,
        theme.unwrap_or(String::from("default")),
        retries,
        calc,
        first,
        second
    );
}
