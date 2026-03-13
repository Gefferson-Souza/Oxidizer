function process(input: string): string {
    const upper: string = input.toUpperCase();
    const trimmed: string = upper.trim();
    return trimmed;
}

function contains(haystack: string, needle: string): boolean {
    return haystack.includes(needle);
}

function splitAndJoin(input: string): string {
    const parts: string[] = input.split(",");
    return parts.join(" - ");
}
