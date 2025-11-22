// Real-world E2E Test - Full Stack TypeScript to Rust
// Uses JSONPlaceholder API for real HTTP requests

interface Post {
    userId: number;
    id: number;
    title: string;
    body: string;
}

interface User {
    id: number;
    name: string;
    email: string;
}

interface Comment {
    postId: number;
    id: number;
    name: string;
    email: string;
    body: string;
}

// Service class with real API calls
class ApiService {
    baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
    }

    async getPosts(): Promise<Post> {
        return await fetch("https://jsonplaceholder.typicode.com/posts/1");
    }

    async getUser(id: number): Promise<User> {
        return await axios.get("https://jsonplaceholder.typicode.com/users/" + id);
    }

    async createPost(post: Post): Promise<Post> {
        return await axios.post("https://jsonplaceholder.typicode.com/posts", post);
    }

    async updatePost(id: number, post: Post): Promise<Post> {
        return await axios.put("https://jsonplaceholder.typicode.com/posts/" + id, post);
    }

    async deletePost(id: number): Promise<void> {
        return await axios.delete("https://jsonplaceholder.typicode.com/posts/" + id);
    }

    getBaseUrl(): string {
        return this.baseUrl;
    }
}

// Helper function for processing
function calculateTotal(a: number, b: number, c: number): number {
    return a + b + c;
}

// Async orchestration
async function fetchAndProcess(userId: number): Promise<number> {
    return await processData(userId);
}

async function processData(id: number): Promise<number> {
    return calculateTotal(id, 10, 20);
}
