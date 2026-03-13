use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_array_for_each() {
    assert_output_equivalent(
        r#"
function main(): void {
    const nums: number[] = [10, 20, 30];
    nums.forEach((n: number): void => {
        console.log(n);
    });
}
main();
"#,
    );
}

#[test]
fn test_equivalence_array_push() {
    assert_output_equivalent(
        r#"
function main(): void {
    let items: number[] = [1, 2, 3];
    items.push(4);
    items.push(5);
    items.forEach((n: number): void => {
        console.log(n);
    });
}
main();
"#,
    );
}

#[test]
fn test_equivalence_array_includes() {
    assert_output_equivalent(
        r#"
function main(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    console.log(nums.includes(3));
    console.log(nums.includes(9));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_array_join() {
    assert_output_equivalent(
        r#"
function main(): void {
    const words: string[] = ["hello", "world"];
    console.log(words.join(" "));
    console.log(words.join(","));
}
main();
"#,
    );
}
