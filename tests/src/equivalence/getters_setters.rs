use crate::helpers::assert_output_equivalent;

/// Basic getter/setter: the transpiler must convert `get prop()` into a
/// method that returns the backing field, and `set prop(value)` into a
/// method that mutates it.  Property-style access (`obj.prop` /
/// `obj.prop = x`) must be rewritten to method calls in the generated Rust.
///
/// Expected output (both TS and Rust):
/// ```text
/// 100
/// 212
/// 0
/// 32
/// ```
#[test]
fn test_equivalence_getter_setter_basic() {
    assert_output_equivalent(
        r#"
class Temperature {
    private _celsius: number;
    constructor(celsius: number) {
        this._celsius = celsius;
    }
    get celsius(): number {
        return this._celsius;
    }
    set celsius(value: number) {
        this._celsius = value;
    }
    get fahrenheit(): number {
        return this._celsius * 1.8 + 32;
    }
}

function main(): void {
    let temp = new Temperature(100);
    console.log(temp.celsius);
    console.log(temp.fahrenheit);
    temp.celsius = 0;
    console.log(temp.celsius);
    console.log(temp.fahrenheit);
}
main();
"#,
    );
}

/// Getter-only (read-only computed property).  No setter is defined, so the
/// transpiler only needs to emit a getter method.  This verifies that the
/// absence of a setter does not cause a compilation error when the property
/// is only read, never assigned.
///
/// Expected output:
/// ```text
/// 78.53750000000001
/// ```
#[test]
fn test_equivalence_getter_only_computed() {
    assert_output_equivalent(
        r#"
class Circle {
    radius: number;
    constructor(radius: number) {
        this.radius = radius;
    }
    get area(): number {
        return 3.14159 * this.radius * this.radius;
    }
}

function main(): void {
    let c = new Circle(5);
    console.log(c.area);
}
main();
"#,
    );
}

/// Setter with validation logic.  The setter body contains an `if` guard
/// that clamps the value, ensuring the transpiler handles non-trivial
/// setter bodies (not just a simple assignment).
///
/// Expected output:
/// ```text
/// 50
/// 100
/// 0
/// ```
#[test]
fn test_equivalence_setter_with_validation() {
    assert_output_equivalent(
        r#"
class Percentage {
    private _value: number;
    constructor(value: number) {
        this._value = value;
    }
    get value(): number {
        return this._value;
    }
    set value(v: number) {
        if (v > 100) {
            this._value = 100;
        } else if (v < 0) {
            this._value = 0;
        } else {
            this._value = v;
        }
    }
}

function main(): void {
    let p = new Percentage(50);
    console.log(p.value);
    p.value = 200;
    console.log(p.value);
    p.value = -10;
    console.log(p.value);
}
main();
"#,
    );
}
