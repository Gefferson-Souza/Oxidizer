use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_array_for_each() {
    assert_output_equivalent(
        r"
function run(): void {
    const nums: number[] = [10, 20, 30];
    nums.forEach((n: number): void => {
        console.log(n);
    });
}
run();
",
    );
}

#[test]
fn test_equivalence_array_push() {
    assert_output_equivalent(
        r"
function run(): void {
    let items: number[] = [1, 2, 3];
    items.push(4);
    items.push(5);
    items.forEach((n: number): void => {
        console.log(n);
    });
}
run();
",
    );
}

#[test]
fn test_equivalence_array_includes() {
    assert_output_equivalent(
        r"
function run(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    console.log(nums.includes(3));
    console.log(nums.includes(9));
}
run();
",
    );
}

#[test]
fn test_equivalence_array_join() {
    assert_output_equivalent(
        r#"
function run(): void {
    const words: string[] = ["hello", "world"];
    console.log(words.join(" "));
    console.log(words.join(","));
}
run();
"#,
    );
}

// Phase 5: New array method equivalence tests

#[test]
fn test_equivalence_array_index_of() {
    assert_output_equivalent(
        r"
function run(): void {
    const nums: number[] = [10, 20, 30, 40];
    console.log(nums.indexOf(30));
    console.log(nums.indexOf(99));
}
run();
",
    );
}

#[test]
fn test_equivalence_array_reverse() {
    assert_output_equivalent(
        r"
function run(): void {
    let nums: number[] = [1, 2, 3, 4, 5];
    nums.reverse();
    nums.forEach((n: number): void => {
        console.log(n);
    });
}
run();
",
    );
}

#[test]
fn test_equivalence_array_slice() {
    assert_output_equivalent(
        r"
function run(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    const sliced: number[] = nums.slice(1, 4);
    sliced.forEach((n: number): void => {
        console.log(n);
    });
}
run();
",
    );
}

#[test]
fn test_equivalence_array_reduce_with_initial() {
    assert_output_equivalent(
        r"
function run(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    const sum: number = nums.reduce((acc: number, n: number): number => acc + n, 0);
    console.log(sum);
}
run();
",
    );
}

#[test]
fn test_equivalence_array_concat() {
    assert_output_equivalent(
        r"
function run(): void {
    const a: number[] = [1, 2];
    const b: number[] = [3, 4];
    const c: number[] = a.concat(b);
    c.forEach((n: number): void => {
        console.log(n);
    });
}
run();
",
    );
}

#[test]
fn test_equivalence_array_sort() {
    assert_output_equivalent(
        r"
function sortTest(): void {
    let nums: number[] = [3, 1, 4, 1, 5, 9, 2, 6];
    nums.sort();
    nums.forEach((n: number): void => {
        console.log(n);
    });
}
function run(): void {
    sortTest();
}
run();
",
    );
}

#[test]
fn test_equivalence_array_shift() {
    assert_output_equivalent(
        r"
function shiftTest(): void {
    let items: number[] = [10, 20, 30];
    const first: number = items.shift();
    console.log(first);
    items.forEach((n: number): void => {
        console.log(n);
    });
}
function run(): void {
    shiftTest();
}
run();
",
    );
}
