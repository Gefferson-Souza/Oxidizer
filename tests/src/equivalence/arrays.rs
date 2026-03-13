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

// Phase 5: New array method equivalence tests

#[test]
fn test_equivalence_array_index_of() {
    assert_output_equivalent(
        r#"
function main(): void {
    const nums: number[] = [10, 20, 30, 40];
    console.log(nums.indexOf(30));
    console.log(nums.indexOf(99));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_array_reverse() {
    assert_output_equivalent(
        r#"
function main(): void {
    let nums: number[] = [1, 2, 3, 4, 5];
    nums.reverse();
    nums.forEach((n: number): void => {
        console.log(n);
    });
}
main();
"#,
    );
}

#[test]
fn test_equivalence_array_slice() {
    assert_output_equivalent(
        r#"
function main(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    const sliced: number[] = nums.slice(1, 4);
    sliced.forEach((n: number): void => {
        console.log(n);
    });
}
main();
"#,
    );
}

#[test]
fn test_equivalence_array_concat() {
    assert_output_equivalent(
        r#"
function main(): void {
    const a: number[] = [1, 2];
    const b: number[] = [3, 4];
    const c: number[] = a.concat(b);
    c.forEach((n: number): void => {
        console.log(n);
    });
}
main();
"#,
    );
}
