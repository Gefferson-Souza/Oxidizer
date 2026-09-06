use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_string_to_upper_case() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello";
    console.log(s.toUpperCase());
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_to_lower_case() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "HELLO";
    console.log(s.toLowerCase());
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_includes() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello world";
    console.log(s.includes("world"));
    console.log(s.includes("xyz"));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_replace() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello world";
    console.log(s.replace("world", "rust"));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_trim() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "  hello  ";
    console.log(s.trim());
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_starts_with() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello world";
    console.log(s.startsWith("hello"));
    console.log(s.startsWith("world"));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_ends_with() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello world";
    console.log(s.endsWith("world"));
    console.log(s.endsWith("hello"));
}
run();
"#,
    );
}

// Phase 5: New string method equivalence tests

#[test]
fn test_equivalence_string_substring() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello world";
    console.log(s.substring(0, 5));
    console.log(s.substring(6));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_char_at() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello";
    console.log(s.charAt(0));
    console.log(s.charAt(4));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_index_of() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello world";
    console.log(s.indexOf("world"));
    console.log(s.indexOf("xyz"));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_repeat() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "ab";
    console.log(s.repeat(3));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_slice() {
    assert_output_equivalent(
        r#"
function run(): void {
    const s: string = "hello world";
    console.log(s.slice(0, 5));
    console.log(s.slice(6));
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_pad_start() {
    assert_output_equivalent(
        r#"
function padStartTest(): string {
    const s: string = "hello";
    return s.padStart(10, "*");
}
function run(): void {
    console.log(padStartTest());
}
run();
"#,
    );
}

#[test]
fn test_equivalence_string_pad_end() {
    assert_output_equivalent(
        r#"
function padEndTest(): string {
    const s: string = "hello";
    return s.padEnd(10, "-");
}
function run(): void {
    console.log(padEndTest());
}
run();
"#,
    );
}
