// Test 7: Scaling Analysis — Same algorithm at 10K, 100K, 1M
// Purpose: Show how performance gap changes with data size
// Tests when Node.js starts to bottleneck vs Rust staying efficient

function compute(size: number): number {
    let data: number[] = [];
    let i: number = 0;
    while (i < size) {
        data.push(Math.sin(i * 0.001) * 100 + Math.cos(i * 0.003) * 50);
        i = i + 1;
    }

    // Multi-pass processing (common in real analytics)
    let positives: number[] = data.filter((v: number) => v > 0);
    let transformed: number[] = positives.map((v: number) => Math.sqrt(v) * Math.log(v + 1));
    let total: number = transformed.reduce((acc: number, v: number) => acc + v, 0);

    // Second pass: count outliers
    let outliers: number = 0;
    let mean: number = total / transformed.length;
    transformed.forEach((v: number) => {
        if (Math.abs(v - mean) > mean * 0.5) {
            outliers = outliers + 1;
        }
    });

    return Math.floor(total) + outliers;
}

function main(): void {
    console.log(compute(1000000));
}
main();
