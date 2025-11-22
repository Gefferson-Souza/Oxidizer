// HTTP Client Mapping Test
// Tests: axios.get/post/put/delete and fetch() transpilation to reqwest

interface User {
    id: number;
    name: string;
    email: string;
}

interface ApiResponse {
    success: boolean;
    data: string;
}

// Axios GET
async function getWithAxios(id: number): Promise<User> {
    return await axios.get("https://api.com/users/" + id);
}

// Axios POST
async function postWithAxios(user: User): Promise<ApiResponse> {
    return await axios.post("https://api.com/users", user);
}

// Axios PUT
async function updateWithAxios(id: number, user: User): Promise<User> {
    return await axios.put("https://api.com/users/" + id, user);
}

// Axios DELETE
async function deleteWithAxios(id: number): Promise<void> {
    return await axios.delete("https://api.com/users/" + id);
}

// Fetch
async function getWithFetch(url: string): Promise<any> {
    return await fetch(url);
}

// Fetch with explicit URL
async function fetchUsers(): Promise<any> {
    return await fetch("https://api.com/users");
}
