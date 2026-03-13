interface User {
    name: string;
    age: number;
    email: string;
    active: boolean;
}

interface Product {
    id: number;
    title: string;
    price: number;
    description?: string;
}

interface ApiResponse {
    data: User[];
    total: number;
    success: boolean;
}
