import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';

async function bootstrap() {
    const app = await NestFactory.create(AppModule, { logger: false });
    await app.listen(3000);
    console.log('NestJS running on http://0.0.0.0:3000');
}
bootstrap();
