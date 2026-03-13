interface Point {
    x: number;
    y: number;
}

function getCoords(point: Point): number {
    const { x, y } = point;
    return x + y;
}
