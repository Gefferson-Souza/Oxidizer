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

#[test]
fn test_equivalence_object_spread_override() {
    assert_output_equivalent(
        r#"
interface Config {
    host: string;
    port: number;
    debug: boolean;
}
function createProd(base: Config): Config {
    const prod: Config = { ...base, debug: false };
    return prod;
}
function main(): void {
    const dev: Config = { host: "localhost", port: 3100, debug: true };
    const prod: Config = createProd(dev);
    console.log(prod.host);
    console.log(prod.port);
    console.log(prod.debug);
}
main();
"#,
    );
}

#[test]
fn test_equivalence_object_spread_copy() {
    assert_output_equivalent(
        r#"
interface Point {
    x: number;
    y: number;
}
function main(): void {
    const p: Point = { x: 10, y: 20 };
    const copy: Point = { ...p };
    console.log(copy.x);
    console.log(copy.y);
}
main();
"#,
    );
}
