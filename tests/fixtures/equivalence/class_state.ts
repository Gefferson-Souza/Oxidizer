// Tests Class internal state mutation using a counter pattern
class Counter {
    count: number;
    constructor(start: number) { this.count = start; }

    add(amount: number): number {
        return this.count + amount;
    }

    report(): string {
        return `Final: ${this.count}`;
    }
}

let total = 10;
const c = new Counter(total);
total = c.add(5);
total = total + 20;
console.log(`Final: ${total}`);
