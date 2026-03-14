// Test 5: Nested Computation — O(n^2) matrix-like operation
// Real-world analog: Report cross-references, distance calculations, scoring

function matrixCompute(n: number): number {
    let total: number = 0;
    let i: number = 0;
    while (i < n) {
        let j: number = 0;
        while (j < n) {
            total = total + Math.floor(Math.sqrt(i * j + 1));
            j = j + 1;
        }
        i = i + 1;
    }
    return total;
}

function main(): void {
    console.log(matrixCompute(3000));
}
main();
