use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_string_to_upper_case() {
    assert_output_equivalent(
        r#"
function main(): void {
    const s: string = "hello";
    console.log(s.toUpperCase());
}
main();
"#,
    );
}

#[test]
fn test_equivalence_string_to_lower_case() {
    assert_output_equivalent(
        r#"
function main(): void {
    const s: string = "HELLO";
    console.log(s.toLowerCase());
}
main();
"#,
    );
}

#[test]
fn test_equivalence_string_includes() {
    assert_output_equivalent(
        r#"
function main(): void {
    const s: string = "hello world";
    console.log(s.includes("world"));
    console.log(s.includes("xyz"));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_string_replace() {
    assert_output_equivalent(
        r#"
function main(): void {
    const s: string = "hello world";
    console.log(s.replace("world", "rust"));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_string_trim() {
    assert_output_equivalent(
        r#"
function main(): void {
    const s: string = "  hello  ";
    console.log(s.trim());
}
main();
"#,
    );
}

#[test]
fn test_equivalence_string_starts_with() {
    assert_output_equivalent(
        r#"
function main(): void {
    const s: string = "hello world";
    console.log(s.startsWith("hello"));
    console.log(s.startsWith("world"));
}
main();
"#,
    );
}

#[test]
fn test_equivalence_string_ends_with() {
    assert_output_equivalent(
        r#"
function main(): void {
    const s: string = "hello world";
    console.log(s.endsWith("world"));
    console.log(s.endsWith("hello"));
}
main();
"#,
    );
}
