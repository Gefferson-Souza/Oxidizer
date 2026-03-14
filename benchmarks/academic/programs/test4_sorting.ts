// Test 4: Sorting — Large dataset ordering
// Real-world analog: Leaderboard computation, price ranking
// Data: Simulated transaction amounts

function sortBenchmark(size: number): string {
    let data: number[] = [];
    let i: number = 0;
    while (i < size) {
        const base: number = Math.abs(Math.sin(i * 0.317)) * 100;
        const spike: number = Math.abs(Math.cos(i * 0.0013)) * Math.abs(Math.sin(i * 0.0007)) * 10000;
        data.push(Math.floor(base + spike));
        i = i + 1;
    }

    data.sort();

    // Count values in ranges (histogram) — avoids index access
    let below_100: number = 0;
    let mid_range: number = 0;
    let above_5000: number = 0;
    data.forEach((v: number) => {
        if (v < 100) { below_100 = below_100 + 1; }
        if (v >= 100) { if (v < 5000) { mid_range = mid_range + 1; } }
        if (v >= 5000) { above_5000 = above_5000 + 1; }
    });

    return below_100.toString() + "," + mid_range.toString() + "," + above_5000.toString();
}

function main(): void {
    console.log(sortBenchmark(500000));
}
main();
