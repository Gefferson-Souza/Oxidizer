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

#[test]
fn test_nestjs_not_found_exception_maps_to_app_error() {
    let rust = crate::helpers::transpile(
        r#"
import { NotFoundException } from "@nestjs/common";
function findUser(id: string): string {
    throw new NotFoundException("User not found");
}
"#,
    );
    assert!(
        rust.contains("AppError") && rust.contains("NotFound"),
        "Expected AppError::NotFound in: {rust}"
    );
}

#[test]
fn test_nestjs_bad_request_exception_maps_to_app_error() {
    let rust = crate::helpers::transpile(
        r#"
import { BadRequestException } from "@nestjs/common";
function validate(input: string): string {
    throw new BadRequestException("Invalid input");
}
"#,
    );
    assert!(
        rust.contains("AppError") && rust.contains("BadRequest"),
        "Expected AppError::BadRequest in: {rust}"
    );
}

#[test]
fn test_use_guards_generates_middleware() {
    let rust = crate::helpers::transpile(
        r#"
import { Controller, Get, UseGuards, Injectable } from "@nestjs/common";
@Injectable()
class AuthGuard {
    canActivate(): boolean {
        return true;
    }
}
@Controller("/api")
@UseGuards(AuthGuard)
class ApiController {
    @Get("/")
    getData(): string {
        return "protected";
    }
}
"#,
    );
    assert!(
        rust.contains("middleware"),
        "Expected middleware function in: {rust}"
    );
    assert!(
        rust.contains("layer"),
        "Expected .layer() call in router: {rust}"
    );
}

#[test]
fn test_guard_class_generates_middleware_fn() {
    let rust = crate::helpers::transpile(
        r#"
import { Injectable } from "@nestjs/common";
@Injectable()
class AuthGuard {
    canActivate(): boolean {
        return true;
    }
}
"#,
    );
    assert!(
        rust.contains("auth_guard_middleware"),
        "Expected auth_guard_middleware function in: {rust}"
    );
    assert!(
        rust.contains("UNAUTHORIZED"),
        "Expected UNAUTHORIZED status code in: {rust}"
    );
}
