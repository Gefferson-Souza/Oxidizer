use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_static_method() {
    assert_output_equivalent(
        r"
class MathUtils {
    static add(a: number, b: number): number {
        return a + b;
    }
    static multiply(a: number, b: number): number {
        return a * b;
    }
}

function run(): void {
    console.log(MathUtils.add(3, 4));
    console.log(MathUtils.multiply(5, 6));
}
run();
",
    );
}

#[test]
fn test_equivalence_static_and_instance() {
    assert_output_equivalent(
        r"
class Counter {
    count: number;
    constructor() {
        this.count = 0;
    }
    static create(): Counter {
        return new Counter();
    }
    increment(): number {
        this.count = this.count + 1;
        return this.count;
    }
}

function run(): void {
    let c = Counter.create();
    console.log(c.increment());
    console.log(c.increment());
}
run();
",
    );
}
