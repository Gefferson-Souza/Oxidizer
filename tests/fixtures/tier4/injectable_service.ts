import { Injectable } from "@nestjs/common";

interface User {
    id: number;
    name: string;
    email: string;
}

@Injectable()
class UsersService {
    private users: User[];

    constructor() {
        this.users = [];
    }

    findAll(): User[] {
        return this.users;
    }

    findById(id: number): User {
        return this.users.find((user: User) => user.id === id);
    }

    create(name: string, email: string): User {
        const user: User = {
            id: this.users.length + 1,
            name: name,
            email: email,
        };
        this.users.push(user);
        return user;
    }
}
