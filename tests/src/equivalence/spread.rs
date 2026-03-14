use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_array_spread_two_arrays() {
    assert_output_equivalent(
        r#"
function combine(): string {
    const arr1: string[] = ["a", "b", "c"];
    const arr2: string[] = ["d", "e", "f"];
    const combined: string[] = [...arr1, ...arr2];
    return combined.join(", ");
}
console.log(combine());
"#,
    );
}

#[test]
fn test_equivalence_array_spread_with_elements() {
    assert_output_equivalent(
        r#"
function build(): string {
    const middle: string[] = ["b", "c"];
    const all: string[] = ["a", ...middle, "d"];
    return all.join(", ");
}
console.log(build());
"#,
    );
}

#[test]
fn test_equivalence_array_spread_single() {
    assert_output_equivalent(
        r#"
function copy(): string {
    const original: string[] = ["x", "y", "z"];
    const clone: string[] = [...original];
    return clone.join(", ");
}
console.log(copy());
"#,
    );
}
