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

#[test]
fn test_param_decorator_generates_path_extractor() {
    let rust = crate::helpers::transpile(
        r#"
import { Controller, Get, Param } from "@nestjs/common";
@Controller("/users")
class UsersController {
    @Get(":id")
    findOne(@Param("id") id: string): string {
        return id;
    }
}
"#,
    );
    assert!(rust.contains("Path("), "Expected Path extractor in: {rust}");
}

#[test]
fn test_httpcode_decorator_generates_status_code() {
    let rust = crate::helpers::transpile(
        r#"
import { Controller, Post, HttpCode, Body } from "@nestjs/common";
interface Item { name: string; }
@Controller("/items")
class ItemsController {
    @Post("/")
    @HttpCode(201)
    create(@Body() item: Item): Item {
        return item;
    }
}
"#,
    );
    assert!(
        rust.contains("StatusCode"),
        "Expected StatusCode in: {rust}"
    );
    assert!(
        rust.contains("CREATED") || rust.contains("201"),
        "Expected CREATED or 201 in: {rust}"
    );
}

#[test]
fn test_query_decorator_generates_query_extractor() {
    let rust = crate::helpers::transpile(
        r#"
import { Controller, Get, Query } from "@nestjs/common";
@Controller("/items")
class ItemsController {
    @Get("/")
    search(@Query("q") q: string): string {
        return q;
    }
}
"#,
    );
    assert!(
        rust.contains("Query("),
        "Expected Query extractor in: {rust}"
    );
}
