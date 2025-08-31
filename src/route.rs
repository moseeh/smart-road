use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Route {
    Right,
    Left,
    Straight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

pub fn get_random_direction() -> Direction {
    let mut rng = rand::rng();
    match rng.random_range(0..4) {
        0 => Direction::East,
        1 => Direction::North,
        2 => Direction::South,
        _ => Direction::West,
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

pub fn get_turn_position(direction: Direction, route: Route) -> (f32, f32) {
    match route {
        Route::Straight => (0.0, 0.0),
        Route::Right => match direction {
            Direction::North => (600.0, 610.0),
            Direction::South => (350.0, 390.0),
            Direction::East => (400.0, 635.0),
            Direction::West => (600.0, 385.0),
        },
        Route::Left => match direction {
            Direction::North => (500.0, 470.0),
            Direction::South => (450.0, 540.0),
            Direction::East => (550.0, 535.0),
            Direction::West => (450.0, 485.0),
        },
    }
}
