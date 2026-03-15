import { Injectable } from '@nestjs/common';

@Injectable()
export class UsersService {
    private nextId: number;

    constructor() {
        this.nextId = 1;
    }

    findAll(): string {
        return "[]";
    }

    create(name: string, email: string): string {
        const id: number = this.nextId;
        this.nextId = this.nextId + 1;
        return "{\"id\":" + id.toString() + ",\"name\":\"" + name + "\",\"email\":\"" + email + "\"}";
    }
}
