use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_math_pow() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.pow(2, 10));
    console.log(Math.pow(3, 3));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_math_sqrt() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.sqrt(16));
    console.log(Math.sqrt(9));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_math_pi() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.PI);
}
main();
"#,
    );
}

#[test]
fn test_equivalence_math_abs() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.abs(-5));
    console.log(Math.abs(3));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_math_floor_ceil_round() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.floor(4.7));
    console.log(Math.ceil(4.2));
    console.log(Math.round(4.5));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_math_max_min() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.max(3, 7));
    console.log(Math.min(3, 7));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_math_trunc() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.trunc(4.9));
    console.log(Math.trunc(-4.9));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_math_sign() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.sign(5));
    console.log(Math.sign(-3));
    console.log(Math.sign(0));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_math_log() {
    assert_output_equivalent(
        r#"
function main(): void {
    console.log(Math.log(1));
}
main();
"#,
    );
}
