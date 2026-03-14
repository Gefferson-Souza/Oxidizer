use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_top_level_const() {
    assert_output_equivalent(
        r#"
const greeting: string = "Hello World";
const count: number = 42;
const active: boolean = true;
console.log(greeting);
console.log(count);
console.log(active);
"#,
    );
}

#[test]
fn test_equivalence_top_level_let() {
    assert_output_equivalent(
        r#"
let counter: number = 0;
counter = counter + 1;
counter = counter + 1;
console.log(counter);
"#,
    );
}

#[test]
fn test_equivalence_top_level_mixed() {
    assert_output_equivalent(
        r#"
function greet(name: string): string {
    return "Hello, " + name;
}
const message: string = greet("World");
console.log(message);
"#,
    );
}
