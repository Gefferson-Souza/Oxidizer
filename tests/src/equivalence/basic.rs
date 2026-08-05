use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_simple_addition() {
    assert_output_equivalent(
        r"
function add(a: number, b: number): number {
    return a + b;
}
function main(): void {
    console.log(add(2, 3));
}
main();
",
    );
}

#[test]
fn test_equivalence_arithmetic_operations() {
    assert_output_equivalent(
        r"
function main(): void {
    console.log(2 + 3);
    console.log(10 - 4);
    console.log(3 * 7);
    console.log(15 / 4);
}
main();
",
    );
}

#[test]
fn test_equivalence_comparison_operators() {
    assert_output_equivalent(
        r"
function main(): void {
    console.log(5 > 3);
    console.log(2 < 1);
    console.log(3 >= 3);
    console.log(4 <= 5);
}
main();
",
    );
}

#[test]
fn test_equivalence_string_concatenation() {
    assert_output_equivalent(
        r#"
function main(): void {
    const greeting: string = "Hello" + " " + "World";
    console.log(greeting);
}
main();
"#,
    );
}

#[test]
fn test_equivalence_if_else() {
    assert_output_equivalent(
        r#"
function classify(x: number): string {
    if (x > 0) {
        return "positive";
    } else if (x < 0) {
        return "negative";
    } else {
        return "zero";
    }
}
function main(): void {
    console.log(classify(5));
    console.log(classify(-3));
    console.log(classify(0));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_while_loop() {
    assert_output_equivalent(
        r"
function sum(n: number): number {
    let total: number = 0;
    let i: number = 1;
    while (i <= n) {
        total = total + i;
        i = i + 1;
    }
    return total;
}
function main(): void {
    console.log(sum(5));
    console.log(sum(10));
}
main();
",
    );
}

#[test]
fn test_equivalence_const_vs_let() {
    assert_output_equivalent(
        r"
function main(): void {
    const x: number = 10;
    let y: number = 20;
    console.log(x);
    console.log(y);
}
main();
",
    );
}

#[test]
fn test_equivalence_unary_negation() {
    assert_output_equivalent(
        r"
function negate(x: number): number {
    return -x;
}
function main(): void {
    console.log(negate(5));
    console.log(negate(-3));
}
main();
",
    );
}

#[test]
fn test_equivalence_logical_not() {
    assert_output_equivalent(
        r"
function invert(x: boolean): boolean {
    return !x;
}
function main(): void {
    console.log(invert(true));
    console.log(invert(false));
}
main();
",
    );
}

#[test]
fn test_equivalence_parenthesized_precedence() {
    assert_output_equivalent(
        r"
function main(): void {
    console.log((2 + 3) * 4);
    console.log(2 + 3 * 4);
    console.log((10 - 2) / 4);
}
main();
",
    );
}

#[test]
fn test_equivalence_exponentiation() {
    assert_output_equivalent(
        r"
function main(): void {
    console.log(2 ** 3);
    console.log(3 ** 2);
    console.log(2 ** 10);
}
main();
",
    );
}

#[test]
fn test_equivalence_boolean_logic() {
    assert_output_equivalent(
        r"
function main(): void {
    const a: boolean = true;
    const b: boolean = false;
    console.log(a && b);
    console.log(a || b);
    console.log(!a);
    console.log(!b);
    console.log(a && !b);
}
main();
",
    );
}
