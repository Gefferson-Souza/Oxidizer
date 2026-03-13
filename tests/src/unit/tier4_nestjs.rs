use crate::helpers::transpile_fixture;

#[test]
fn test_injectable_uses_arc_mutex() {
    let rust = transpile_fixture("tier4/injectable_service");
    // The transpiler uses fully-qualified paths: std::sync::Arc<std::sync::Mutex<...>>
    let has_arc_mutex = rust.contains("Arc<Mutex<")
        || rust.contains("Arc<std::sync::Mutex<")
        || rust.contains("std::sync::Arc<std::sync::Mutex<");
    assert!(has_arc_mutex, "Expected Arc<Mutex<...>> pattern in: {rust}");
}

#[test]
fn test_injectable_generates_struct() {
    let rust = transpile_fixture("tier4/injectable_service");
    assert!(
        rust.contains("struct UsersService"),
        "Expected 'struct UsersService' in: {rust}"
    );
    assert!(
        rust.contains("impl UsersService"),
        "Expected 'impl UsersService' in: {rust}"
    );
}

#[test]
fn test_controller_generates_router() {
    let rust = transpile_fixture("tier4/controller");
    assert!(
        rust.contains("fn router"),
        "Expected 'fn router' in: {rust}"
    );
}

#[test]
fn test_controller_has_axum_routing() {
    let rust = transpile_fixture("tier4/controller");
    let has_routing =
        rust.contains("axum::routing") || rust.contains("Router") || rust.contains(".route(");
    assert!(has_routing, "Expected axum routing in: {rust}");
}

#[test]
fn test_controller_handler_uses_json() {
    let rust = transpile_fixture("tier4/controller");
    assert!(
        rust.contains("Json"),
        "Expected 'Json' extractor in: {rust}"
    );
}
