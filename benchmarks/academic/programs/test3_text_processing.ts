// Test 3: Text Processing — String operations at scale
// Real-world analog: Log parsing, text search, template rendering

function textProcessing(iterations: number): number {
    let count: number = 0;
    let i: number = 0;
    while (i < iterations) {
        const name: string = "user_" + i.toString();
        const upper: string = name.toUpperCase();
        const has_prefix: boolean = upper.startsWith("USER_1");
        if (has_prefix) {
            count = count + 1;
        }
        const trimmed: string = ("  " + name + "  ").trim();
        const replaced: string = trimmed.replace("user", "account");
        if (replaced.includes("account")) {
            count = count + 1;
        }
        i = i + 1;
    }
    return count;
}

function main(): void {
    console.log(textProcessing(50000));
}
main();
