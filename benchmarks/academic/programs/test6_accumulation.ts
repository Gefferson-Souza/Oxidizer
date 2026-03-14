// Test 6: Data Accumulation + Multi-pass Analytics
// Real-world analog: IoT sensor dashboard, weather station aggregation
// Data: Simulated temperature + humidity readings over time

function accumulate(readings: number): string {
    // Phase 1: Collect sensor data
    let temperatures: number[] = [];
    let humidities: number[] = [];
    let i: number = 0;

    while (i < readings) {
        const temp: number = 25 + Math.sin(i * 0.0001) * 10 + Math.cos(i * 0.01) * 5 + Math.sin(i * 3.7) * 2;
        temperatures.push(temp);

        const hum: number = 60 - (temp - 25) * 2 + Math.sin(i * 0.007) * 15;
        humidities.push(hum);

        i = i + 1;
    }

    // Phase 2: Temperature stats
    let temp_sum: number = 0;
    temperatures.forEach((t: number) => { temp_sum = temp_sum + t; });
    const temp_mean: number = temp_sum / readings;

    // Phase 3: Variance + anomaly detection
    let temp_var_sum: number = 0;
    temperatures.forEach((t: number) => {
        const diff: number = t - temp_mean;
        temp_var_sum = temp_var_sum + diff * diff;
    });
    const temp_std: number = Math.sqrt(temp_var_sum / readings);

    let anomalies: number = 0;
    temperatures.forEach((t: number) => {
        if (Math.abs(t - temp_mean) > 2 * temp_std) {
            anomalies = anomalies + 1;
        }
    });

    // Phase 4: Humidity stats
    let hum_sum: number = 0;
    humidities.forEach((h: number) => { hum_sum = hum_sum + h; });
    const hum_mean: number = hum_sum / readings;

    return Math.floor(temp_mean).toString() + "," + Math.floor(hum_mean).toString() + "," + anomalies.toString();
}

function main(): void {
    console.log(accumulate(1000000));
}
main();
