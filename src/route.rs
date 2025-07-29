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

// Helper function to get spawn position based on direction and route
pub fn get_spawn_position(direction: Direction, route: Route) -> (f32, f32) {
    match direction {
        Direction::North => {
            let lane_x = match route {
                Route::Right => 600.0,    // Rightmost lane going north
                Route::Straight => 550.0, // Middle lane going north
                Route::Left => 500.0,     // Leftmost lane going north
            };
            (lane_x, 980.0) // Start at bottom of screen
        }
        Direction::South => {
            let lane_x = match route {
                Route::Right => 350.0,    // Rightmost lane going south
                Route::Straight => 400.0, // Middle lane going south
                Route::Left => 450.0,     // Leftmost lane going south
            };
            (lane_x, 0.0) // Start at top of screen
        }
        Direction::East => {
            let lane_y = match route {
                Route::Right => 600.0,    // Bottom lane going east
                Route::Straight => 550.0, // Middle lane going east
                Route::Left => 500.0,     // Top lane going east
            };
            (0.0, lane_y) // Start at left of screen
        }
        Direction::West => {
            let lane_y = match route {
                Route::Right => 350.0,    // Top lane going west
                Route::Straight => 400.0, // Middle lane going west
                Route::Left => 450.0,     // Bottom lane going west
            };
            (980.0, lane_y) // Start at right of screen
        }
    }
}
