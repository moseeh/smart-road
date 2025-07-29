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
}

impl<'a> Vehicle<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        route: Route,
        direction: Direction,
        spawn_position: (f32, f32),
    ) -> Result<Self, String> {
        let mut rng = rand::rng();
        let car_index = rng.random_range(1..=5);
        let path = format!("assets/Cars/car{}.png", car_index);
        let texture = texture_creator.load_texture(&path)?;

        Ok(Self {
            texture,
            route,
            direction,
            current_speed: Velocity::Medium,
            width: 40,
            height: 70,
            safety_distance: 50.0,
            position: spawn_position,
        })
    }
}
