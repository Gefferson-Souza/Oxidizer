// Test 5: Matrix-like Computation — O(n²) with real math
// Real-world analog: Distance matrix for recommendation engine,
//   similarity scoring between products/users, image kernel convolution
// Data: Simulated coordinate pairs (like GPS positions or feature vectors)

function matrixCompute(n: number): number {
    // Compute pairwise "distance" between n points
    // Each point has coords derived from trigonometric functions
    let total_distance: number = 0;
    let comparisons: number = 0;
    let i: number = 0;

    while (i < n) {
        // Point i coordinates
        const xi: number = Math.sin(i * 0.1) * 100;
        const yi: number = Math.cos(i * 0.15) * 100;

        let j: number = i + 1;
        while (j < n) {
            // Point j coordinates
            const xj: number = Math.sin(j * 0.1) * 100;
            const yj: number = Math.cos(j * 0.15) * 100;

            // Euclidean distance
            const dx: number = xi - xj;
            const dy: number = yi - yj;
            const dist: number = Math.sqrt(dx * dx + dy * dy);

            total_distance = total_distance + dist;
            comparisons = comparisons + 1;
            j = j + 1;
        }
        i = i + 1;
    }

    return Math.floor(total_distance / comparisons);
}

function main(): void {
    console.log(matrixCompute(4000));
}
main();
