// Standard Library Test - Comprehensive coverage

function processData2(input: string): string {
    // String methods
    const len = input.length;
    if (input.includes("ERROR")) {
        console.error("Bad input detected");
        return "";
    }

    // String transformation
    const parts = input.split(",");
    const upper = parts.map(p => p.toUpperCase());
    const trimmed = input.trim();
    const replaced = input.replace("old", "new");

    // Math operations
    const rand = Math.random();
    const maxVal = Math.max(10.0, 20.0);
    const minVal = Math.min(5.0, 15.0);
    const rounded = Math.round(3.7);
    const floored = Math.floor(3.9);
    const ceiled = Math.ceil(3.1);
    const absolute = Math.abs(-42.0);

    // Console output
    console.log("Processing complete");

    // Array methods
    const result = upper.join("-");

    return result;
}

function calculate(a: number, b: number): number {
    const max = Math.max(a, b);
    const min = Math.min(a, b);
    return max - min;
}
