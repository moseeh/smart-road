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
            Velocity::Slow => 3.0,   // 3 pixel per frame
            Velocity::Medium => 5.0, // 5 pixels per frame
            Velocity::Fast => 7.0,   // 7 pixels per frame
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

    // Vehicle methods for collision detection and safety
    pub fn get_center(&self) -> (f32, f32) {
        (
            self.position.0 + self.width as f32 / 2.0,
            self.position.1 + self.height as f32 / 2.0,
        )
    }
    pub fn get_visual_center(&self) -> (f32, f32) {
        let (vx, vy, vw, vh) = self.get_visual_bounds();
        (vx + vw / 2.0, vy + vh / 2.0)
    }

    pub fn get_effective_dimensions(&self) -> (f32, f32) {
        let (_, _, vw, vh) = self.get_visual_bounds();
        (vw, vh)
    }
    pub fn get_front_position(&self) -> (f32, f32) {
        let (vx, vy, vw, vh) = self.get_visual_bounds();
        let center = (vx + vw / 2.0, vy + vh / 2.0);

        match self.direction {
            Direction::North => (center.0, vy),
            Direction::South => (center.0, vy + vh),
            Direction::East => (vx + vw, center.1),
            Direction::West => (vx, center.1),
        }
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

        let my_center = self.get_center();
        let other_center = other.get_center();
        let lane_tolerance = 30.0; // Allow some variance

        match self.direction {
            Direction::North | Direction::South => {
                (my_center.0 - other_center.0).abs() < lane_tolerance
            }
            Direction::East | Direction::West => {
                (my_center.1 - other_center.1).abs() < lane_tolerance
            }
        }
    }

    pub fn is_ahead_of_me(&self, other: &Vehicle) -> bool {
        if !self.is_in_same_lane(other) {
            return false;
        }

        let my_center = self.get_center();
        let other_center = other.get_center();

        match self.direction {
            Direction::North => other_center.1 < my_center.1,
            Direction::South => other_center.1 > my_center.1,
            Direction::East => other_center.0 > my_center.0,
            Direction::West => other_center.0 < my_center.0,
        }
    }

    pub fn is_past_intersection(&self) -> bool {
        let center = self.get_center();

        match self.direction {
            Direction::North => center.1 < 350.0, // Past intersection going north
            Direction::South => center.1 > 650.0, // Past intersection going south
            Direction::East => center.0 > 650.0,  // Past intersection going east
            Direction::West => center.0 < 350.0,  // Past intersection going west
        }
    }

    pub fn distance_to_vehicle(&self, other: &Vehicle) -> f32 {
        let my_center = self.get_center();
        let other_center = other.get_center();

        match self.direction {
            Direction::North | Direction::South => (my_center.1 - other_center.1).abs(),
            Direction::East | Direction::West => (my_center.0 - other_center.0).abs(),
        }
    }

    pub fn get_safe_following_distance(&self, lead_vehicle: &Vehicle) -> f32 {
        let my_length = match self.direction {
            Direction::North | Direction::South => self.height as f32,
            Direction::East | Direction::West => self.width as f32,
        };

        let lead_length = match lead_vehicle.direction {
            Direction::North | Direction::South => lead_vehicle.height as f32,
            Direction::East | Direction::West => lead_vehicle.width as f32,
        };

        // Safe distance = half of each car's length + safety buffer
        (my_length / 2.0) + (lead_length / 2.0) + self.safety_distance
    }

    pub fn should_slow_for_traffic(&self, vehicles: &[Vehicle]) -> Option<Velocity> {
        let mut closest_distance = f32::MAX;
        let mut required_distance = 0.0;

        for other in vehicles {
            if other.id == self.id || !self.is_ahead_of_me(other) {
                continue;
            }

            let distance = self.distance_to_vehicle(other);
            if distance < closest_distance {
                closest_distance = distance;
                required_distance = self.get_safe_following_distance(other);
            }
        }

        if closest_distance == f32::MAX {
            return None; // No vehicle ahead
        }

        // Determine speed based on distance to vehicle ahead
        if closest_distance < required_distance * 0.3 {
            Some(Velocity::Slow) // Very close
        } else if closest_distance < required_distance * 0.7 {
            Some(Velocity::Medium) // Getting close
        } else {
            None // Safe distance
        }
    }

    pub fn is_outside_canvas(&self) -> bool {
        self.position.0 < -100.0
            || self.position.0 > 1100.0
            || self.position.1 < -100.0
            || self.position.1 > 1100.0
    }
}
