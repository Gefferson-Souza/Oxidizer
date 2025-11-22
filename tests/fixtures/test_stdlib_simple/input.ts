// Simple stdlib test that can be executed

function testMath(): number {
    const max = Math.max(10.0, 20.0);
    const min = Math.min(5.0, 15.0);
    const rounded = Math.round(3.7);
    return max + min + rounded;
}

function testString(input: string): string {
    const upper = input.toUpperCase();
    const hasError = input.includes("test");
    return upper;
}
