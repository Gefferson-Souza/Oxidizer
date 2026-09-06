// Test 3: Text Processing — String-intensive operations at scale
// Real-world analog: Log parser counting error patterns,
//   search engine indexing, CSV field extraction
// Data: Simulated server access logs

function textProcessing(entries: number): string {
    // Generate log entries
    let error_count: number = 0;
    let warn_count: number = 0;
    let total_length: number = 0;
    let i: number = 0;

    while (i < entries) {
        // Build log line
        const timestamp: string = "2026-03-14T" + Math.floor(i % 24).toString() + ":" + Math.floor(i % 60).toString();
        const method: string = i % 3 === 0 ? "GET" : i % 3 === 1 ? "POST" : "PUT";
        const path: string = "/api/v1/users/" + (i % 1000).toString();
        const status: string = i % 7 === 0 ? "500" : i % 5 === 0 ? "404" : "200";

        const line: string = timestamp + " " + method + " " + path + " " + status;

        // Analyze
        const upper_line: string = line.toUpperCase();
        total_length = total_length + line.length;

        if (status === "500") {
            error_count = error_count + 1;
        }
        if (status === "404") {
            warn_count = warn_count + 1;
        }

        // String operations: search, replace, check
        if (line.includes("/users/42")) {
            error_count = error_count + 0;
        }
        const cleaned: string = line.trim();
        const replaced: string = cleaned.replace("api", "service");
        if (replaced.startsWith("2026")) {
            total_length = total_length + 1;
        }

        i = i + 1;
    }

    return error_count.toString() + "," + warn_count.toString() + "," + total_length.toString();
}

function run(): void {
    console.log(textProcessing(200000));
}
run();
