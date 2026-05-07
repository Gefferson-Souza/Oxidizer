import { Injectable } from "@nestjs/common";
import { CreateItemDto, Item } from "./dto";

@Injectable()
export class AppService {
    private items: Item[];
    private nextId: number;

    constructor() {
        this.items = [];
        this.nextId = 1;
    }

    findAll(): Item[] {
        return this.items;
    }

    create(dto: CreateItemDto): Item {
        const item: Item = {
            id: this.nextId,
            name: dto.name,
            quantity: dto.quantity,
        };
        this.items.push(item);
        this.nextId = this.nextId + 1;
        return item;
    }
}
