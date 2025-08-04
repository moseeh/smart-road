use crate::route::{Direction, Route};
use crate::velocities::Velocity;
use rand::Rng;
use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

pub struct Vehicle<'a> {
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
}

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
        })
    }

    pub fn update(&mut self) {
        // No delta_time parameter needed
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
}
