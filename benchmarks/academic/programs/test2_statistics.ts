// Test 2: Statistical Analysis — Descriptive stats on time-series data
// Real-world analog: Monitoring system computing latency percentiles,
//   analytics dashboard for e-commerce transaction values
// Data: Simulated API response latencies (log-normal-like distribution)

function statistics(size: number): string {
    let data: number[] = [];
    let i: number = 1;
    while (i <= size) {
        // Simulate API latency: base ~50ms + spikes + noise
        // Log-normal-like: exp(sin) produces right-skewed distribution
        const base: number = 50.0;
        const spike: number = Math.abs(Math.sin(i * 0.0073) * Math.cos(i * 0.0031)) * 200;
        const jitter: number = Math.sin(i * 3.17) * 10;
        data.push(base + spike + jitter);
        i = i + 1;
    }

    // Pass 1: Mean
    let sum: number = 0;
    data.forEach((v: number) => { sum = sum + v; });
    const mean: number = sum / size;

    // Pass 2: Variance + min/max
    let variance_sum: number = 0;
    let min_val: number = data[0];
    let max_val: number = data[0];
    let above_threshold: number = 0;
    data.forEach((v: number) => {
        const diff: number = v - mean;
        variance_sum = variance_sum + diff * diff;
        if (v < min_val) { min_val = v; }
        if (v > max_val) { max_val = v; }
        if (v > 150) { above_threshold = above_threshold + 1; }
    });
    const std_dev: number = Math.sqrt(variance_sum / size);

    // Pass 3: Count within 1 std dev (normality check)
    let within_1sd: number = 0;
    data.forEach((v: number) => {
        if (v > mean - std_dev) {
            if (v < mean + std_dev) {
                within_1sd = within_1sd + 1;
            }
        }
    });

    const pct_within: number = Math.floor(within_1sd * 100 / size);
    return Math.floor(mean).toString() + "," + Math.floor(std_dev).toString() + "," + pct_within.toString();
}

function run(): void {
    console.log(statistics(2000000));
}
run();
