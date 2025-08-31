use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::time::Duration;
mod intersection;
mod route;
mod vehicle;
mod velocities;

use intersection::*;
use route::*;


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

    // Initialize intersection - now it manages everything
    let mut intersection = SmartIntersection::new();
    let mut current_time = 0.0f32;
    let mut frame_count = 0u32;

    println!("🚦 Smart Intersection Simulation Started!");
    println!("Controls:");
    println!("  Arrow Keys - Spawn vehicles from specific directions");
    println!("  R - Spawn vehicle from random direction");
    println!("  ESC - Exit simulation");
    println!("  G - Print current grid state");
    println!("  S - Print grid statistics");
    println!("  V - Print grid with vehicle IDs");

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        // Increment time (assuming 60 FPS = 1/60 second per frame)
        current_time += 1.0 / 60.0;
        frame_count += 1;

        // Print grid state periodically (every 3 seconds)
        if frame_count % 120 == 0 {
            intersection.print_grid_stats(current_time);
        }

        // Print detailed grid every 5 seconds
        if frame_count % 180 == 0 {
            intersection.print_grid(current_time);
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    // Print final statistics before exiting
                    intersection.print_final_stats();
                    break 'running;
                }

                // Vehicle creation events
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    match key {
                        // Direction-specific spawning
                        Keycode::Up => {
                            intersection.spawn_vehicle(&texture_creator, Some(Direction::North));
                        }
                        Keycode::Down => {
                            intersection.spawn_vehicle(&texture_creator, Some(Direction::South));
                        }
                        Keycode::Right => {
                            intersection.spawn_vehicle(&texture_creator, Some(Direction::East));
                        }
                        Keycode::Left => {
                            intersection.spawn_vehicle(&texture_creator, Some(Direction::West));
                        }
                        Keycode::R => {
                            intersection.spawn_vehicle(&texture_creator, None); // Random direction
                        }

                        // Debug output controls
                        Keycode::G => {
                            println!("\n🔍 Manual Grid State Request:");
                            intersection.print_grid(current_time);
                        }
                        Keycode::S => {
                            println!("\n📊 Manual Statistics Request:");
                            intersection.print_grid_stats(current_time);
                        }
                        Keycode::V => {
                            println!("\n🚗 Manual Vehicle ID Grid Request:");
                            intersection.print_grid_with_vehicle_ids(current_time);
                        }
                        Keycode::D => {
                            // Print details about all active vehicle paths
                            println!("\n🛣️  Active Vehicle Path Details:");
                            for vehicle in &intersection.active_vehicles {
                                if !vehicle.is_past_intersection() {
                                    let distance = vehicle.distance_to_intersection();
                                    println!(
                                        "Vehicle {}: {:?} {:?}, Distance to intersection: {:.1}",
                                        vehicle.id, vehicle.direction, vehicle.route, distance
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Update intersection (handles all vehicles)
        intersection.update(current_time);

        // Clear screen and draw
        canvas.clear();
        canvas.copy(&road_texture, None, None)?;

        // Draw all vehicles managed by intersection
        for vehicle in &intersection.active_vehicles {
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
