// Executable Test - Class with methods

class Calculator {
    value: number;

    constructor(initial: number) {
        this.value = initial;
    }

    add(x: number): number {
        return this.value + x;
    }

    multiply(x: number): number {
        return this.value * x;
    }

    getValue(): number {
        return this.value;
    }
}
