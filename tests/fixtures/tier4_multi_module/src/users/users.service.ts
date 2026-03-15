import { Injectable } from '@nestjs/common';

@Injectable()
export class UsersService {
    private nextId: number;

    constructor() {
        this.nextId = 1;
    }

    findAll(): string {
        return "All users";
    }

    create(name: string): string {
        const id: number = this.nextId;
        this.nextId = this.nextId + 1;
        return "Created user " + name + " with id " + id.toString();
    }
}
