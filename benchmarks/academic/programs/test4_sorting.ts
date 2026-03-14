// Test 4: Sorting — Sort 100K numbers (leaderboard simulation)
// Real-world analog: Database result ordering, ranking algorithms

function sortBenchmark(size: number): string {
    let data: number[] = [];
    let i: number = 0;
    while (i < size) {
        data.push(Math.floor(Math.sin(i * 1.0) * 1000000));
        i = i + 1;
    }

    data.sort();

    let first: number = 0;
    let last: number = 0;
    data.forEach((v: number) => {
        if (first === 0) {
            first = v;
        }
        last = v;
    });

    return first.toString() + "," + last.toString();
}

function main(): void {
    console.log(sortBenchmark(100000));
}
main();
