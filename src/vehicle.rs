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
        })
    }

    pub fn update(&mut self) {
        // No delta_time parameter needed
        let pixels_per_frame = match self.current_speed {
            Velocity::Slow => 3.0,   // 3 pixel per frame 
            Velocity::Medium => 5.0, // 5 pixels per frame 
            Velocity::Fast => 7.0,   // 7 pixels per frame 
        };

        match self.direction {
            Direction::North => self.position.1 -= pixels_per_frame,
            Direction::South => self.position.1 += pixels_per_frame,
            Direction::East => self.position.0 += pixels_per_frame,
            Direction::West => self.position.0 -= pixels_per_frame,
        }
    }
}
