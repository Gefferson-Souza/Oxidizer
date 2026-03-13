function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
}

function roundToTwo(n: number): number {
    return Math.round(n * 100) / 100;
}
