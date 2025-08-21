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
use vehicle::Vehicle;
use velocities::Velocity;

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

    // Initialize intersection and vehicle storage
    let mut intersection = SmartIntersection::new();
    let mut vehicles: Vec<Vehicle> = Vec::new();
    let mut current_time = 0.0f32;

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        // Increment time (assuming 60 FPS = 1/60 second per frame)
        current_time += 1.0 / 60.0;

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

        // Update all vehicles with smart intersection management
        update_vehicles_with_intersection(&mut vehicles, &mut intersection, current_time);

        // Remove vehicles that have left the canvas
        vehicles.retain(|vehicle| !vehicle.is_outside_canvas());

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

/// Update all vehicles with traffic and intersection management
fn update_vehicles_with_intersection(
    vehicles: &mut Vec<Vehicle>,
    intersection: &mut SmartIntersection,
    current_time: f32,
) {
    // Calculate traffic speeds for all vehicles first
    let mut target_speeds = Vec::with_capacity(vehicles.len());

    for i in 0..vehicles.len() {
        let current_vehicle = &vehicles[i];

        // If vehicle is past intersection, it can go fast (no collision risk)
        if current_vehicle.is_past_intersection() {
            target_speeds.push(Velocity::Fast);
            continue;
        }

        let mut target_speed = Velocity::Fast;
        let mut closest_distance = f32::MAX;
        let mut required_distance = 0.0;

        // Check traffic by manually iterating through other vehicles
        for (j, other_vehicle) in vehicles.iter().enumerate() {
            if i == j {
                continue;
            }

            // Only check vehicles that are ahead and in same lane
            if current_vehicle.is_ahead_of_me(other_vehicle) {
                let distance = current_vehicle.distance_to_vehicle(other_vehicle);
                if distance < closest_distance {
                    closest_distance = distance;
                    required_distance = current_vehicle.get_safe_following_distance(other_vehicle);
                }
            }
        }

        // Determine speed based on closest vehicle ahead
        if closest_distance != f32::MAX && closest_distance < required_distance {
            if closest_distance < required_distance * 0.4 {
                target_speed = Velocity::Slow; // Very close - slow down significantly
            } else if closest_distance < required_distance * 0.8 {
                target_speed = Velocity::Medium; // Getting close - moderate speed
            }
            // If distance >= required_distance * 0.8, keep Fast speed
        }

        target_speeds.push(target_speed);
    }

    // Now update each vehicle
    for i in 0..vehicles.len() {
        let vehicle = &mut vehicles[i];

        // Reset intersection status if vehicle is far away
        intersection.reset_vehicle_intersection_status(vehicle);

        let traffic_speed = target_speeds[i];

        // If vehicle is past intersection, ignore intersection management
        let final_speed = if vehicle.is_past_intersection() {
            Velocity::Fast // Full speed past intersection
        } else {
            // Check intersection requirements
            let intersection_speed =
                intersection.manage_vehicle_intersection_approach(vehicle, current_time);

            // Take the slower of traffic and intersection requirements
            match (traffic_speed, intersection_speed) {
                (Velocity::Slow, _) | (_, Velocity::Slow) => Velocity::Slow,
                (Velocity::Medium, _) | (_, Velocity::Medium) => Velocity::Medium,
                (Velocity::Fast, Velocity::Fast) => Velocity::Fast,
            }
        };

        // Apply the speed
        vehicle.current_speed = final_speed;

        // Update vehicle position
        vehicle.update();

        // Release cells behind the vehicle (only if in intersection area)
        if vehicle.is_in_intersection() || vehicle.distance_to_intersection() < 50.0 {
            release_cells_behind_vehicle(intersection, vehicle);
        }
    }
}

pub fn is_safe_to_spawn(
    vehicles: &[Vehicle],
    direction: Direction,
    route: Route,
    spawn_pos: (f32, f32),
) -> bool {
    // Create a temporary vehicle to get its visual bounds at spawn position
    let temp_rotation = match direction {
        Direction::North => 0.0,
        Direction::South => 180.0,
        Direction::East => 90.0,
        Direction::West => 270.0,
    };

    // Calculate visual bounds for the spawn vehicle
    let width = 40.0;
    let height = 70.0;
    let center_x = spawn_pos.0 + width / 2.0;
    let center_y = spawn_pos.1 + height / 2.0;
    
    let (spawn_vx, spawn_vy, spawn_vw, spawn_vh) = match temp_rotation as i32 % 360 {
        0 | 180 => (spawn_pos.0, spawn_pos.1, width, height),
        90 | 270 => {
            let visual_width = height;
            let visual_height = width;
            let visual_x = center_x - visual_width / 2.0;
            let visual_y = center_y - visual_height / 2.0;
            (visual_x, visual_y, visual_width, visual_height)
        },
        _ => (spawn_pos.0, spawn_pos.1, width, height)
    };

    let spawn_visual_center = (spawn_vx + spawn_vw / 2.0, spawn_vy + spawn_vh / 2.0);

    for vehicle in vehicles
        .iter()
        .filter(|v| v.direction == direction && v.route == route)
    {
        let other_visual_center = vehicle.get_visual_center();
        let (_, _, other_vw, other_vh) = vehicle.get_visual_bounds();

        // Calculate distance between visual bounds
        let distance = match direction {
            Direction::North => {
                if other_visual_center.1 < spawn_visual_center.1 {
                    spawn_visual_center.1 - other_visual_center.1 - (spawn_vh / 2.0 + other_vh / 2.0)
                } else {
                    continue;
                }
            }
            Direction::South => {
                if other_visual_center.1 > spawn_visual_center.1 {
                    other_visual_center.1 - spawn_visual_center.1 - (spawn_vh / 2.0 + other_vh / 2.0)
                } else {
                    continue;
                }
            }
            Direction::East => {
                if other_visual_center.0 > spawn_visual_center.0 {
                    other_visual_center.0 - spawn_visual_center.0 - (spawn_vw / 2.0 + other_vw / 2.0)
                } else {
                    continue;
                }
            }
            Direction::West => {
                if other_visual_center.0 < spawn_visual_center.0 {
                    spawn_visual_center.0 - other_visual_center.0 - (spawn_vw / 2.0 + other_vw / 2.0)
                } else {
                    continue;
                }
            }
        };

        // Check if distance is safe
        if distance < vehicle.safety_distance {
            return false;
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
