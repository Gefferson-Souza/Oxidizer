interface UserData {
    id: number;
    name: string;
}

async function fetchUser(id: number): Promise<UserData> {
    const response = await fetch(`https://api.example.com/users/${id}`);
    const data: UserData = await response.json();
    return data;
}

async function processUser(id: number): Promise<string> {
    const user: UserData = await fetchUser(id);
    return user.name;
}
