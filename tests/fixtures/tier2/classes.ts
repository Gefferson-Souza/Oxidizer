class Calculator {
    private result: number;

    constructor() {
        this.result = 0;
    }

    add(value: number): number {
        this.result = this.result + value;
        return this.result;
    }

    getResult(): number {
        return this.result;
    }

    reset(): void {
        this.result = 0;
    }
}
