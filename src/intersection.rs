use crate::vehicle::Vehicle;

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
            let idx = self.cell_index(col, row);
            if self.conflict(&self.grid[idx], entry_time, exit_time) {
                return false; // conflict found, reject request
            }
        }

        // No conflicts → reserve them
        for &(col, row) in cells {
            let idx = self.cell_index(col, row);
            self.grid[idx].slots.push(TimeSlot {
                start: entry_time,
                end: exit_time,
                vehicle_id,
            });
        }

        true
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
