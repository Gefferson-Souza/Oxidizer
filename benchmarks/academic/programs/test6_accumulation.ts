// Test 6: Data Accumulation — Building large results
// Real-world analog: Building API responses, aggregating metrics

function accumulate(iterations: number): number {
    let results: number[] = [];
    let i: number = 0;
    while (i < iterations) {
        const value: number = Math.floor(Math.pow(i * 1.0, 1.5)) + Math.abs(Math.sin(i * 1.0) * 100);
        results.push(value);
        i = i + 1;
    }

    let sum: number = 0;
    results.forEach((v: number) => {
        sum = sum + v;
    });

    return Math.floor(sum / iterations);
}

function main(): void {
    console.log(accumulate(500000));
}
main();
