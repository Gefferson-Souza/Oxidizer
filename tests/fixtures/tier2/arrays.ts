function doubleAll(nums: number[]): number[] {
    return nums.map((n: number) => n * 2);
}

function evens(nums: number[]): number[] {
    return nums.filter((n: number) => n % 2 === 0);
}

function sum(nums: number[]): number {
    let total: number = 0;
    nums.forEach((n: number) => {
        total = total + n;
    });
    return total;
}
