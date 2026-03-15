use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_subtract_assign() {
    assert_output_equivalent(
        r#"
function main(): void {
    let x: number = 100;
    x -= 37;
    console.log(x);
}
main();
"#,
    );
}

#[test]
fn test_equivalence_multiply_assign() {
    assert_output_equivalent(
        r#"
function main(): void {
    let x: number = 6;
    x *= 7;
    console.log(x);
}
main();
"#,
    );
}

#[test]
fn test_equivalence_divide_assign() {
    assert_output_equivalent(
        r#"
function main(): void {
    let x: number = 100;
    x /= 4;
    console.log(x);
}
main();
"#,
    );
}

#[test]
fn test_equivalence_modulo_assign() {
    assert_output_equivalent(
        r#"
function main(): void {
    let x: number = 17;
    x %= 5;
    console.log(x);
}
main();
"#,
    );
}

#[test]
fn test_equivalence_all_assign_ops() {
    assert_output_equivalent(
        r#"
function main(): void {
    let a: number = 100;
    a += 10;
    a -= 5;
    a *= 2;
    a /= 3;
    console.log(Math.floor(a));
}
main();
"#,
    );
}
