import { Body, Controller, Get, Post } from "@nestjs/common";
import { AppService } from "./app.service";
import { CreateItemDto, Item } from "./dto";

@Controller("/items")
export class AppController {
    constructor(private readonly appService: AppService) {}

    @Get("/")
    findAll(): Item[] {
        return this.appService.findAll();
    }

    @Post("/")
    create(@Body() dto: CreateItemDto): Item {
        return this.appService.create(dto);
    }
}
