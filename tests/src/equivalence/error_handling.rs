use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_try_catch_basic() {
    assert_output_equivalent(
        r#"
function safeDivide(a: number, b: number): string {
    try {
        if (b === 0) {
            throw new Error("Division by zero");
        }
        const result: number = a / b;
        return result.toString();
    } catch (error) {
        return "Error caught";
    }
}

function main(): void {
    console.log(safeDivide(10, 2));
    console.log(safeDivide(10, 0));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_try_catch_throw() {
    assert_output_equivalent(
        r#"
function processAge(age: number): string {
    try {
        if (age < 0) {
            throw new Error("Age cannot be negative");
        }
        if (age > 150) {
            throw new Error("Age too high");
        }
        return "Valid age: " + age.toString();
    } catch (error) {
        return "Invalid: caught error";
    }
}

function main(): void {
    console.log(processAge(25));
    console.log(processAge(-5));
    console.log(processAge(200));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_nested_try_catch() {
    assert_output_equivalent(
        r#"
function outer(): string {
    try {
        try {
            throw new Error("inner");
        } catch (e) {
            return "caught inner";
        }
    } catch (e) {
        return "caught outer";
    }
    return "no error";
}

function main(): void {
    console.log(outer());
}
main();
"#,
    );
}

#[test]
fn test_equivalence_try_catch_no_throw() {
    assert_output_equivalent(
        r#"
function safe(): string {
    try {
        const x: number = 42;
        return x.toString();
    } catch (error) {
        return "error";
    }
}

function main(): void {
    console.log(safe());
}
main();
"#,
    );
}
