use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::image::LoadTexture;
use rand::Rng;

pub struct VehicleTexture<'a> {
    pub texture: Texture<'a>,
}

impl<'a> VehicleTexture<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        // Pick a random number from 1 to 5
        let mut rng = rand::thread_rng();
        let car_index = rng.gen_range(1..=5); // inclusive range

        let path = format!("assets/car{}.png", car_index);
        let texture = texture_creator.load_texture(&path)?;

        Ok(Self { texture })
    }
}
