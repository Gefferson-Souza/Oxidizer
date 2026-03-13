function max(a: number, b: number): number {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

function countdown(n: number): number {
    let result: number = 0;
    let i: number = n;
    while (i > 0) {
        result = result + i;
        i = i - 1;
    }
    return result;
}

function classify(x: number): string {
    if (x > 0) {
        return "positive";
    } else if (x < 0) {
        return "negative";
    } else {
        return "zero";
    }
}
