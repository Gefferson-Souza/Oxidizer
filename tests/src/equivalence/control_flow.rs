use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_for_loop() {
    assert_output_equivalent(
        r"
function run(): void {
    let sum: number = 0;
    for (let i: number = 1; i <= 5; i++) {
        sum = sum + i;
    }
    console.log(sum);
}
run();
",
    );
}

#[test]
fn test_equivalence_for_of_loop() {
    assert_output_equivalent(
        r"
function run(): void {
    const nums: number[] = [10, 20, 30];
    for (const n of nums) {
        console.log(n);
    }
}
run();
",
    );
}

#[test]
fn test_equivalence_do_while_loop() {
    assert_output_equivalent(
        r"
function run(): void {
    let i: number = 0;
    do {
        i = i + 1;
    } while (i < 5);
    console.log(i);
}
run();
",
    );
}

#[test]
fn test_equivalence_switch_basic() {
    assert_output_equivalent(
        r#"
function describe(x: number): string {
    switch (x) {
        case 1:
            return "one";
        case 2:
            return "two";
        case 3:
            return "three";
        default:
            return "other";
    }
}
function run(): void {
    console.log(describe(1));
    console.log(describe(2));
    console.log(describe(3));
    console.log(describe(99));
}
run();
"#,
    );
}
