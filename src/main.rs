use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use std::time::Duration;
mod intersection;
mod route;
mod vehicle;
mod stats;
mod velocities;

use intersection::*;
use route::*;
use stats::*;


// Constants for the game design
const WINDOW_WIDTH: u32 = 1000;
const WINDOW_HEIGHT: u32 = 1000;
const FRAME_DELAY: Duration = Duration::from_millis(16);

fn run_game(
    sdl_context: &sdl2::Sdl,
    video_subsystem: &sdl2::VideoSubsystem,
) -> Result<Option<String>, String> {
    let window = video_subsystem
        .window("SMART ROAD", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let road_texture =
        texture_creator.load_texture("assets/road-intersection/road-intersection.png")?;

    let mut intersection = SmartIntersection::new();
    let mut current_time = 0.0f32;

    let mut event_pump = sdl_context.event_pump()?;
    loop {
        current_time += 1.0 / 60.0;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return Ok(Some(intersection.get_final_stats())); // Quit the whole application
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return Ok(Some(intersection.get_final_stats()));
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Up => {
                        intersection.spawn_vehicle(
                            &texture_creator,
                            Some(Direction::North),
                            current_time,
                        );
                    }
                    Keycode::Down => {
                        intersection.spawn_vehicle(
                            &texture_creator,
                            Some(Direction::South),
                            current_time,
                        );
                    }
                    Keycode::Right => {
                        intersection.spawn_vehicle(
                            &texture_creator,
                            Some(Direction::East),
                            current_time,
                        );
                    }
                    Keycode::Left => {
                        intersection.spawn_vehicle(
                            &texture_creator,
                            Some(Direction::West),
                            current_time,
                        );
                    }
                    Keycode::R => {
                        intersection.spawn_vehicle(&texture_creator, None, current_time);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        intersection.update(current_time);

        canvas.clear();
        canvas.copy(&road_texture, None, None)?;

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
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    if let Some(stats) = run_game(&sdl_context, &video_subsystem)? {
        show_stats(&sdl_context, &video_subsystem, &ttf_context, &stats)?;
    }

    Ok(())
}
