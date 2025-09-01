use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
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
                    return Ok(Some(intersection.get_final_stats()));// Quit the whole application
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
                        intersection.spawn_vehicle(&texture_creator, None);
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

fn show_stats(
    sdl_context: &sdl2::Sdl,
    video_subsystem: &sdl2::VideoSubsystem,
    ttf_context: &sdl2::ttf::Sdl2TtfContext,
    stats_text: &str,
) -> Result<(), String> {
    let window = video_subsystem
        .window("Statistics", 1000, 1000)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let font = ttf_context.load_font("assets/fonts/OpenSans-Bold.ttf", 24)?;

    let mut event_pump = sdl_context.event_pump()?;
    
    let lines: Vec<&str> = stats_text.split('\n').collect();
    let mut y = 50;

    'stats_running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'stats_running;
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();

        for line in &lines {
            if line.is_empty() {
                y += 10; // Add some space for empty lines
                continue;
            }
            let surface = font
                .render(line)
                .shaded(Color::RGBA(0, 0, 0, 255), Color::RGBA(0, 255, 255, 255))
                .map_err(|e| e.to_string())?;
            
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;

            let query = texture.query();
            let target_rect = Rect::new(
                (1000 - query.width as i32) / 2,
                y,
                query.width,
                query.height,
            );
            canvas.copy(&texture, None, target_rect)?;
            y += query.height as i32 + 5; // Move y for the next line
        }
        y = 50; // Reset y for the next frame

        canvas.present();
        std::thread::sleep(FRAME_DELAY);
    }

    Ok(())
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