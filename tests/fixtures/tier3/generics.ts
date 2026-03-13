interface Container<T> {
    value: T;
    label: string;
}

class Wrapper<T> {
    private data: T;

    constructor(data: T) {
        this.data = data;
    }

    get(): T {
        return this.data;
    }
}

function identity<T>(value: T): T {
    return value;
}
