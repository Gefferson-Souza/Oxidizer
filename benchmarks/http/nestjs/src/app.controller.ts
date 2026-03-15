import { Controller, Get, Post, Put, Delete, Patch } from '@nestjs/common';
import { AppService } from './app.service';

@Controller('')
export class AppController {
    constructor(private readonly appService: AppService) {}

    @Get('/')
    getHealth(): string {
        return this.appService.getHealth();
    }

    @Get('/greet')
    greet(): string {
        return this.appService.greet("Gefferson");
    }

    @Get('/calc/add')
    add(): string {
        return this.appService.add(3, 7);
    }

    @Get('/calc/subtract')
    subtract(): string {
        return this.appService.subtract(100, 37);
    }

    @Get('/calc/multiply')
    multiply(): string {
        return this.appService.multiply(6, 7);
    }

    @Get('/calc/divide')
    divide(): string {
        return this.appService.divide(355, 113);
    }

    @Get('/calc/power')
    power(): string {
        return this.appService.power(2, 10);
    }

    @Get('/calc/sqrt')
    squareRoot(): string {
        return this.appService.squareRoot(144);
    }

    @Post('/format/uppercase')
    toUpperCase(): string {
        return this.appService.toUpperCase("hello world from tyrus");
    }

    @Post('/format/lowercase')
    toLowerCase(): string {
        return this.appService.toLowerCase("TYRUS COMPILES NESTJS TO RUST");
    }

    @Put('/format/trim')
    trimText(): string {
        return this.appService.trimText("   spaces around   ");
    }

    @Patch('/format/repeat')
    repeatText(): string {
        return this.appService.repeatText("ha", 5);
    }

    @Get('/format/length')
    getLength(): string {
        return this.appService.getLength("Tyrus Transpiler");
    }

    @Post('/users/alice')
    createAlice(): string {
        return this.appService.createUser("Alice", "alice@tyrus.dev");
    }

    @Post('/users/bob')
    createBob(): string {
        return this.appService.createUser("Bob", "bob@tyrus.dev");
    }

    @Delete('/reset')
    resetAll(): string {
        return "System reset complete";
    }
}
