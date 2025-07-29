use rand::Rng;

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

// Helper function to get random route
pub fn get_random_route() -> Route {
    let mut rng = rand::rng();
    match rng.random_range(0..3) {
        0 => Route::Right,
        1 => Route::Straight,
        _ => Route::Left,
    }
}
