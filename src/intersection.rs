use crate::vehicle::Vehicle;
use crate::velocities::Velocity;
use crate::route::{Direction, Route, get_random_direction, get_random_route, get_spawn_position, get_turn_position};
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

/// Intersection geometry on your 1000x1000 canvas:
/// 300x300 centered square => [350,650] x [350,650]
const IX_MIN: f32 = 350.0;
const IY_MIN: f32 = 350.0;
const IX_MAX: f32 = 650.0;
const IY_MAX: f32 = 650.0;

#[derive(Clone)]
struct TimeSlot {
    start: f32,        // when the car enters this cell
    end: f32,          // when the car leaves this cell
    vehicle_id: usize, // index in active_vehicles
}

#[derive(Clone)]
struct Cell {
    slots: Vec<TimeSlot>, // reservations in chronological order
}

pub struct SmartIntersection<'a> {
    pub active_vehicles: Vec<Vehicle<'a>>,

    // --- reservation grid ---
    zone_px: u32, // e.g., 30 => 10x10 grid
    cols: usize,  // 300/zone_px
    rows: usize,
    grid: Vec<Cell>, // flattened rows*cols

    // --- stats ---
    pub total_vehicles_passed: u32,
    pub max_velocity_recorded: f32,
    pub min_velocity_recorded: f32,
    pub max_time_in_intersection: f32,
    pub min_time_in_intersection: f32,
    pub close_calls: u32,
    pub is_running: bool,

    // --- tracking data ---
    vehicle_intersection_times: std::collections::HashMap<usize, f32>, // vehicle_id -> entry_time
}

impl<'a> SmartIntersection<'a> {
    pub fn new() -> Self {
        let zone_px = 15;
        let cols = (300 / zone_px) as usize;
        let rows = cols;
        Self {
            active_vehicles: Vec::new(),
            zone_px,
            cols,
            rows,
            grid: vec![Cell { slots: Vec::new() }; cols * rows],
            total_vehicles_passed: 0,
            max_velocity_recorded: 0.0,
            min_velocity_recorded: f32::MAX,
            max_time_in_intersection: 0.0,
            min_time_in_intersection: f32::MAX,
            close_calls: 0,
            is_running: true,
            vehicle_intersection_times: std::collections::HashMap::new(),
        }
    }

    /// Main update function - handles all vehicle management
    pub fn update(&mut self, current_time: f32) {
        // Update all vehicles with smart intersection management
        self.update_vehicles_with_intersection(current_time);

        // Remove vehicles that have left the canvas and update stats
        let mut vehicles_to_remove = Vec::new();
        for (i, vehicle) in self.active_vehicles.iter().enumerate() {
            if vehicle.is_outside_canvas() {
                vehicles_to_remove.push((i, vehicle.id, vehicle.current_speed));
            }
        }

        // Remove vehicles in reverse order to maintain indices and update stats
        for &(i, vehicle_id, current_speed) in vehicles_to_remove.iter().rev() {
            self.active_vehicles.remove(i);
            self.update_stats_for_exiting_vehicle_by_data(vehicle_id, current_speed, current_time);
        }

        // Track intersection entry/exit times
        self.track_intersection_times(current_time);
    }

    /// Spawn a new vehicle if safe
    pub fn spawn_vehicle(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        direction: Option<Direction>,
    ) {
        let dir = match direction {
            Some(d) => d,
            None => get_random_direction(),
        };

        let route = get_random_route();
        let spawn_pos = get_spawn_position(dir, route);
        let turn_pos = get_turn_position(dir, route);

        if self.is_safe_to_spawn(dir, route, spawn_pos) {
            match Vehicle::new(texture_creator, route, dir, spawn_pos, turn_pos) {
                Ok(vehicle) => {
                    self.active_vehicles.push(vehicle);
                }
                Err(e) => println!("Failed to create vehicle: {}", e),
            }
        }
    }

    /// Check if it's safe to spawn a vehicle
    fn is_safe_to_spawn(&self, direction: Direction, route: Route, spawn_pos: (f32, f32)) -> bool {
        // Create temporary visual bounds for spawn vehicle
        let temp_rotation = match direction {
            Direction::North => 0.0,
            Direction::South => 180.0,
            Direction::East => 90.0,
            Direction::West => 270.0,
        };

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

        for vehicle in self.active_vehicles
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

    /// Track intersection entry and exit times
    fn track_intersection_times(&mut self, current_time: f32) {
        let mut to_remove = Vec::new();
        
        for vehicle in &self.active_vehicles {
            let vehicle_id = vehicle.id;
            
            if vehicle.is_in_intersection() {
                // Vehicle entered intersection
                if !self.vehicle_intersection_times.contains_key(&vehicle_id) {
                    self.vehicle_intersection_times.insert(vehicle_id, current_time);
                }
            } else if self.vehicle_intersection_times.contains_key(&vehicle_id) {
                // Vehicle exited intersection
                let entry_time = self.vehicle_intersection_times[&vehicle_id];
                let time_in_intersection = current_time - entry_time;
                
                // Update stats
                if time_in_intersection > self.max_time_in_intersection {
                    self.max_time_in_intersection = time_in_intersection;
                }
                if time_in_intersection < self.min_time_in_intersection {
                    self.min_time_in_intersection = time_in_intersection;
                }
                
                to_remove.push(vehicle_id);
                println!("Vehicle {} exited intersection after {:.2} seconds", vehicle_id, time_in_intersection);
            }
        }
        
        // Remove vehicles that exited intersection from tracking
        for id in to_remove {
            self.vehicle_intersection_times.remove(&id);
        }
    }

    /// Update stats when a vehicle exits the simulation (using data instead of reference)
    fn update_stats_for_exiting_vehicle_by_data(&mut self, vehicle_id: usize, current_speed: Velocity, _current_time: f32) {
        self.total_vehicles_passed += 1;
        
        // Update velocity stats
        let vehicle_max_speed = match current_speed {
            Velocity::Slow => 3.0,
            Velocity::Medium => 5.0,
            Velocity::Fast => 7.0,
        };
        
        if vehicle_max_speed > self.max_velocity_recorded {
            self.max_velocity_recorded = vehicle_max_speed;
        }
        if vehicle_max_speed < self.min_velocity_recorded {
            self.min_velocity_recorded = vehicle_max_speed;
        }
        
        // Clean up any remaining intersection time tracking
        self.vehicle_intersection_times.remove(&vehicle_id);
        
        println!("Vehicle {} completed journey. Total vehicles passed: {}", 
                 vehicle_id, self.total_vehicles_passed);
    }

    /// Update stats when a vehicle exits the simulation
    fn update_stats_for_exiting_vehicle(&mut self, vehicle: &Vehicle, _current_time: f32) {
        self.update_stats_for_exiting_vehicle_by_data(vehicle.id, vehicle.current_speed, _current_time);
    }

    /// Print final statistics
    pub fn print_final_stats(&self) {
        println!("\n=== SMART INTERSECTION FINAL STATISTICS ===");
        println!("Total vehicles passed: {}", self.total_vehicles_passed);
        println!("Max velocity recorded: {:.1} pixels/frame", self.max_velocity_recorded);
        println!("Min velocity recorded: {:.1} pixels/frame", 
                 if self.min_velocity_recorded == f32::MAX { 0.0 } else { self.min_velocity_recorded });
        println!("Max time in intersection: {:.2} seconds", self.max_time_in_intersection);
        println!("Min time in intersection: {:.2} seconds", 
                 if self.min_time_in_intersection == f32::MAX { 0.0 } else { self.min_time_in_intersection });
        println!("Close calls detected: {}", self.close_calls);
        println!("Active vehicles remaining: {}", self.active_vehicles.len());
        println!("==========================================\n");
    }

    /// Remove a specific vehicle by ID
    pub fn remove_vehicle(&mut self, vehicle_id: usize) -> Option<Vehicle<'a>> {
        if let Some(pos) = self.active_vehicles.iter().position(|v| v.id == vehicle_id) {
            Some(self.active_vehicles.remove(pos))
        } else {
            None
        }
    }

    /// Try to reserve cells along the path for this vehicle
    pub fn request_cells(
        &mut self,
        vehicle_id: usize,
        cells: &[(usize, usize)], // list of (col,row) coordinates
        entry_time: f32,
        exit_time: f32,
    ) -> bool {
        // First check if all requested cells are free
        for &(col, row) in cells {
            if col >= self.cols || row >= self.rows {
                continue; // Skip out of bounds cells
            }
            let idx = self.cell_index(col, row);
            if self.conflict(&self.grid[idx], entry_time, exit_time) {
                return false; // conflict found, reject request
            }
        }

        // No conflicts → reserve them
        for &(col, row) in cells {
            if col >= self.cols || row >= self.rows {
                continue; // Skip out of bounds cells
            }
            let idx = self.cell_index(col, row);
            self.grid[idx].slots.push(TimeSlot {
                start: entry_time,
                end: exit_time,
                vehicle_id,
            });
        }

        true
    }

    /// Release specific cells that a vehicle has passed through
    pub fn release_specific_cells(&mut self, cells: &[(usize, usize)], vehicle_id: usize) {
        for &(col, row) in cells {
            if col >= self.cols || row >= self.rows {
                continue;
            }
            let idx = self.cell_index(col, row);
            // Remove only the reservations made by this specific vehicle
            self.grid[idx]
                .slots
                .retain(|slot| slot.vehicle_id != vehicle_id);
        }
    }

    /// Update all vehicles with traffic and intersection management
    fn update_vehicles_with_intersection(&mut self, current_time: f32) {
        // Calculate traffic speeds for all vehicles first
        let mut target_speeds = Vec::with_capacity(self.active_vehicles.len());

        for i in 0..self.active_vehicles.len() {
            let current_vehicle = &self.active_vehicles[i];

            // If vehicle is past intersection, it can go fast (no collision risk)
            if current_vehicle.is_past_intersection() {
                target_speeds.push(Velocity::Fast);
                continue;
            }

            let mut target_speed = Velocity::Fast;
            let mut closest_distance = f32::MAX;
            let mut required_distance = 0.0;

            // Check traffic by manually iterating through other vehicles
            for (j, other_vehicle) in self.active_vehicles.iter().enumerate() {
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

        // Collect all vehicle data needed for intersection management to avoid borrowing conflicts
        let mut vehicle_updates = Vec::new();
        
        for i in 0..self.active_vehicles.len() {
            let vehicle = &self.active_vehicles[i];
            let vehicle_id = vehicle.id;
            let distance_to_intersection = vehicle.distance_to_intersection();
            let is_past_intersection = vehicle.is_past_intersection();
            let is_in_intersection = vehicle.is_in_intersection();
            let mut requested_intersection = vehicle.requested_intersection;
            let mut intersection_permission = vehicle.intersection_permission;
            let vehicle_route = vehicle.route;
            let vehicle_direction = vehicle.direction;
            let (vx, vy, vw, vh) = vehicle.get_visual_bounds();
            let traffic_speed = target_speeds[i];

            // Reset intersection status if vehicle is far away
            if distance_to_intersection > 150.0 {
                requested_intersection = false;
                intersection_permission = false;
            }

            // Calculate intersection speed
            let intersection_speed = if is_past_intersection {
                Velocity::Fast
            } else if distance_to_intersection > 120.0 {
                Velocity::Fast
            } else if is_in_intersection {
                Velocity::Fast
            } else if !requested_intersection {
                // Need to request intersection permission
                let permission = self.try_intersection_request_by_data(
                    vehicle_id, vehicle_route, vehicle_direction, 
                    current_time, distance_to_intersection
                );
                requested_intersection = true;
                intersection_permission = permission;
                
                if permission {
                    Velocity::Fast
                } else {
                    if distance_to_intersection < 25.0 {
                        Velocity::Slow
                    } else if distance_to_intersection < 60.0 {
                        Velocity::Medium
                    } else {
                        Velocity::Medium
                    }
                }
            } else {
                // Already requested, use existing permission
                if intersection_permission {
                    Velocity::Fast
                } else {
                    if distance_to_intersection < 25.0 {
                        Velocity::Slow
                    } else if distance_to_intersection < 60.0 {
                        Velocity::Medium
                    } else {
                        Velocity::Medium
                    }
                }
            };

            // Calculate final speed
            let final_speed = if is_past_intersection {
                Velocity::Fast
            } else {
                match (traffic_speed, intersection_speed) {
                    (Velocity::Slow, _) | (_, Velocity::Slow) => Velocity::Slow,
                    (Velocity::Medium, _) | (_, Velocity::Medium) => Velocity::Medium,
                    (Velocity::Fast, Velocity::Fast) => Velocity::Fast,
                }
            };

            // Calculate cells to release if needed
            let cells_to_release = if is_in_intersection || distance_to_intersection < 50.0 {
                let zone_px = self.zone_px as f32;
                self.calculate_cells_to_release(vehicle_direction, vx, vy, vw, vh, zone_px)
            } else {
                Vec::new()
            };

            vehicle_updates.push((i, final_speed, requested_intersection, 
                                intersection_permission, cells_to_release, vehicle_id));
        }

        // Now apply all updates without borrowing conflicts
        for (i, final_speed, requested_intersection, intersection_permission, cells_to_release, vehicle_id) in vehicle_updates {
            let vehicle = &mut self.active_vehicles[i];
            
            // Apply the speed
            vehicle.current_speed = final_speed;
            
            // Update intersection status
            vehicle.requested_intersection = requested_intersection;
            vehicle.intersection_permission = intersection_permission;

            // Update vehicle position
            vehicle.update();

            // Release cells if needed
            if !cells_to_release.is_empty() {
                self.release_specific_cells(&cells_to_release, vehicle_id);
            }

            // Detect close calls
            self.detect_close_calls(i);
        }
    }

    /// Detect close calls between vehicles
    fn detect_close_calls(&mut self, vehicle_index: usize) {
        let current_vehicle = &self.active_vehicles[vehicle_index];
        
        for (j, other_vehicle) in self.active_vehicles.iter().enumerate() {
            if vehicle_index == j {
                continue;
            }
            
            // Check if vehicles are very close
            let distance = current_vehicle.distance_to_vehicle(other_vehicle);
            let min_safe_distance = 30.0; // Very close threshold
            
            if distance < min_safe_distance && 
               (current_vehicle.is_in_intersection() || other_vehicle.is_in_intersection()) {
                self.close_calls += 1;
                println!("Close call detected between vehicles {} and {} at distance {:.1}", 
                        current_vehicle.id, other_vehicle.id, distance);
            }
        }
    }

    /// Calculate cells to release behind a moving vehicle
    fn calculate_cells_to_release(
        &self,
        direction: Direction,
        vx: f32,
        vy: f32,
        vw: f32,
        vh: f32,
        zone_px: f32,
    ) -> Vec<(usize, usize)> {
        match direction {
            Direction::North => {
                // Release cells below visual bounds
                let behind_y = vy + vh + zone_px;
                if behind_y >= IX_MIN && behind_y < IX_MAX {
                    let left_col = ((vx - IX_MIN) / zone_px) as usize;
                    let right_col = ((vx + vw - IX_MIN) / zone_px) as usize;
                    let row = ((behind_y - IY_MIN) / zone_px) as usize;

                    let mut cells = Vec::new();
                    for col in left_col..=right_col.min(self.cols - 1) {
                        if col < self.cols && row < self.rows {
                            cells.push((col, row));
                        }
                    }
                    cells
                } else {
                    vec![]
                }
            }
            Direction::South => {
                // Release cells above visual bounds
                let behind_y = vy - zone_px;
                if behind_y >= IY_MIN {
                    let left_col = ((vx - IX_MIN) / zone_px) as usize;
                    let right_col = ((vx + vw - IX_MIN) / zone_px) as usize;
                    let row = ((behind_y - IY_MIN) / zone_px) as usize;

                    let mut cells = Vec::new();
                    for col in left_col..=right_col.min(self.cols - 1) {
                        if col < self.cols && row < self.rows {
                            cells.push((col, row));
                        }
                    }
                    cells
                } else {
                    vec![]
                }
            }
            Direction::East => {
                // Release cells to the left of visual bounds
                let behind_x = vx - zone_px;
                if behind_x >= IX_MIN {
                    let top_row = ((vy - IY_MIN) / zone_px) as usize;
                    let bottom_row = ((vy + vh - IY_MIN) / zone_px) as usize;
                    let col = ((behind_x - IX_MIN) / zone_px) as usize;

                    let mut cells = Vec::new();
                    for row in top_row..=bottom_row.min(self.rows - 1) {
                        if col < self.cols && row < self.rows {
                            cells.push((col, row));
                        }
                    }
                    cells
                } else {
                    vec![]
                }
            }
            Direction::West => {
                // Release cells to the right of visual bounds
                let behind_x = vx + vw + zone_px;
                if behind_x < IX_MAX {
                    let top_row = ((vy - IY_MIN) / zone_px) as usize;
                    let bottom_row = ((vy + vh - IY_MIN) / zone_px) as usize;
                    let col = ((behind_x - IX_MIN) / zone_px) as usize;

                    let mut cells = Vec::new();
                    for row in top_row..=bottom_row.min(self.rows - 1) {
                        if col < self.cols && row < self.rows {
                            cells.push((col, row));
                        }
                    }
                    cells
                } else {
                    vec![]
                }
            }
        }
    }

    /// Try to get intersection permission using vehicle data instead of reference
    fn try_intersection_request_by_data(
        &mut self,
        vehicle_id: usize,
        route: Route,
        direction: Direction,
        current_time: f32,
        distance_to_intersection: f32,
    ) -> bool {
        let time_to_intersection =
            self.calculate_time_to_intersection_by_speed(distance_to_intersection);
        let crossing_time = self.calculate_crossing_time_by_direction(direction);

        let entry_time = current_time + time_to_intersection;
        let exit_time = entry_time + crossing_time;

        let required_cells = self.calculate_vehicle_path_cells_by_data(route, direction);

        self.request_cells(vehicle_id, &required_cells, entry_time, exit_time)
    }

    /// Calculate time for vehicle to reach intersection using default speed assumptions
    fn calculate_time_to_intersection_by_speed(&self, distance: f32) -> f32 {
        let speed_pixels_per_frame = 5.0; // Use medium speed as default assumption

        if speed_pixels_per_frame == 0.0 {
            return f32::MAX;
        }

        // Convert frames to seconds (assuming 60 FPS)
        distance / speed_pixels_per_frame / 60.0
    }

    /// Calculate time for vehicle to cross intersection completely using direction
    fn calculate_crossing_time_by_direction(&self, direction: Direction) -> f32 {
        // Intersection crossing distance plus vehicle length to clear completely
        let crossing_distance = match direction {
            Direction::North | Direction::South => {
                300.0 + 70.0 // height
            }
            Direction::East | Direction::West => {
                300.0 + 40.0 // width
            }
        };

        let speed_pixels_per_frame = 7.0; // Assume fast speed when crossing
        crossing_distance / speed_pixels_per_frame / 60.0
    }

    /// Calculate which cells vehicle will need during crossing using data
    fn calculate_vehicle_path_cells_by_data(&self, route: Route, direction: Direction) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        let zone_px = self.zone_px as f32;

        match route {
            Route::Straight => {
                self.get_straight_path_cells_by_direction(direction, zone_px, &mut cells);
            }
            Route::Right => {
                self.get_right_turn_path_cells_by_direction(direction, zone_px, &mut cells);
            }
            Route::Left => {
                self.get_left_turn_path_cells_by_direction(&mut cells);
            }
        }

        cells
    }

    fn get_straight_path_cells_by_direction(
        &self,
        direction: Direction,
        zone_px: f32,
        cells: &mut Vec<(usize, usize)>,
    ) {
        // Use approximate vehicle positioning based on direction and route
        match direction {
            Direction::North | Direction::South => {
                // Reserve entire columns for straight paths
                let start_col = 0;
                let end_col = self.cols;

                for col in start_col..end_col {
                    for row in 0..self.rows {
                        cells.push((col, row));
                    }
                }
            }
            Direction::East | Direction::West => {
                // Reserve entire rows for straight paths
                let start_row = 0;
                let end_row = self.rows;

                for row in start_row..end_row {
                    for col in 0..self.cols {
                        cells.push((col, row));
                    }
                }
            }
        }
    }

    fn get_right_turn_path_cells_by_direction(
        &self,
        direction: Direction,
        _zone_px: f32,
        cells: &mut Vec<(usize, usize)>,
    ) {
        // For right turns, reserve a broader area to be safe
        match direction {
            Direction::North => {
                // Vehicle turning from south to east
                for row in 0..self.rows {
                    for col in (self.cols / 2)..self.cols {
                        cells.push((col, row));
                    }
                }
            }
            Direction::South => {
                // Vehicle turning from north to west
                for row in 0..self.rows {
                    for col in 0..(self.cols / 2) {
                        cells.push((col, row));
                    }
                }
            }
            Direction::East => {
                // Vehicle turning from west to south
                for row in (self.rows / 2)..self.rows {
                    for col in 0..self.cols {
                        cells.push((col, row));
                    }
                }
            }
            Direction::West => {
                // Vehicle turning from east to north
                for row in 0..(self.rows / 2) {
                    for col in 0..self.cols {
                        cells.push((col, row));
                    }
                }
            }
        }
    }

    fn get_left_turn_path_cells_by_direction(
        &self,
        cells: &mut Vec<(usize, usize)>,
    ) {
        // Left turns need the entire intersection
        for row in 0..self.rows {
            for col in 0..self.cols {
                cells.push((col, row));
            }
        }
    }

    /// Try to get intersection permission
    fn try_intersection_request(
        &mut self,
        vehicle: &Vehicle,
        current_time: f32,
        distance_to_intersection: f32,
    ) -> bool {
        let time_to_intersection =
            self.calculate_time_to_intersection(vehicle, distance_to_intersection);
        let crossing_time = self.calculate_crossing_time(vehicle);

        let entry_time = current_time + time_to_intersection;
        let exit_time = entry_time + crossing_time;

        let required_cells = self.calculate_vehicle_path_cells(vehicle);

        self.request_cells(vehicle.id, &required_cells, entry_time, exit_time)
    }

    /// Calculate time for vehicle to reach intersection
    fn calculate_time_to_intersection(&self, vehicle: &Vehicle, distance: f32) -> f32 {
        let speed_pixels_per_frame = match vehicle.current_speed {
            Velocity::Slow => 3.0,
            Velocity::Medium => 5.0,
            Velocity::Fast => 7.0,
        };

        if speed_pixels_per_frame == 0.0 {
            return f32::MAX;
        }

        // Convert frames to seconds (assuming 60 FPS)
        distance / speed_pixels_per_frame / 60.0
    }

    /// Calculate time for vehicle to cross intersection completely
    fn calculate_crossing_time(&self, vehicle: &Vehicle) -> f32 {
        // Intersection crossing distance plus vehicle length to clear completely
        let crossing_distance = match vehicle.direction {
            Direction::North | Direction::South => {
                300.0 + vehicle.height as f32
            }
            Direction::East | Direction::West => {
                300.0 + vehicle.width as f32
            }
        };

        let speed_pixels_per_frame = 7.0; // Assume fast speed when crossing
        crossing_distance / speed_pixels_per_frame / 60.0
    }

    /// Calculate which cells vehicle will need during crossing
    fn calculate_vehicle_path_cells(&self, vehicle: &Vehicle) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        let zone_px = self.zone_px as f32;

        match vehicle.route {
            Route::Straight => {
                self.get_straight_path_cells(vehicle, zone_px, &mut cells);
            }
            Route::Right => {
                self.get_right_turn_path_cells(vehicle, zone_px, &mut cells);
            }
            Route::Left => {
                self.get_left_turn_path_cells(vehicle, zone_px, &mut cells);
            }
        }

        cells
    }

    fn get_straight_path_cells(
        &self,
        vehicle: &Vehicle,
        zone_px: f32,
        cells: &mut Vec<(usize, usize)>,
    ) {
        let (vx, vy, vw, vh) = vehicle.get_visual_bounds();

        match vehicle.direction {
            Direction::North | Direction::South => {
                // Reserve column(s) that the visual bounds occupy
                let left_col = ((vx - IX_MIN) / zone_px) as usize;
                let right_col = ((vx + vw - IX_MIN) / zone_px) as usize;

                for col in left_col..=right_col.min(self.cols - 1) {
                    if col < self.cols {
                        for row in 0..self.rows {
                            cells.push((col, row));
                        }
                    }
                }
            }
            Direction::East | Direction::West => {
                // Reserve row(s) that the visual bounds occupy
                let top_row = ((vy - IY_MIN) / zone_px) as usize;
                let bottom_row = ((vy + vh - IY_MIN) / zone_px) as usize;

                for row in top_row..=bottom_row.min(self.rows - 1) {
                    if row < self.rows {
                        for col in 0..self.cols {
                            cells.push((col, row));
                        }
                    }
                }
            }
        }
    }

    fn get_right_turn_path_cells(
        &self,
        vehicle: &Vehicle,
        zone_px: f32,
        cells: &mut Vec<(usize, usize)>,
    ) {
        let (vx, vy, vw, vh) = vehicle.get_visual_bounds();

        match vehicle.direction {
            Direction::North => {
                // Vehicle visual bounds occupy certain cells
                let start_col = ((vx - IX_MIN) / zone_px) as usize;
                let end_col = self.cols;
                let start_row = ((vy - IY_MIN) / zone_px) as usize;
                let end_row = ((vy + vh - IY_MIN) / zone_px) as usize + 5; // Extra for turn

                for row in start_row..end_row.min(self.rows) {
                    for col in start_col..end_col.min(self.cols) {
                        cells.push((col, row));
                    }
                }
            }
            Direction::South => {
                let end_col = ((vx + vw - IX_MIN) / zone_px) as usize + 1;
                let start_row = ((vy - IX_MIN) / zone_px) as usize;
                let end_row = ((vy + vh - IY_MIN) / zone_px) as usize + 5;

                for row in start_row..end_row.min(self.rows) {
                    for col in 0..end_col.min(self.cols) {
                        cells.push((col, row));
                    }
                }
            }
            Direction::East => {
                let start_col = ((vx - IX_MIN) / zone_px) as usize;
                let end_col = ((vx + vw - IX_MIN) / zone_px) as usize + 5;
                let start_row = ((vy - IY_MIN) / zone_px) as usize;
                let end_row = self.rows;

                for row in start_row..end_row.min(self.rows) {
                    for col in start_col..end_col.min(self.cols) {
                        cells.push((col, row));
                    }
                }
            }
            Direction::West => {
                let start_col = ((vx - IX_MIN) / zone_px) as usize;
                let end_col = ((vx + vw - IX_MIN) / zone_px) as usize + 5;
                let end_row = ((vy + vh - IY_MIN) / zone_px) as usize + 1;

                for row in 0..end_row.min(self.rows) {
                    for col in start_col..end_col.min(self.cols) {
                        cells.push((col, row));
                    }
                }
            }
        }
    }

    fn get_left_turn_path_cells(
        &self,
        _vehicle: &Vehicle,
        _zone_px: f32,
        cells: &mut Vec<(usize, usize)>,
    ) {
        for row in 0..self.rows {
            for col in 0..self.cols {
                cells.push((col, row));
            }
        }
    }

    /// Reset vehicle's intersection request when it's far enough away (kept for compatibility)
    pub fn reset_vehicle_intersection_status(&mut self, vehicle: &mut Vehicle) {
        if vehicle.distance_to_intersection() > 150.0 {
            vehicle.requested_intersection = false;
            vehicle.intersection_permission = false;
        }
    }

    /// Check if a cell has a conflicting reservation
    fn conflict(&self, cell: &Cell, start: f32, end: f32) -> bool {
        cell.slots
            .iter()
            .any(|slot| start < slot.end && slot.start < end)
    }

    /// Utility: convert (col,row) to index
    fn cell_index(&self, col: usize, row: usize) -> usize {
        row * self.cols + col
    }
}

/// Helper function to release cells behind a moving vehicle (kept for compatibility)
pub fn release_cells_behind_vehicle(intersection: &mut SmartIntersection, vehicle: &Vehicle) {
    let zone_px = intersection.zone_px as f32;
    let (vx, vy, vw, vh) = vehicle.get_visual_bounds();

    let cells_to_release = intersection.calculate_cells_to_release(
        vehicle.direction, vx, vy, vw, vh, zone_px
    );

    if !cells_to_release.is_empty() {
        intersection.release_specific_cells(&cells_to_release, vehicle.id);
    }
}