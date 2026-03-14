// Test 1: Data Pipeline — ETL-style filter/map/reduce on 100K records
// Real-world analog: API response processing, database result transformation

function dataPipeline(size: number): number {
    let data: number[] = [];
    let i: number = 0;
    while (i < size) {
        data.push(i * 1.0);
        i = i + 1;
    }

    const result: number = data
        .filter((n: number) => n % 3 !== 0)
        .map((n: number) => n * n + Math.sqrt(n))
        .reduce((acc: number, n: number) => acc + n, 0);

    return Math.floor(result);
}

function main(): void {
    console.log(dataPipeline(100000));
}
main();
