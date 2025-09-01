use crate::route::{
    Direction, Route, get_random_direction, get_random_route, get_spawn_position, get_turn_position,
};
use crate::vehicle::Vehicle;
use crate::velocities::Velocity;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use std::collections::HashMap;

/// Intersection geometry
const IX_MIN: f32 = 350.0;
const IY_MIN: f32 = 350.0;
const IX_MAX: f32 = 650.0;
const IY_MAX: f32 = 650.0;

#[derive(Clone)]
struct TimeSlot {
    start: f32,
    end: f32,
    vehicle_id: usize,
}

#[derive(Clone)]
struct Cell {
    slots: Vec<TimeSlot>,
}

/// Memoized path data for each direction+route combination
#[derive(Clone, Debug)]
struct PathSegment {
    cells: Vec<(usize, usize)>,
    distance: f32,
}

#[derive(Clone, Debug)]
struct VehiclePath {
    segment1: PathSegment, // Entry to turn position (or full path for straight)
    segment2: Option<PathSegment>, // Turn position to exit (None for straight)
    turn_position: Option<(f32, f32)>, // Where the vehicle turns (None for straight)
}

type PathCache = HashMap<(Direction, Route), VehiclePath>;

pub struct SmartIntersection<'a> {
    pub active_vehicles: Vec<Vehicle<'a>>,

    // --- reservation grid ---
    zone_px: u32, // e.g., 30 => 10x10 grid
    cols: usize,  // 300/zone_px
    rows: usize,
    grid: Vec<Cell>, // flattened rows*cols

    // Memoized path calculations
    path_cache: PathCache,

    // Stats
    pub total_vehicles_passed: u32,
    pub max_velocity_recorded: f32,
    pub min_velocity_recorded: f32,
    pub max_time_in_intersection: f32,
    pub min_time_in_intersection: f32,
    pub close_calls: u32,
    pub is_running: bool,
    pub close_call_pairs_this_frame: std::collections::HashSet<(usize, usize)>,

    vehicle_intersection_times: HashMap<usize, f32>,
}

impl<'a> SmartIntersection<'a> {
    pub fn new() -> Self {
        let zone_px = 10;
        let cols = 300 / zone_px;
        let rows = cols;

        let mut intersection = Self {
            active_vehicles: Vec::new(),
            zone_px: zone_px as u32,
            cols,
            rows,
            grid: vec![Cell { slots: Vec::new() }; cols * rows],
            path_cache: HashMap::new(),
            total_vehicles_passed: 0,
            max_velocity_recorded: 0.0,
            min_velocity_recorded: f32::MAX,
            max_time_in_intersection: 0.0,
            min_time_in_intersection: f32::MAX,
            close_calls: 0,
            is_running: true,
            close_call_pairs_this_frame: std::collections::HashSet::new(),
            vehicle_intersection_times: HashMap::new(),
        };

        // Pre-calculate all possible paths
        intersection.initialize_path_cache();
        intersection
    }

    /// Pre-calculate all possible vehicle paths for memoization
    fn initialize_path_cache(&mut self) {
        let directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        let routes = [Route::Straight, Route::Left, Route::Right];

        for &direction in &directions {
            for &route in &routes {
                let path = self.calculate_vehicle_path(direction, route);

                println!(
                    "Cached path for {:?} {:?}: Segment1={} cells, Segment2={} cells",
                    direction,
                    route,
                    path.segment1.cells.len(),
                    path.segment2.as_ref().map_or(0, |s| s.cells.len())
                );

                self.path_cache.insert((direction, route), path); // This now works because path implements Clone
            }
        }
    }

    /// Calculate the complete path for a vehicle (called during initialization)
    fn calculate_vehicle_path(&self, direction: Direction, route: Route) -> VehiclePath {
        match route {
            Route::Straight => {
                let cells = self.calculate_straight_path_cells(direction);
                let distance = self.calculate_straight_path_distance(direction);

                VehiclePath {
                    segment1: PathSegment { cells, distance },
                    segment2: None,
                    turn_position: None,
                }
            }
            Route::Right | Route::Left => {
                let turn_pos = get_turn_position(direction, route);
                let (segment1_cells, segment1_distance) =
                    self.calculate_path_to_turn(direction, route, turn_pos);
                let (segment2_cells, segment2_distance) =
                    self.calculate_path_from_turn(direction, route, turn_pos);

                VehiclePath {
                    segment1: PathSegment {
                        cells: segment1_cells,
                        distance: segment1_distance,
                    },
                    segment2: Some(PathSegment {
                        cells: segment2_cells,
                        distance: segment2_distance,
                    }),
                    turn_position: Some(turn_pos),
                }
            }
        }
    }

    /// Calculate straight path cells
    fn calculate_straight_path_cells(&self, direction: Direction) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();

        match direction {
            Direction::North => {
                // Northbound straight: lanes around x=550 (cols 20-24)
                for row in 0..self.rows {
                    for col in 20..25 {
                        if col < self.cols {
                            cells.push((col, row));
                        }
                    }
                }
            }
            Direction::South => {
                // Southbound straight: lanes around x=400 (cols 5-9)
                for row in 0..self.rows {
                    for col in 5..10 {
                        if col < self.cols {
                            cells.push((col, row));
                        }
                    }
                }
            }
            Direction::East => {
                // Eastbound straight: lanes around y=550 (rows 20-24)
                for col in 0..self.cols {
                    for row in 20..25 {
                        if row < self.rows {
                            cells.push((col, row));
                        }
                    }
                }
            }
            Direction::West => {
                // Westbound straight: lanes around y=400 (rows 5-9)
                for col in 0..self.cols {
                    for row in 5..10 {
                        if row < self.rows {
                            cells.push((col, row));
                        }
                    }
                }
            }
        }

        cells
    }

    /// Calculate distance for straight path through intersection
    fn calculate_straight_path_distance(&self, _direction: Direction) -> f32 {
        300.0 // Intersection is 300px across
    }

    /// Calculate path from entry to turn position
    fn calculate_path_to_turn(
        &self,
        direction: Direction,
        route: Route,
        turn_pos: (f32, f32),
    ) -> (Vec<(usize, usize)>, f32) {
        let mut cells = Vec::new();

        match direction {
            Direction::North => {
                let cols = if route == Route::Left { 15..20 } else { 25..30 }; // Left or right lane
                let entry_y = 650.0;
                let turn_y = turn_pos.1;
                let distance = entry_y - turn_y;

                let start_row = ((turn_y - IY_MIN) / self.zone_px as f32) as usize;
                let end_row = ((entry_y - IY_MIN) / self.zone_px as f32) as usize;

                for row in start_row..=end_row.min(self.rows - 1) {
                    for col in cols.clone() {
                        if col < self.cols {
                            cells.push((col, row));
                        }
                    }
                }

                (cells, distance)
            }
            Direction::South => {
                let cols = if route == Route::Left { 10..15 } else { 0..5 }; // Left or right lane
                let entry_y = 350.0;
                let turn_y = turn_pos.1;
                let distance = turn_y - entry_y;

                let start_row = ((entry_y - IY_MIN) / self.zone_px as f32) as usize;
                let end_row = ((turn_y - IY_MIN) / self.zone_px as f32) as usize;

                for row in start_row..=end_row.min(self.rows - 1) {
                    for col in cols.clone() {
                        if col < self.cols {
                            cells.push((col, row));
                        }
                    }
                }

                (cells, distance)
            }
            Direction::East => {
                let rows = if route == Route::Left { 15..20 } else { 25..30 }; // Left or right lane
                let entry_x = 350.0;
                let turn_x = turn_pos.0;
                let distance = turn_x - entry_x;

                let start_col = ((entry_x - IX_MIN) / self.zone_px as f32) as usize;
                let end_col = ((turn_x - IX_MIN) / self.zone_px as f32) as usize;

                for col in start_col..=end_col.min(self.cols - 1) {
                    for row in rows.clone() {
                        if row < self.rows {
                            cells.push((col, row));
                        }
                    }
                }

                (cells, distance)
            }
            Direction::West => {
                let rows = if route == Route::Left { 10..15 } else { 0..5 }; // Left or right lane
                let entry_x = 650.0;
                let turn_x = turn_pos.0;
                let distance = entry_x - turn_x;

                let start_col = ((turn_x - IX_MIN) / self.zone_px as f32) as usize;
                let end_col = ((entry_x - IX_MIN) / self.zone_px as f32) as usize;

                for col in start_col..=end_col.min(self.cols - 1) {
                    for row in rows.clone() {
                        if row < self.rows {
                            cells.push((col, row));
                        }
                    }
                }

                (cells, distance)
            }
        }
    }

    /// Calculate path from turn position to exit
    fn calculate_path_from_turn(
        &self,
        direction: Direction,
        route: Route,
        turn_pos: (f32, f32),
    ) -> (Vec<(usize, usize)>, f32) {
        let mut cells = Vec::new();

        // After turning, vehicle changes direction
        let new_direction = match (direction, route) {
            (Direction::North, Route::Right) => Direction::East,
            (Direction::North, Route::Left) => Direction::West,
            (Direction::South, Route::Right) => Direction::West,
            (Direction::South, Route::Left) => Direction::East,
            (Direction::East, Route::Right) => Direction::South,
            (Direction::East, Route::Left) => Direction::North,
            (Direction::West, Route::Right) => Direction::North,
            (Direction::West, Route::Left) => Direction::South,
            _ => direction, // Should not happen for turns
        };

        match new_direction {
            Direction::North => {
                let exit_y = 350.0;
                let turn_y = turn_pos.1;
                let distance = turn_y - exit_y;

                let cols = 15..20; // After-turn lane width
                let start_row = ((exit_y - IY_MIN) / self.zone_px as f32) as usize;
                let end_row = ((turn_y - IY_MIN) / self.zone_px as f32) as usize;

                for row in start_row..=end_row.min(self.rows - 1) {
                    for col in cols.clone() {
                        if col < self.cols {
                            cells.push((col, row));
                        }
                    }
                }

                (cells, distance)
            }
            Direction::South => {
                let exit_y = 650.0;
                let turn_y = turn_pos.1;
                let distance = exit_y - turn_y;

                let cols = 10..15; // After-turn lane width
                let start_row = ((turn_y - IY_MIN) / self.zone_px as f32) as usize;
                let end_row = ((exit_y - IY_MIN) / self.zone_px as f32) as usize;

                for row in start_row..=end_row.min(self.rows - 1) {
                    for col in cols.clone() {
                        if col < self.cols {
                            cells.push((col, row));
                        }
                    }
                }

                (cells, distance)
            }
            Direction::East => {
                let exit_x = 650.0;
                let turn_x = turn_pos.0;
                let distance = exit_x - turn_x;

                let rows = 10..15; // After-turn lane width
                let start_col = ((turn_x - IX_MIN) / self.zone_px as f32) as usize;
                let end_col = ((exit_x - IX_MIN) / self.zone_px as f32) as usize;

                for col in start_col..=end_col.min(self.cols - 1) {
                    for row in rows.clone() {
                        if row < self.rows {
                            cells.push((col, row));
                        }
                    }
                }

                (cells, distance)
            }
            Direction::West => {
                let exit_x = 350.0;
                let turn_x = turn_pos.0;
                let distance = turn_x - exit_x;

                let rows = 15..20; // After-turn lane width
                let start_col = ((exit_x - IX_MIN) / self.zone_px as f32) as usize;
                let end_col = ((turn_x - IX_MIN) / self.zone_px as f32) as usize;

                for col in start_col..=end_col.min(self.cols - 1) {
                    for row in rows.clone() {
                        if row < self.rows {
                            cells.push((col, row));
                        }
                    }
                }

                (cells, distance)
            }
        }
    }

    /// Main update function
    pub fn update(&mut self, current_time: f32) {
        self.update_vehicles_with_two_path_system(current_time);

        // Remove vehicles that left canvas
        let mut vehicles_to_remove = Vec::new();
        for (i, vehicle) in self.active_vehicles.iter().enumerate() {
            if vehicle.is_outside_canvas() {
                vehicles_to_remove.push((i, vehicle.id, vehicle.current_speed));
            }
        }

        for &(i, vehicle_id, current_speed) in vehicles_to_remove.iter().rev() {
            self.active_vehicles.remove(i);
            self.update_stats_for_exiting_vehicle_by_data(vehicle_id, current_speed, current_time);
        }

        self.track_intersection_times(current_time);
    }

    /// Updated vehicle management with two-path system
    fn update_vehicles_with_two_path_system(&mut self, current_time: f32) {
        // Calculate traffic speeds
        let mut target_speeds = Vec::with_capacity(self.active_vehicles.len());

        for i in 0..self.active_vehicles.len() {
            let current_vehicle = &self.active_vehicles[i];

            if current_vehicle.is_past_intersection() {
                target_speeds.push(Velocity::Fast);
                continue;
            }

            let mut target_speed = Velocity::Fast;
            let mut closest_distance = f32::MAX;
            let mut required_distance = 0.0;

            for (j, other_vehicle) in self.active_vehicles.iter().enumerate() {
                if i == j {
                    continue;
                }

                if current_vehicle.is_ahead_of_me(other_vehicle) {
                    let distance = current_vehicle.distance_to_vehicle(other_vehicle);
                    if distance < closest_distance {
                        closest_distance = distance;
                        required_distance =
                            current_vehicle.get_safe_following_distance(other_vehicle);
                    }
                }
            }

            if closest_distance != f32::MAX && closest_distance < required_distance {
                if closest_distance < required_distance * 0.7 {
                    target_speed = Velocity::Stopped
                } else if closest_distance < required_distance * 0.8 {
                    target_speed = Velocity::Medium;
                }
            }

            target_speeds.push(target_speed);
        }

        // Process intersection requests with two-path system
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
            let vehicle_speed = vehicle.current_speed;
            let (vx, vy, vw, vh) = vehicle.get_visual_bounds();
            let traffic_speed = target_speeds[i];

            // Reset intersection status if far away
            if distance_to_intersection > 150.0 {
                requested_intersection = false;
                intersection_permission = false;
            }

            let intersection_speed = if is_past_intersection {
                Velocity::Fast
            } else if distance_to_intersection > 60.0 {
                Velocity::Fast
            } else if is_in_intersection {
                Velocity::Fast
            } else if !requested_intersection || !intersection_permission {
                // Check if vehicle should stop at intersection entrance
                if distance_to_intersection <= 10.0 && !intersection_permission {
                    // Vehicle is at intersection entrance and was previously denied
                    // Keep trying with fast speed while stopped
                    let (permission, recommended_speed) = self.try_two_path_intersection_request(
                        vehicle_id,
                        vehicle_route,
                        vehicle_direction,
                        Velocity::Fast, // Always try with fast speed when stopped
                        current_time,
                        distance_to_intersection,
                    );
                    requested_intersection = true;
                    intersection_permission = permission;

                    if permission {
                        println!(
                            "✅ Vehicle {} got permission after waiting - resuming at fast speed",
                            vehicle_id
                        );
                        Velocity::Fast
                    } else {
                        println!(
                            "🛑 Vehicle {} still waiting at intersection entrance",
                            vehicle_id
                        );
                        Velocity::Stopped
                    }
                } else {
                    // Normal intersection request with adaptive speed
                    let (permission, recommended_speed) = self.try_two_path_intersection_request(
                        vehicle_id,
                        vehicle_route,
                        vehicle_direction,
                        vehicle_speed,
                        current_time,
                        distance_to_intersection,
                    );
                    requested_intersection = true;
                    intersection_permission = permission;

                    if !permission && distance_to_intersection <= 15.0 {
                        // Close to intersection but denied - stop the vehicle
                        println!(
                            "🛑 Vehicle {} denied permission - stopping at intersection entrance",
                            vehicle_id
                        );
                        Velocity::Stopped
                    } else {
                        recommended_speed
                    }
                }
            } else {
                Velocity::Fast
            };

            // Determine final speed
            let final_speed = if is_past_intersection {
                Velocity::Fast
            } else {
                match (traffic_speed, intersection_speed) {
                    (Velocity::Stopped, _) | (_, Velocity::Stopped) => Velocity::Stopped, // NEW: Stop overrides everything
                    (Velocity::Slow, _) | (_, Velocity::Slow) => Velocity::Slow,
                    (Velocity::Medium, _) | (_, Velocity::Medium) => Velocity::Medium,
                    (Velocity::Fast, Velocity::Fast) => Velocity::Fast,
                }
            };

            // Calculate cells to release
            let cells_to_release = if is_in_intersection || distance_to_intersection < 50.0 {
                self.calculate_cells_to_release_two_path(
                    vehicle_direction,
                    vehicle_route,
                    vx,
                    vy,
                    vw,
                    vh,
                )
            } else {
                Vec::new()
            };

            vehicle_updates.push((
                i,
                final_speed,
                requested_intersection,
                intersection_permission,
                cells_to_release,
                vehicle_id,
            ));
        }

        // Apply updates
        for (
            i,
            final_speed,
            requested_intersection,
            intersection_permission,
            cells_to_release,
            vehicle_id,
        ) in vehicle_updates
        {
            let vehicle = &mut self.active_vehicles[i];

            vehicle.current_speed = final_speed;
            vehicle.requested_intersection = requested_intersection;
            vehicle.intersection_permission = intersection_permission;

            vehicle.update();

            if !cells_to_release.is_empty() {
                self.release_specific_cells(&cells_to_release, vehicle_id);
            }

            self.detect_close_calls(i);
        }
    }

    /// Try intersection request with two-path system and adaptive speed
    fn try_two_path_intersection_request(
        &mut self,
        vehicle_id: usize,
        route: Route,
        direction: Direction,
        current_speed: Velocity,
        current_time: f32,
        distance_to_intersection: f32,
    ) -> (bool, Velocity) {
        // Get cached path for this direction+route combination
        let path = match self.path_cache.get(&(direction, route)) {
            Some(p) => p.clone(),
            None => {
                println!("⚠️ No cached path for {:?} {:?}", direction, route);
                return (false, Velocity::Slow);
            }
        };

        // Try different speeds until we get permission
        let speeds_to_try = match current_speed {
            Velocity::Fast => vec![Velocity::Fast, Velocity::Medium, Velocity::Slow],
            Velocity::Medium => vec![Velocity::Medium, Velocity::Slow],
            Velocity::Slow => vec![Velocity::Slow],
            Velocity::Stopped => vec![Velocity::Fast],
        };

        for attempt_speed in speeds_to_try {
            if let Some(vehicle) = self.active_vehicles.iter_mut().find(|v| v.id == vehicle_id) {
                vehicle.current_speed = attempt_speed;
            }
            // Calculate timing for segment 1
            let time_to_intersection =
                self.calculate_time_with_speed(distance_to_intersection, attempt_speed);
            let segment1_time =
                self.calculate_time_with_speed(path.segment1.distance, attempt_speed);

            let segment1_entry = current_time + time_to_intersection;
            let segment1_exit = segment1_entry + segment1_time;

            // Try to reserve segment 1
            if !self.can_reserve_cells(&path.segment1.cells, segment1_entry, segment1_exit) {
                continue; // Try slower speed
            }

            // If there's a second segment (turning vehicles), check that too
            let mut segment2_exit = segment1_exit;
            if let Some(ref segment2) = path.segment2 {
                let segment2_time =
                    self.calculate_time_with_speed(segment2.distance, attempt_speed);
                segment2_exit = segment1_exit + segment2_time;

                if !self.can_reserve_cells(&segment2.cells, segment1_exit, segment2_exit) {
                    continue; // Try slower speed
                }
            }

            // Both segments can be reserved - make the reservations
            self.reserve_cells_for_vehicle(
                vehicle_id,
                &path.segment1.cells,
                segment1_entry,
                segment1_exit,
            );

            if let Some(ref segment2) = path.segment2 {
                self.reserve_cells_for_vehicle(
                    vehicle_id,
                    &segment2.cells,
                    segment1_exit,
                    segment2_exit,
                );
            }

            println!(
                "✅ Vehicle {} got intersection permission at speed {:?} (segments: {} + {} cells)",
                vehicle_id,
                attempt_speed,
                path.segment1.cells.len(),
                path.segment2.as_ref().map_or(0, |s| s.cells.len())
            );

            return (true, attempt_speed);
        }

        if let Some(vehicle) = self.active_vehicles.iter_mut().find(|v| v.id == vehicle_id) {
            vehicle.current_speed = Velocity::Stopped;
            println!(
                "🛑 Vehicle {} STOPPED at intersection entrance - waiting for clearance",
                vehicle_id
            );
        }
        (false, Velocity::Stopped)
    }

    /// Check if cells can be reserved (without actually reserving them)
    fn can_reserve_cells(&self, cells: &[(usize, usize)], start_time: f32, end_time: f32) -> bool {
        for &(col, row) in cells {
            if col >= self.cols || row >= self.rows {
                continue;
            }
            let idx = self.cell_index(col, row);
            if self.conflict(&self.grid[idx], start_time, end_time) {
                return false;
            }
        }
        true
    }

    /// Reserve cells for a vehicle
    fn reserve_cells_for_vehicle(
        &mut self,
        vehicle_id: usize,
        cells: &[(usize, usize)],
        start_time: f32,
        end_time: f32,
    ) {
        for &(col, row) in cells {
            if col >= self.cols || row >= self.rows {
                continue;
            }
            let idx = self.cell_index(col, row);
            self.grid[idx].slots.push(TimeSlot {
                start: start_time,
                end: end_time,
                vehicle_id,
            });
        }
    }

    /// Calculate time with specific speed
    fn calculate_time_with_speed(&self, distance: f32, speed: Velocity) -> f32 {
        let speed_pixels_per_frame = match speed {
            Velocity::Slow => 3.0,
            Velocity::Medium => 5.0,
            Velocity::Fast => 7.0,
            Velocity::Stopped => return 0.0,
        };

        distance / speed_pixels_per_frame / 60.0 // Convert to seconds
    }

    /// Calculate cells to release for two-path system
    fn calculate_cells_to_release_two_path(
        &self,
        direction: Direction,
        route: Route,
        vx: f32,
        vy: f32,
        vw: f32,
        vh: f32,
    ) -> Vec<(usize, usize)> {
        // Use the cached path to determine which cells to release
        let path = match self.path_cache.get(&(direction, route)) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut cells_to_release = Vec::new();

        // Release cells behind the vehicle based on its current position
        match direction {
            Direction::North => {
                let behind_y = vy + vh;
                if behind_y >= IY_MIN && behind_y <= IY_MAX {
                    let row = ((behind_y - IY_MIN) / self.zone_px as f32) as usize;

                    // Release cells from segment 1 that are behind the vehicle
                    for &(col, cell_row) in &path.segment1.cells {
                        if cell_row == row {
                            cells_to_release.push((col, cell_row));
                        }
                    }
                }
            }
            Direction::South => {
                let behind_y = vy;
                if behind_y >= IY_MIN && behind_y <= IY_MAX {
                    let row = ((behind_y - IY_MIN) / self.zone_px as f32) as usize;

                    for &(col, cell_row) in &path.segment1.cells {
                        if cell_row == row {
                            cells_to_release.push((col, cell_row));
                        }
                    }
                }
            }
            Direction::East => {
                let behind_x = vx;
                if behind_x >= IX_MIN && behind_x <= IX_MAX {
                    let col = ((behind_x - IX_MIN) / self.zone_px as f32) as usize;

                    for &(cell_col, row) in &path.segment1.cells {
                        if cell_col == col {
                            cells_to_release.push((cell_col, row));
                        }
                    }
                }
            }
            Direction::West => {
                let behind_x = vx + vw;
                if behind_x >= IX_MIN && behind_x <= IX_MAX {
                    let col = ((behind_x - IX_MIN) / self.zone_px as f32) as usize;

                    for &(cell_col, row) in &path.segment1.cells {
                        if cell_col == col {
                            cells_to_release.push((cell_col, row));
                        }
                    }
                }
            }
        }

        cells_to_release
    }

    // === UTILITY METHODS ===

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
                    println!(
                        "Spawning vehicle {} ({:?} {:?}) at ({:.0}, {:.0})",
                        vehicle.id, dir, route, spawn_pos.0, spawn_pos.1
                    );
                    self.active_vehicles.push(vehicle);
                }
                Err(e) => println!("Failed to create vehicle: {}", e),
            }
        } else {
            println!(
                "Spawn blocked for {:?} {:?} - too close to existing vehicle",
                dir, route
            );
        }
    }

    fn is_safe_to_spawn(&self, direction: Direction, route: Route, spawn_pos: (f32, f32)) -> bool {
        // Simplified spawn safety check
        for vehicle in &self.active_vehicles {
            if vehicle.direction == direction && vehicle.route == route {
                let distance = match direction {
                    Direction::North | Direction::South => (vehicle.position.1 - spawn_pos.1).abs(),
                    Direction::East | Direction::West => (vehicle.position.0 - spawn_pos.0).abs(),
                };

                if distance < 100.0 {
                    // Minimum spawn distance
                    return false;
                }
            }
        }
        true
    }

    fn track_intersection_times(&mut self, current_time: f32) {
        let mut to_remove = Vec::new();

        for vehicle in &self.active_vehicles {
            let vehicle_id = vehicle.id;

            if vehicle.is_in_intersection() {
                if !self.vehicle_intersection_times.contains_key(&vehicle_id) {
                    self.vehicle_intersection_times
                        .insert(vehicle_id, current_time);
                }
            } else if self.vehicle_intersection_times.contains_key(&vehicle_id) {
                let entry_time = self.vehicle_intersection_times[&vehicle_id];
                let time_in_intersection = current_time - entry_time;

                if time_in_intersection > self.max_time_in_intersection {
                    self.max_time_in_intersection = time_in_intersection;
                }
                if time_in_intersection < self.min_time_in_intersection {
                    self.min_time_in_intersection = time_in_intersection;
                }

                to_remove.push(vehicle_id);
                println!(
                    "Vehicle {} exited intersection after {:.2} seconds",
                    vehicle_id, time_in_intersection
                );
            }
        }

        for id in to_remove {
            self.vehicle_intersection_times.remove(&id);
        }
    }

    fn update_stats_for_exiting_vehicle_by_data(
        &mut self,
        vehicle_id: usize,
        current_speed: Velocity,
        _current_time: f32,
    ) {
        self.total_vehicles_passed += 1;

        let vehicle_max_speed = match current_speed {
            Velocity::Slow => 3.0,
            Velocity::Medium => 5.0,
            Velocity::Fast => 7.0,
            Velocity::Stopped => 0.0,
        };

        if vehicle_max_speed > self.max_velocity_recorded {
            self.max_velocity_recorded = vehicle_max_speed;
        }
        if vehicle_max_speed < self.min_velocity_recorded {
            self.min_velocity_recorded = vehicle_max_speed;
        }

        self.vehicle_intersection_times.remove(&vehicle_id);

        println!(
            "Vehicle {} completed journey. Total vehicles passed: {}",
            vehicle_id, self.total_vehicles_passed
        );
    }

    fn detect_close_calls(&mut self, vehicle_index: usize) {
        let current_vehicle = &self.active_vehicles[vehicle_index];
        if !current_vehicle.is_in_intersection() {
            return;
        }

        for (j, other_vehicle) in self.active_vehicles.iter().enumerate() {
            if vehicle_index == j {
                continue;
            }

            // Create a normalized pair (smaller ID first) to avoid counting (2,3) and (3,2) as different
            let pair = if current_vehicle.id < other_vehicle.id {
                (current_vehicle.id, other_vehicle.id)
            } else {
                (other_vehicle.id, current_vehicle.id)
            };

            // Skip if we already processed this pair this frame
            if self.close_call_pairs_this_frame.contains(&pair) {
                continue;
            }

            let distance = current_vehicle.distance_to_vehicle(other_vehicle);
            let min_safe_distance = 5.0;

            if distance < min_safe_distance
                && (current_vehicle.is_in_intersection() && other_vehicle.is_in_intersection())
            {
                self.close_calls += 1;
                self.close_call_pairs_this_frame.insert(pair);
               
            }
        }
    }

    pub fn print_final_stats(&self) {
        println!("\n=== SMART INTERSECTION FINAL STATISTICS ===");
        println!("Total vehicles passed: {}", self.total_vehicles_passed);
        println!(
            "Max velocity recorded: {:.1} pixels/frame",
            self.max_velocity_recorded
        );
        println!(
            "Min velocity recorded: {:.1} pixels/frame",
            if self.min_velocity_recorded == f32::MAX {
                0.0
            } else {
                self.min_velocity_recorded
            }
        );
        println!(
            "Max time in intersection: {:.2} seconds",
            self.max_time_in_intersection
        );
        println!(
            "Min time in intersection: {:.2} seconds",
            if self.min_time_in_intersection == f32::MAX {
                0.0
            } else {
                self.min_time_in_intersection
            }
        );
        println!("Close calls detected: {}", self.close_calls);
        println!("Active vehicles remaining: {}", self.active_vehicles.len());
        println!("==========================================\n");
    }

    fn release_specific_cells(&mut self, cells: &[(usize, usize)], vehicle_id: usize) {
        for &(col, row) in cells {
            if col >= self.cols || row >= self.rows {
                continue;
            }
            let idx = self.cell_index(col, row);
            self.grid[idx]
                .slots
                .retain(|slot| slot.vehicle_id != vehicle_id);
        }
    }

    fn conflict(&self, cell: &Cell, start: f32, end: f32) -> bool {
        cell.slots
            .iter()
            .any(|slot| start < slot.end && slot.start < end)
    }

    fn cell_index(&self, col: usize, row: usize) -> usize {
        row * self.cols + col
    }

    /// Print the current state of the grid showing cell reservations
    pub fn print_grid(&self, current_time: f32) {
        println!(
            "\n=== INTERSECTION GRID STATE (Time: {:.2}s) ===",
            current_time
        );
        println!(
            "Grid covers intersection area ({},{}) to ({},{})",
            IX_MIN, IY_MIN, IX_MAX, IY_MAX
        );
        println!("Each cell is {}x{} pixels", self.zone_px, self.zone_px);
        println!("Legend: [ ] = Free, [X] = Reserved, [#] = Multiple reservations");

        // Print column headers
        print!("   ");
        for col in 0..self.cols.min(20) {
            print!("{:2} ", col);
        }
        if self.cols > 20 {
            print!("...");
        }
        println!();

        // Print each row
        for row in 0..self.rows.min(20) {
            print!("{:2} ", row);

            for col in 0..self.cols.min(20) {
                let idx = self.cell_index(col, row);
                let cell = &self.grid[idx];

                let active_count = cell
                    .slots
                    .iter()
                    .filter(|slot| slot.start <= current_time && current_time <= slot.end)
                    .count();

                let symbol = match active_count {
                    0 => " ",
                    1 => "X",
                    _ => "#",
                };

                print!("[{}]", symbol);
            }
            if self.cols > 20 {
                print!("...");
            }
            println!();
        }
        if self.rows > 20 {
            println!("...");
        }
        println!("===========================================\n");
    }

    pub fn print_grid_stats(&self, current_time: f32) {
        let mut total_cells = 0;
        let mut active_cells = 0;
        let mut future_reservations = 0;
        let mut total_reservations = 0;

        for cell in &self.grid {
            total_cells += 1;
            total_reservations += cell.slots.len();

            let has_active = cell
                .slots
                .iter()
                .any(|slot| slot.start <= current_time && current_time <= slot.end);

            let has_future = cell.slots.iter().any(|slot| slot.start > current_time);

            if has_active {
                active_cells += 1;
            }
            if has_future {
                future_reservations += 1;
            }
        }

        println!("\n=== GRID STATISTICS (Time: {:.2}s) ===", current_time);
        println!(
            "Total cells: {} ({}x{} grid)",
            total_cells, self.cols, self.rows
        );
        println!("Cell size: {}x{} pixels", self.zone_px, self.zone_px);
        println!("Currently reserved cells: {}", active_cells);
        println!("Cells with future reservations: {}", future_reservations);
        println!("Total reservation slots: {}", total_reservations);
        println!(
            "Grid utilization: {:.1}%",
            (active_cells as f32 / total_cells as f32) * 100.0
        );
        println!("Active vehicles: {}", self.active_vehicles.len());
        println!("=====================================\n");
    }

    pub fn print_grid_with_vehicle_ids(&self, current_time: f32) {
        println!(
            "\n=== GRID WITH VEHICLE IDs (Time: {:.2}s) ===",
            current_time
        );
        println!("Shows vehicle ID of current reservation, or . for free");

        // Print column headers
        print!("   ");
        for col in 0..self.cols.min(20) {
            print!("{:3}", col);
        }
        if self.cols > 20 {
            print!("...");
        }
        println!();

        // Print each row
        for row in 0..self.rows.min(20) {
            print!("{:2} ", row);

            for col in 0..self.cols.min(20) {
                let idx = self.cell_index(col, row);
                let cell = &self.grid[idx];

                // Find active reservation at current time
                let active_vehicle = cell
                    .slots
                    .iter()
                    .find(|slot| slot.start <= current_time && current_time <= slot.end)
                    .map(|slot| slot.vehicle_id);

                match active_vehicle {
                    Some(id) => print!("{:3}", id),
                    None => print!("  ."),
                }
            }
            if self.cols > 20 {
                print!("...");
            }
            println!();
        }
        if self.rows > 20 {
            println!("...");
        }
        println!("==========================================\n");
    }
}
