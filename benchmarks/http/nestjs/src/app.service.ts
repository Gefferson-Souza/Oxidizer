import { Injectable } from '@nestjs/common';

@Injectable()
export class AppService {
    private nextId: number;

    constructor() {
        this.nextId = 1;
    }

    getHealth(): string {
        return "ok";
    }

    add(a: number, b: number): string {
        const result: number = a + b;
        return result.toString();
    }

    subtract(a: number, b: number): string {
        const result: number = a - b;
        return result.toString();
    }

    multiply(a: number, b: number): string {
        const result: number = a * b;
        return result.toString();
    }

    divide(a: number, b: number): string {
        if (b === 0) {
            return "error: division by zero";
        }
        const result: number = a / b;
        return result.toString();
    }

    power(base: number, exp: number): string {
        const result: number = Math.pow(base, exp);
        return result.toString();
    }

    squareRoot(n: number): string {
        const result: number = Math.sqrt(n);
        return result.toString();
    }

    toUpperCase(text: string): string {
        return text.toUpperCase();
    }

    toLowerCase(text: string): string {
        return text.toLowerCase();
    }

    repeatText(text: string, times: number): string {
        return text.repeat(times);
    }

    trimText(text: string): string {
        return text.trim();
    }

    createUser(name: string, email: string): string {
        const id: number = this.nextId;
        this.nextId = this.nextId + 1;
        return "User #" + id.toString() + ": " + name + " (" + email + ")";
    }

    greet(name: string): string {
        return "Hello, " + name + "! Welcome to Tyrus.";
    }

    getLength(text: string): string {
        const len: number = text.length;
        return len.toString();
    }
}
