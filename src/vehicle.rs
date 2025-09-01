use crate::route::{Direction, Route};
use crate::velocities::Velocity;
use rand::Rng;
use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Vehicle<'a> {
    pub id: usize,
    pub texture: Texture<'a>,
    pub route: Route,
    pub direction: Direction,
    pub current_speed: Velocity,
    pub width: u32,
    pub height: u32,
    pub safety_distance: f32,
    pub position: (f32, f32),
    pub turn_position: (f32, f32),
    pub rotation: f64,
    pub has_turned: bool,
    pub requested_intersection: bool,
    pub intersection_permission: bool,
}

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

impl<'a> Vehicle<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        route: Route,
        direction: Direction,
        spawn_position: (f32, f32),
        turn_position: (f32, f32),
    ) -> Result<Self, String> {
        let mut rng = rand::rng();
        let car_index = rng.random_range(1..=5);
        let path = format!("assets/Cars/car{}.png", car_index);
        let texture = texture_creator.load_texture(&path)?;
        // Set rotation based on direction
        let rotation = match direction {
            Direction::North => 0.0,   // No rotation (assuming cars face north in image)
            Direction::South => 180.0, // Flip around
            Direction::East => 90.0,   // Turn right 90 degrees
            Direction::West => 270.0,  // Turn left 90 degrees (or -90.0)
        };

        Ok(Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            texture,
            route,
            direction,
            current_speed: Velocity::Fast,
            width: 40,
            height: 70,
            safety_distance: 50.0,
            position: spawn_position,
            turn_position,
            rotation,
            has_turned: false,
            requested_intersection: false,
            intersection_permission: false,
        })
    }

    pub fn update(&mut self) {
        let pixels_per_frame = match self.current_speed {
            Velocity::Slow => 3.0,    // 3 pixel per frame
            Velocity::Medium => 5.0,  // 5 pixels per frame
            Velocity::Fast => 7.0,    // 7 pixels per frame
            Velocity::Stopped => 0.0, // vehicle doesnt move
        };

        if !self.has_turned {
            let center = (
                self.position.0 + self.width as f32 / 2.0,
                self.position.1 + self.height as f32 / 2.0,
            );
            let dx = center.0 - self.turn_position.0;
            let dy = center.1 - self.turn_position.1;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= 25.0 {
                self.execute_turn(); // change direction & rotation
                self.has_turned = true;
            }
        }

        match self.direction {
            Direction::North => self.position.1 -= pixels_per_frame,
            Direction::South => self.position.1 += pixels_per_frame,
            Direction::East => self.position.0 += pixels_per_frame,
            Direction::West => self.position.0 -= pixels_per_frame,
        }
    }
    pub fn get_visual_bounds(&self) -> (f32, f32, f32, f32) {
        let center_x = self.position.0 + self.width as f32 / 2.0;
        let center_y = self.position.1 + self.height as f32 / 2.0;

        match self.rotation as i32 % 360 {
            0 | 180 => {
                // No rotation change needed
                (
                    self.position.0,
                    self.position.1,
                    self.width as f32,
                    self.height as f32,
                )
            }
            90 | 270 => {
                // Width/height swap, position adjusts
                let visual_width = self.height as f32;
                let visual_height = self.width as f32;
                let visual_x = center_x - visual_width / 2.0;
                let visual_y = center_y - visual_height / 2.0;
                (visual_x, visual_y, visual_width, visual_height)
            }
            _ => (
                self.position.0,
                self.position.1,
                self.width as f32,
                self.height as f32,
            ),
        }
    }

    pub fn execute_turn(&mut self) {
        match self.route {
            Route::Right => match self.direction {
                Direction::North => {
                    self.direction = Direction::East;
                    self.rotation = 90.0;
                }
                Direction::South => {
                    self.direction = Direction::West;
                    self.rotation = 270.0;
                }
                Direction::East => {
                    self.direction = Direction::South;
                    self.rotation = 180.0;
                }
                Direction::West => {
                    self.direction = Direction::North;
                    self.rotation = 0.0;
                }
            },
            Route::Left => match self.direction {
                Direction::North => {
                    self.direction = Direction::West;
                    self.rotation = 270.0;
                }
                Direction::South => {
                    self.direction = Direction::East;
                    self.rotation = 90.0;
                }
                Direction::East => {
                    self.direction = Direction::North;
                    self.rotation = 0.0;
                }
                Direction::West => {
                    self.direction = Direction::South;
                    self.rotation = 180.0;
                }
            },
            Route::Straight => {} // no turn
        }
    }
    pub fn get_visual_center(&self) -> (f32, f32) {
        let (vx, vy, vw, vh) = self.get_visual_bounds();
        (vx + vw / 2.0, vy + vh / 2.0)
    }
    pub fn distance_to_intersection(&self) -> f32 {
        let (vx, vy, vw, vh) = self.get_visual_bounds();
        let center = (vx + vw / 2.0, vy + vh / 2.0);

        match self.direction {
            Direction::North => {
                if center.1 > 650.0 {
                    center.1 - 650.0
                } else {
                    0.0
                }
            }
            Direction::South => {
                if center.1 < 350.0 {
                    350.0 - center.1
                } else {
                    0.0
                }
            }
            Direction::East => {
                if center.0 < 350.0 {
                    350.0 - center.0
                } else {
                    0.0
                }
            }
            Direction::West => {
                if center.0 > 650.0 {
                    center.0 - 650.0
                } else {
                    0.0
                }
            }
        }
    }

    pub fn is_in_intersection(&self) -> bool {
        let (vx, vy, vw, vh) = self.get_visual_bounds();
        // Check if any part of visual bounds overlaps intersection
        let right = vx + vw;
        let bottom = vy + vh;

        !(right < 350.0 || vx > 650.0 || bottom < 350.0 || vy > 650.0)
    }

    pub fn is_in_same_lane(&self, other: &Vehicle) -> bool {
        if self.direction != other.direction {
            return false;
        }
        if self.route != other.route {
            return false;
        }
        true
    }

    pub fn is_ahead_of_me(&self, other: &Vehicle) -> bool {
        if !self.is_in_same_lane(other) {
            return false;
        }

        let my_center = self.get_visual_center();
        let other_center = other.get_visual_center();

        match self.direction {
            Direction::North => other_center.1 < my_center.1,
            Direction::South => other_center.1 > my_center.1,
            Direction::East => other_center.0 > my_center.0,
            Direction::West => other_center.0 < my_center.0,
        }
    }
    pub fn is_past_intersection(&self) -> bool {
        let (vx, vy, vw, vh) = self.get_visual_bounds();

        match self.direction {
            Direction::North => vy + vh < 350.0, // Entire vehicle past intersection
            Direction::South => vy > 650.0,
            Direction::East => vx > 650.0,
            Direction::West => vx + vw < 350.0,
        }
    }

    pub fn distance_to_vehicle(&self, other: &Vehicle) -> f32 {
        let my_center = self.get_visual_center();
        let other_center = other.get_visual_center();

        match self.direction {
            Direction::North | Direction::South => (my_center.1 - other_center.1).abs(),
            Direction::East | Direction::West => (my_center.0 - other_center.0).abs(),
        }
    }
    fn calculate_exit_position(&self) -> (f32, f32) {
        // Use the turn position to determine the appropriate exit position
        let turn_pos = self.turn_position;

        // Determine final direction after turn
        let final_direction = match (self.direction, self.route) {
            (Direction::North, Route::Right) => Direction::East,
            (Direction::North, Route::Left) => Direction::West,
            (Direction::South, Route::Right) => Direction::West,
            (Direction::South, Route::Left) => Direction::East,
            (Direction::East, Route::Right) => Direction::South,
            (Direction::East, Route::Left) => Direction::North,
            (Direction::West, Route::Right) => Direction::North,
            (Direction::West, Route::Left) => Direction::South,
            _ => self.direction,
        };

        // Exit position maintains the same lane position (x or y) as the turn position
        match final_direction {
            Direction::North => (turn_pos.0, 0.0), // Keep x from turn, exit at top
            Direction::South => (turn_pos.0, 1000.0), // Keep x from turn, exit at bottom
            Direction::East => (1000.0, turn_pos.1), // Keep y from turn, exit at right
            Direction::West => (0.0, turn_pos.1),  // Keep y from turn, exit at left
        }
    }

    pub fn get_safe_following_distance(&self, _lead_vehicle: &Vehicle) -> f32 {
        70.0 + self.safety_distance
    }

    pub fn is_outside_canvas(&self) -> bool {
        self.position.0 < 0.0
            || self.position.0 > 1000.0
            || self.position.1 < 0.0
            || self.position.1 > 1000.0
    }
}
