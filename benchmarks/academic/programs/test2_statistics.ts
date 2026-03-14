// Test 2: Statistical Analysis — Mean, Variance, Std Dev on 1M data points
// Real-world analog: Analytics dashboards, ML preprocessing, monitoring systems

function statistics(size: number): string {
    let data: number[] = [];
    let i: number = 0;
    while (i < size) {
        data.push(Math.sin(i * 0.001) * 100 + Math.cos(i * 0.002) * 50);
        i = i + 1;
    }

    let sum: number = 0;
    data.forEach((v: number) => {
        sum = sum + v;
    });
    const mean: number = sum / size;

    let variance_sum: number = 0;
    data.forEach((v: number) => {
        const diff: number = v - mean;
        variance_sum = variance_sum + diff * diff;
    });
    const variance: number = variance_sum / size;
    const std_dev: number = Math.sqrt(variance);

    return Math.floor(mean).toString() + "," + Math.floor(std_dev).toString();
}

function main(): void {
    console.log(statistics(1000000));
}
main();
