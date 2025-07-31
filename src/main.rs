use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::time::Duration;
mod route;
mod vehicle;
mod velocities;

use route::{Direction, Route};
use vehicle::Vehicle;

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
    let road_texture =
        texture_creator.load_texture("assets/road-intersection/road-intersection.png")?;

    // Add vehicle storage
    let mut vehicles: Vec<Vehicle> = Vec::new();

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                // Vehicle creation events
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // Generate vehicle from south to north
                    let route = get_random_route();
                    let spawn_pos = get_spawn_position(Direction::North, route);
                    match Vehicle::new(&texture_creator, route, Direction::North, spawn_pos) {
                        Ok(vehicle) => vehicles.push(vehicle),
                        Err(e) => println!("Failed to create vehicle: {}", e),
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // Generate vehicle from north to south
                    let route = get_random_route();
                    let spawn_pos = get_spawn_position(Direction::South, route);
                    match Vehicle::new(&texture_creator, route, Direction::South, spawn_pos) {
                        Ok(vehicle) => vehicles.push(vehicle),
                        Err(e) => println!("Failed to create vehicle: {}", e),
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // Generate vehicle from west to east
                    let route = get_random_route();
                    let spawn_pos = get_spawn_position(Direction::East, route);
                    match Vehicle::new(&texture_creator, route, Direction::East, spawn_pos) {
                        Ok(vehicle) => vehicles.push(vehicle),
                        Err(e) => println!("Failed to create vehicle: {}", e),
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // Generate vehicle from east to west
                    // TODO: Add spawn logic
                }
                _ => {}
            }
        }

        // Clear screen, draw road, draw vehicles, update screen
        canvas.clear();
        canvas.copy(&road_texture, None, None)?;

        // TODO: Render vehicles here

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
