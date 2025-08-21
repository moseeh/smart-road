use crate::vehicle::Vehicle;
use crate::velocities::Velocity;

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
        }
    }

    pub fn add_vehicle(&mut self, vehicle: Vehicle<'a>) {
        self.active_vehicles.push(vehicle);
    }

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

    /// Manage vehicle approaching intersection - returns recommended speed
    pub fn manage_vehicle_intersection_approach(
        &mut self,
        vehicle: &mut Vehicle,
        current_time: f32,
    ) -> Velocity {
        let distance_to_intersection = vehicle.distance_to_intersection();

        // If vehicle is far from intersection, use normal traffic rules only
        if distance_to_intersection > 120.0 {
            return Velocity::Fast;
        }

        // If vehicle is in intersection, maintain speed
        if vehicle.is_in_intersection() {
            return Velocity::Fast;
        }

        // Vehicle is approaching intersection - check if it can get permission
        if !vehicle.requested_intersection {
            let permission =
                self.try_intersection_request(vehicle, current_time, distance_to_intersection);
            vehicle.requested_intersection = true;
            vehicle.intersection_permission = permission;
        }

        // Determine speed based on permission and distance
        if vehicle.intersection_permission {
            Velocity::Fast // Green light - go fast
        } else {
            // No permission - slow down based on distance
            if distance_to_intersection < 25.0 {
                Velocity::Slow // Very close - almost stop
            } else if distance_to_intersection < 60.0 {
                Velocity::Medium // Getting close - slow down
            } else {
                Velocity::Medium // Still some distance - moderate speed
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
            crate::route::Direction::North | crate::route::Direction::South => {
                300.0 + vehicle.height as f32
            }
            crate::route::Direction::East | crate::route::Direction::West => {
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
            crate::route::Route::Straight => {
                self.get_straight_path_cells(vehicle, zone_px, &mut cells);
            }
            crate::route::Route::Right => {
                self.get_right_turn_path_cells(vehicle, zone_px, &mut cells);
            }
            crate::route::Route::Left => {
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
        let visual_center = (vx + vw / 2.0, vy + vh / 2.0);

        match vehicle.direction {
            crate::route::Direction::North | crate::route::Direction::South => {
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
            crate::route::Direction::East | crate::route::Direction::West => {
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
            crate::route::Direction::North => {
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
            crate::route::Direction::South => {
                let end_col = ((vx + vw - IX_MIN) / zone_px) as usize + 1;
                let start_row = ((vy - IX_MIN) / zone_px) as usize;
                let end_row = ((vy + vh - IY_MIN) / zone_px) as usize + 5;

                for row in start_row..end_row.min(self.rows) {
                    for col in 0..end_col.min(self.cols) {
                        cells.push((col, row));
                    }
                }
            }
            crate::route::Direction::East => {
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
            crate::route::Direction::West => {
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

    /// Reset vehicle's intersection request when it's far enough away
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

/// Helper function to release cells behind a moving vehicle
pub fn release_cells_behind_vehicle(intersection: &mut SmartIntersection, vehicle: &Vehicle) {
    let zone_px = intersection.zone_px as f32;
    let (vx, vy, vw, vh) = vehicle.get_visual_bounds();

    let cells_to_release = match vehicle.direction {
        crate::route::Direction::North => {
            // Release cells below visual bounds
            let behind_y = vy + vh + zone_px;
            if behind_y >= IY_MIN && behind_y < IY_MAX {
                let left_col = ((vx - IX_MIN) / zone_px) as usize;
                let right_col = ((vx + vw - IX_MIN) / zone_px) as usize;
                let row = ((behind_y - IY_MIN) / zone_px) as usize;
                
                let mut cells = Vec::new();
                for col in left_col..=right_col.min(intersection.cols - 1) {
                    if col < intersection.cols && row < intersection.rows {
                        cells.push((col, row));
                    }
                }
                cells
            } else {
                vec![]
            }
        }
        crate::route::Direction::South => {
            // Release cells above visual bounds
            let behind_y = vy - zone_px;
            if behind_y >= IY_MIN {
                let left_col = ((vx - IX_MIN) / zone_px) as usize;
                let right_col = ((vx + vw - IX_MIN) / zone_px) as usize;
                let row = ((behind_y - IY_MIN) / zone_px) as usize;
                
                let mut cells = Vec::new();
                for col in left_col..=right_col.min(intersection.cols - 1) {
                    if col < intersection.cols && row < intersection.rows {
                        cells.push((col, row));
                    }
                }
                cells
            } else {
                vec![]
            }
        }
        crate::route::Direction::East => {
            // Release cells to the left of visual bounds
            let behind_x = vx - zone_px;
            if behind_x >= IX_MIN {
                let top_row = ((vy - IY_MIN) / zone_px) as usize;
                let bottom_row = ((vy + vh - IY_MIN) / zone_px) as usize;
                let col = ((behind_x - IX_MIN) / zone_px) as usize;
                
                let mut cells = Vec::new();
                for row in top_row..=bottom_row.min(intersection.rows - 1) {
                    if col < intersection.cols && row < intersection.rows {
                        cells.push((col, row));
                    }
                }
                cells
            } else {
                vec![]
            }
        }
        crate::route::Direction::West => {
            // Release cells to the right of visual bounds
            let behind_x = vx + vw + zone_px;
            if behind_x < IX_MAX {
                let top_row = ((vy - IY_MIN) / zone_px) as usize;
                let bottom_row = ((vy + vh - IY_MIN) / zone_px) as usize;
                let col = ((behind_x - IX_MIN) / zone_px) as usize;
                
                let mut cells = Vec::new();
                for row in top_row..=bottom_row.min(intersection.rows - 1) {
                    if col < intersection.cols && row < intersection.rows {
                        cells.push((col, row));
                    }
                }
                cells
            } else {
                vec![]
            }
        }
    };

    if !cells_to_release.is_empty() {
        intersection.release_specific_cells(&cells_to_release, vehicle.id);
    }
}