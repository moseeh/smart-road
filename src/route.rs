#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Route {
    Right,
    Left,
    Straight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    North, // Coming from south, going north
    South, // Coming from north, going south
    East,  // Coming from west, going east
    West,  // Coming from east, going west
}
