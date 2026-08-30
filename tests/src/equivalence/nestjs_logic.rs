use crate::helpers::assert_output_equivalent;

/// Verifies that the service logic from the reference `NestJS` project
/// produces identical output in both TypeScript and generated Rust.
/// This is the REAL equivalence test — not just "does it compile".

#[test]
fn test_equivalence_users_service_find_all() {
    assert_output_equivalent(
        r#"
class UsersService {
    findAll(): string {
        return "[]";
    }
}

function run(): void {
    let svc = new UsersService();
    console.log(svc.findAll());
}
run();
"#,
    );
}

#[test]
fn test_equivalence_users_service_create() {
    assert_output_equivalent(
        r#"
class UsersService {
    private nextId: number;

    constructor() {
        this.nextId = 1;
    }

    create(name: string, email: string): string {
        const id: number = this.nextId;
        this.nextId = this.nextId + 1;
        return "{\"id\":" + id.toString() + ",\"name\":\"" + name + "\",\"email\":\"" + email + "\"}";
    }
}

function run(): void {
    let svc = new UsersService();
    console.log(svc.create("Alice", "alice@test.com"));
    console.log(svc.create("Bob", "bob@test.com"));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_app_service_health() {
    assert_output_equivalent(
        r#"
class AppService {
    getHealth(): string {
        return "{\"status\":\"ok\"}";
    }
}

function run(): void {
    let svc = new AppService();
    console.log(svc.getHealth());
}
run();
"#,
    );
}

#[test]
fn test_equivalence_class_composition_di() {
    assert_output_equivalent(
        r#"
class UserStore {
    findAll(): string {
        return "[]";
    }
}

class UserHandler {
    private store: UserStore;

    constructor(store: UserStore) {
        this.store = store;
    }

    getAll(): string {
        return this.store.findAll();
    }
}

function run(): void {
    let store = new UserStore();
    let handler = new UserHandler(store);
    console.log(handler.getAll());
}
run();
"#,
    );
}

#[test]
fn test_equivalence_multi_service_create_sequence() {
    assert_output_equivalent(
        r#"
class UsersService {
    private nextId: number;

    constructor() {
        this.nextId = 1;
    }

    create(name: string): string {
        const id: number = this.nextId;
        this.nextId = this.nextId + 1;
        return "User " + name + " created with id " + id.toString();
    }

    findAll(): string {
        return "[]";
    }
}

class AppService {
    getHealth(): string {
        return "ok";
    }
}

function run(): void {
    let usersSvc = new UsersService();
    let appSvc = new AppService();
    console.log(appSvc.getHealth());
    console.log(usersSvc.create("Alice"));
    console.log(usersSvc.create("Bob"));
    console.log(usersSvc.findAll());
}
run();
"#,
    );
}
