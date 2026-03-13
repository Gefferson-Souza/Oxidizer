import { Controller, Get, Post, Body, Param } from "@nestjs/common";
import { Injectable } from "@nestjs/common";

interface CreateUserDto {
    name: string;
    email: string;
}

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

    create(dto: CreateUserDto): User {
        const user: User = {
            id: this.users.length + 1,
            name: dto.name,
            email: dto.email,
        };
        this.users.push(user);
        return user;
    }
}

@Controller("/users")
class UsersController {
    constructor(private usersService: UsersService) {}

    @Get("/")
    findAll(): User[] {
        return this.usersService.findAll();
    }

    @Post("/")
    create(@Body() dto: CreateUserDto): User {
        return this.usersService.create(dto);
    }
}
