// Complex E2E scenario combining all Milestone features
// Milestone 2: Interfaces (DTOs)
interface User {
    id: number;
    name: string;
    isActive: boolean;
}

export interface ApiResponse {
    status: number;
    data: User;
}

// Milestone 3: Functions with Math
export function calculate_age(birthYear: number, currentYear: number): number {
    return currentYear - birthYear;
}

export function sum(a: number, b: number, c: number): number {
    return a + b + c;
}

// Helper functions for database operations
async function getFromDatabase(id: number): Promise<User> {
    // Simulated database fetch
    return {
        id: id,
        name: "Test User",
        isActive: true
    };
}

async function postToDatabase(user: User): Promise<ApiResponse> {
    // Simulated database save
    return {
        status: 200,
        data: user
    };
}

// Milestone 4: Async/Await
export async function fetch_user(id: number): Promise<User> {
    await dummy_async();
    return { id: id, name: "Test", isActive: true };
}

async function dummy_async(): Promise<void> { }

export async function save_user(user: User): Promise<ApiResponse> {
    return { status: 200, data: user };
}

// Mixed sync/async
export function process_user(user: User): void {
    let age = calculate_age(1990, 2024);
}

