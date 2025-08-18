use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::time::Duration;
mod route;
mod vehicle;
mod velocities;

use route::*;
use vehicle::Vehicle;

// Constants for the game design
const WINDOW_WIDTH: u32 = 1000;
const WINDOW_HEIGHT: u32 = 1000;
const FRAME_DELAY: Duration = Duration::from_millis(16);

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Initialize SDL2_image (for PNG/JPG support)
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

    // Create window and canvas
    let window = video_subsystem
        .window("SMART ROAD", WINDOW_WIDTH, WINDOW_HEIGHT)
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
                    keycode: Some(key), ..
                } => {
                    let direction = match key {
                        Keycode::Up => Some(Direction::North),
                        Keycode::Down => Some(Direction::South),
                        Keycode::Right => Some(Direction::East),
                        Keycode::Left => Some(Direction::West),
                        Keycode::R => None, // Random direction
                        _ => continue,
                    };
                    spawn_vehicle_for_direction(&mut vehicles, &texture_creator, direction);
                }
                _ => {}
            }
        }

        // Update all vehicles
        for vehicle in &mut vehicles {
            vehicle.update();
        }

        // Clear screen and draw
        canvas.clear();
        canvas.copy(&road_texture, None, None)?;

        for vehicle in &vehicles {
            let dest_rect = sdl2::rect::Rect::new(
                vehicle.position.0 as i32,
                vehicle.position.1 as i32,
                vehicle.width,
                vehicle.height,
            );

            canvas.copy_ex(
                &vehicle.texture,
                None,
                dest_rect,
                vehicle.rotation,
                None,
                false,
                false,
            )?;
        }

        canvas.present();
        std::thread::sleep(FRAME_DELAY);
    }

    Ok(())
}

pub fn is_safe_to_spawn(
    vehicles: &[Vehicle],
    direction: Direction,
    route: Route,
    spawn_pos: (f32, f32),
) -> bool {
    // vehicle size constants (same as Vehicle::new)
    let width = 40.0;
    let height = 70.0;

    // Calculate spawn vehicle's bounding box center
    let center = (spawn_pos.0 + width / 2.0, spawn_pos.1 + height / 2.0);

    for vehicle in vehicles
        .iter()
        .filter(|v| v.direction == direction && v.route == route)
    {
        let other_center = (
            vehicle.position.0 + vehicle.width as f32 / 2.0,
            vehicle.position.1 + vehicle.height as f32 / 2.0,
        );

        match direction {
            Direction::North => {
                // cars move UP (y decreasing)
                // check if existing car is ahead of the spawn (smaller y)
                if other_center.1 < center.1 {
                    let dist = center.1 - other_center.1 - vehicle.height as f32 / 2.0;
                    if dist < vehicle.safety_distance {
                        return false;
                    }
                }
            }
            Direction::South => {
                // cars move DOWN (y increasing)
                if other_center.1 > center.1 {
                    let dist = other_center.1 - center.1 - vehicle.height as f32 / 2.0;
                    if dist < vehicle.safety_distance {
                        return false;
                    }
                }
            }
            Direction::East => {
                // cars move RIGHT (x increasing)
                if other_center.0 > center.0 {
                    let dist = other_center.0 - center.0 - vehicle.width as f32 / 2.0;
                    if dist < vehicle.safety_distance {
                        return false;
                    }
                }
            }
            Direction::West => {
                // cars move LEFT (x decreasing)
                if other_center.0 < center.0 {
                    let dist = center.0 - other_center.0 - vehicle.width as f32 / 2.0;
                    if dist < vehicle.safety_distance {
                        return false;
                    }
                }
            }
        }
    }

    true
}

fn spawn_vehicle_for_direction<'a>(
    vehicles: &mut Vec<Vehicle<'a>>,
    texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    direction: Option<Direction>,
) {
    let dir = match direction {
        Some(d) => d,
        None => get_random_direction(),
    };

    let route = get_random_route();
    let spawn_pos = get_spawn_position(dir, route);
    let turn_pos = get_turn_position(dir, route);

    if is_safe_to_spawn(vehicles, dir, route, spawn_pos) {
        match Vehicle::new(texture_creator, route, dir, spawn_pos, turn_pos) {
            Ok(vehicle) => vehicles.push(vehicle),
            Err(e) => println!("Failed to create vehicle: {}", e),
        }
    }
}
