use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_console_log_number() {
    assert_output_equivalent(
        r"
function run(): void {
    console.log(42);
}
run();
",
    );
}

#[test]
fn test_equivalence_console_log_string() {
    assert_output_equivalent(
        r#"
function run(): void {
    console.log("hello world");
}
run();
"#,
    );
}

#[test]
fn test_equivalence_console_log_boolean() {
    assert_output_equivalent(
        r"
function run(): void {
    console.log(true);
    console.log(false);
}
run();
",
    );
}
