use rand::Rng;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

struct AnimatedCar {
    x: f32,
    y: f32,
    rotation: f64,
    car_type: u32, // 1-5 for different car textures
}

impl AnimatedCar {
    fn new() -> Self {
        let mut rng = rand::rng();

        // Random spawn position anywhere on canvas
        let x = rng.random_range(0.0..1000.0);
        let y = rng.random_range(0.0..1000.0);

        let rotation = 135.0;

        let car_type = rng.random_range(1..=5);

        Self {
            x,
            y,
            rotation,
            car_type,
        }
    }

    fn update(&mut self) {
        self.x += 2.0;
        self.y += 2.0;
    }

    fn is_off_screen(&self) -> bool {
        self.x < -100.0 || self.x > 1100.0 || self.y < -100.0 || self.y > 1100.0
    }

    fn should_render_behind_stats(&self) -> bool {
        // Check if car is in the stats box area (center of screen)
        let stats_left = 100.0;
        let stats_right = 850.0;
        let stats_top = 150.0;
        let stats_bottom = 800.0;

        self.x >= stats_left
            && self.x <= stats_right
            && self.y >= stats_top
            && self.y <= stats_bottom
    }
}

pub fn show_stats(
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

    // Load a better font with larger size
    let font = ttf_context.load_font("assets/fonts/Orbitron-VariableFont_wght.ttf", 28)?;
    let title_font = ttf_context.load_font("assets/fonts/Orbitron-VariableFont_wght.ttf", 36)?;

    // Load car textures
    let mut car_textures = Vec::new();
    for i in 1..=5 {
        let path = format!("assets/Cars/car{}.png", i);
        let texture = texture_creator.load_texture(&path)?;
        car_textures.push(texture);
    }

    let mut event_pump = sdl_context.event_pump()?;
    let mut animated_cars: Vec<AnimatedCar> = Vec::new();

    // Spawn initial cars
    for _ in 0..15 {
        animated_cars.push(AnimatedCar::new());
    }

    let lines: Vec<&str> = stats_text.split('\n').collect();

    // First pass: calculate the maximum label WIDTH in pixels for alignment
    let mut max_label_width = 0u32;
    for line in &lines {
        if line.contains(':') && !line.contains("Press esc") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let label = parts[0].trim();
                let label_surface = font
                    .render(label)
                    .blended(Color::RGB(255, 255, 255))
                    .map_err(|e| e.to_string())?;
                let label_texture = texture_creator
                    .create_texture_from_surface(&label_surface)
                    .map_err(|e| e.to_string())?;
                let label_query = label_texture.query();
                max_label_width = max_label_width.max(label_query.width);
            }
        }
    }

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

        // Update cars
        for car in &mut animated_cars {
            car.update();
        }

        // Remove off-screen cars and add new ones
        animated_cars.retain(|car| !car.is_off_screen());

        // Spawn new cars occasionally
        let mut rng = rand::rng();
        if rng.random_range(0..30) == 0 {
            // Roughly every 30 frames
            animated_cars.push(AnimatedCar::new());
        }

        // Clear with black background
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Draw cars that should be behind stats
        for car in &animated_cars {
            if car.should_render_behind_stats() {
                let car_texture = &car_textures[(car.car_type - 1) as usize];
                let dest_rect = Rect::new(car.x as i32, car.y as i32, 40, 70);

                canvas.copy_ex(
                    car_texture,
                    None,
                    dest_rect,
                    car.rotation,
                    None,
                    false,
                    false,
                )?;
            }
        }

        // Draw stats box background
        let stats_bg_rect = Rect::new(150, 200, 700, 600);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 200)); // Semi-transparent black
        canvas.fill_rect(stats_bg_rect)?;

        // Draw stats box border
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.draw_rect(stats_bg_rect)?;

        // Draw inner border for depth
        let inner_rect = Rect::new(155, 205, 690, 590);
        canvas.draw_rect(inner_rect)?;

        // Draw stats text with enhanced styling and left-aligned padding
        let mut y = 240;

        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                y += 15;
                continue;
            }

            // Special handling for title (first line)
            if i == 0 {
                let surface = title_font
                    .render(line)
                    .blended(Color::RGB(0, 191, 255)) // Deep sky blue title
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
                y += query.height as i32 + 8;
                continue;
            }

            // Special handling for the final instruction (press esc button to quit)
            if line.contains("Press esc button to quit") {
                // Split the line to make "esc" glow
                let parts: Vec<&str> = line.split("esc").collect();

                let mut current_x = 200; // Start position for centered text

                // Render "Press " part
                let surface1 = font
                    .render(parts[0])
                    .blended(Color::RGB(0, 255, 255)) // Cyan
                    .map_err(|e| e.to_string())?;

                let texture1 = texture_creator
                    .create_texture_from_surface(&surface1)
                    .map_err(|e| e.to_string())?;

                let query1 = texture1.query();
                let rect1 = Rect::new(current_x, y, query1.width, query1.height);
                canvas.copy(&texture1, None, rect1)?;
                current_x += query1.width as i32;

                // Render "esc" part with glow effect (multiple renders with slight offsets)
                let esc_text = "esc";

                // Glow effect - render multiple times with slight offsets
                for offset_x in [-1, 0, 1] {
                    for offset_y in [-1, 0, 1] {
                        let glow_surface = font
                            .render(esc_text)
                            .blended(Color::RGB(255, 100, 0)) // Bright orange glow
                            .map_err(|e| e.to_string())?;

                        let glow_texture = texture_creator
                            .create_texture_from_surface(&glow_surface)
                            .map_err(|e| e.to_string())?;

                        let glow_query = glow_texture.query();
                        let glow_rect = Rect::new(
                            current_x + offset_x,
                            y + offset_y,
                            glow_query.width,
                            glow_query.height,
                        );
                        canvas.copy(&glow_texture, None, glow_rect)?;
                    }
                }

                // Main "esc" text on top
                let esc_surface = font
                    .render(esc_text)
                    .blended(Color::RGB(255, 255, 255)) // White center
                    .map_err(|e| e.to_string())?;

                let esc_texture = texture_creator
                    .create_texture_from_surface(&esc_surface)
                    .map_err(|e| e.to_string())?;

                let esc_query = esc_texture.query();
                let esc_rect = Rect::new(current_x, y, esc_query.width, esc_query.height);
                canvas.copy(&esc_texture, None, esc_rect)?;
                current_x += esc_query.width as i32;

                // Render " button to quit" part
                let surface3 = font
                    .render(" button to quit")
                    .blended(Color::RGB(0, 255, 255)) // Cyan
                    .map_err(|e| e.to_string())?;

                let texture3 = texture_creator
                    .create_texture_from_surface(&surface3)
                    .map_err(|e| e.to_string())?;

                let query3 = texture3.query();
                let rect3 = Rect::new(current_x, y, query3.width, query3.height);
                canvas.copy(&texture3, None, rect3)?;

                y += esc_query.height as i32 + 8;
                continue;
            }

            // Handle lines with colons (stats data) with pixel-perfect alignment
            if line.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let label = parts[0].trim();
                    let value = parts[1].trim();

                    // Render label to get its actual width
                    let label_surface = font
                        .render(label)
                        .blended(Color::RGB(255, 255, 255)) // White labels
                        .map_err(|e| e.to_string())?;

                    let label_texture = texture_creator
                        .create_texture_from_surface(&label_surface)
                        .map_err(|e| e.to_string())?;

                    let label_query = label_texture.query();

                    // Render colon and value
                    let colon_surface = font
                        .render(": ")
                        .blended(Color::RGB(255, 255, 255)) // White colon
                        .map_err(|e| e.to_string())?;

                    let value_surface = font
                        .render(value)
                        .blended(Color::RGB(255, 255, 0)) // Yellow values
                        .map_err(|e| e.to_string())?;

                    let colon_texture = texture_creator
                        .create_texture_from_surface(&colon_surface)
                        .map_err(|e| e.to_string())?;

                    let value_texture = texture_creator
                        .create_texture_from_surface(&value_surface)
                        .map_err(|e| e.to_string())?;

                    let colon_query = colon_texture.query();
                    let value_query = value_texture.query();

                    // Position everything with pixel-perfect alignment
                    let fixed_start_x = 200; // Fixed left margin

                    // Render label at fixed position
                    let label_rect =
                        Rect::new(fixed_start_x, y, label_query.width, label_query.height);
                    canvas.copy(&label_texture, None, label_rect)?;

                    // Render colon at the SAME position for all lines (based on max_label_width)
                    let colon_x = fixed_start_x + max_label_width as i32;
                    let colon_rect = Rect::new(colon_x, y, colon_query.width, colon_query.height);
                    canvas.copy(&colon_texture, None, colon_rect)?;

                    // Render value immediately after colon
                    let value_rect = Rect::new(
                        colon_x + colon_query.width as i32,
                        y,
                        value_query.width,
                        value_query.height,
                    );
                    canvas.copy(&value_texture, None, value_rect)?;

                    y += label_query.height as i32 + 8;
                    continue;
                }
            }

            // Default rendering for any other lines
            let surface = font
                .render(line)
                .blended(Color::RGB(255, 255, 255)) // White text
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
            y += query.height as i32 + 8;
        }

        // Draw cars that should be in front of stats
        for car in &animated_cars {
            if !car.should_render_behind_stats() {
                let car_texture = &car_textures[(car.car_type - 1) as usize];
                let dest_rect = Rect::new(car.x as i32, car.y as i32, 40, 70);

                canvas.copy_ex(
                    car_texture,
                    None,
                    dest_rect,
                    car.rotation,
                    None,
                    false,
                    false,
                )?;
            }
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    Ok(())
}
