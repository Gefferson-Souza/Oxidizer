use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_simple_addition() {
    assert_output_equivalent(
        r#"
function add(a: number, b: number): number {
    return a + b;
}
function main(): void {
    console.log(add(2, 3));
}
main();
"#,
    );
}
