use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::time::Duration;

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Initialize SDL2_image (for PNG/JPG support)
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

    // Create window and canvas
    let window = video_subsystem
        .window("SMART ROAD", 1000, 1000)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync() // limits framerate to monitor's refresh rate
        .build()
        .map_err(|e| e.to_string())?;

    // Load the road image from assets
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/road-intersection/road-intersection.png")?;

    // Setup event loop
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Clear screen, draw texture, update screen
        canvas.clear();
        canvas.copy(&texture, None, None)?; // draw entire texture to the full window
        canvas.present();

        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    Ok(())
}
