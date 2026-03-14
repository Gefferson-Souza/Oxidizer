// Test 1: Data Pipeline — ETL-style transformation at multiple scales
// Real-world analog: API backend processing user records from database
// Simulates: filter invalid entries → transform fields → aggregate results
// Data: Synthetic user activity scores (sine wave + noise = realistic distribution)

function dataPipeline(size: number): number {
    let data: number[] = [];
    let i: number = 0;
    while (i < size) {
        // Simulate user activity scores: base seasonal pattern + daily variation
        const seasonal: number = Math.sin(i * 0.0001) * 50 + 50;
        const daily: number = Math.cos(i * 0.01) * 20;
        const noise: number = Math.sin(i * 7.31 + 0.5) * 10;
        data.push(seasonal + daily + noise);
        i = i + 1;
    }

    // Pipeline: filter active users → compute engagement score → sum
    const result: number = data
        .filter((score: number) => score > 30)
        .map((score: number) => score * score + Math.sqrt(score) * 10)
        .reduce((acc: number, val: number) => acc + val, 0);

    return Math.floor(result);
}

function main(): void {
    console.log(dataPipeline(500000));
}
main();
