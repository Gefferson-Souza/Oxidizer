function findFirst(nums: number[]): number {
    const found: number = nums.find((n: number) => n > 10) ?? 0;
    return found;
}

function hasLarge(nums: number[]): boolean {
    return nums.some((n: number) => n > 100);
}

function allPositive(nums: number[]): boolean {
    return nums.every((n: number) => n > 0);
}
