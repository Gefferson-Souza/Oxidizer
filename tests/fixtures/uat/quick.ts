// Tyrus UAT fixture — single-file Tier 1 + 2 sample.
// Used by Lane L4 (Single-file Hacker).

interface User {
    id: number;
    name: string;
    email: string;
    active: boolean;
}

function isAdult(age: number): boolean {
    return age >= 18;
}

function classify(score: number): string {
    if (score >= 90) {
        return "excellent";
    } else if (score >= 70) {
        return "good";
    } else if (score >= 50) {
        return "passing";
    } else {
        return "failing";
    }
}

function sumPositive(nums: number[]): number {
    let total: number = 0;
    for (const n of nums) {
        if (n > 0) {
            total = total + n;
        }
    }
    return total;
}

function greetUser(user: User): string {
    return `Hello, ${user.name}! Your id is ${user.id}.`;
}

const alice: User = {
    id: 1,
    name: "Alice",
    email: "alice@example.com",
    active: true,
};

console.log(greetUser(alice));
console.log("isAdult(20):", isAdult(20));
console.log("classify(85):", classify(85));
console.log("sumPositive([1,-2,3,-4,5]):", sumPositive([1, -2, 3, -4, 5]));
