export function add(a: number, b: number): number {
    return a + b;
}

export class Calculator {
    value: number;

    constructor(initial: number) {
        this.value = initial;
    }

    add(n: number): void {
        this.value = this.value + n;
    }
}

function privateHelper(): void {
    console.log("Internal");
}
