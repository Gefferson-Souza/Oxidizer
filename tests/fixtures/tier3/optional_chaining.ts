interface Profile {
    name: string;
    address?: string;
}

interface User {
    id: number;
    profile?: Profile;
}

function getUserName(user: User): string {
    const name: string = user.profile?.name ?? "Unknown";
    return name;
}

function getAddress(user: User): string {
    return user.profile?.address ?? "No address";
}
