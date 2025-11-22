// Complex E2E Test - Full Stack TypeScript to Rust
// Tests: Interfaces, Classes, Async Functions, Type Mappings

// Data Transfer Objects
interface User {
    id: number;
    name: string;
    email: string;
    isActive: boolean;
}

interface ApiResponse {
    success: boolean;
    data: string;
    timestamp: number;
}

// Service Class with Async Methods
class UserService {
    baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
    }

    async fetchUser(id: number): Promise<User> {
        return await getFromDatabase(id);
    }

    async saveUser(user: User): Promise<ApiResponse> {
        return await postToDatabase(user);
    }

    getBaseUrl(): string {
        return this.baseUrl;
    }
}

// Helper Functions
async function getFromDatabase(id: number): Promise<User> {
    return {
        id: id,
        name: "Test User",
        email: "test@example.com",
        isActive: true
    };
}

async function postToDatabase(user: User): Promise<ApiResponse> {
    return {
        success: true,
        data: user.name,
        timestamp: 1234567890
    };
}

// Pure function with math
function calculateTotal(a: number, b: number, c: number): number {
    return a + b + c;
}
