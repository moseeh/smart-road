use crate::vehicle::Vehicle;

/// Intersection geometry on your 1000x1000 canvas:
/// 300x300 centered square => [350,650] x [350,650]
const IX_MIN: f32 = 350.0;
const IY_MIN: f32 = 350.0;
const IX_MAX: f32 = 650.0;
const IY_MAX: f32 = 650.0;

pub struct SmartIntersection {
    pub active_vehicles: Vec<Vehicle>,
    pub total_vehicles_passed: u32,
    pub max_velocity_recorded: f32,
    pub min_velocity_recorded: f32,
    pub max_time_in_intersection: f32,
    pub min_time_in_intersection: f32,
    pub close_calls: u32,
    pub is_running: bool,
}

impl SmartIntersection {
    pub fn new() -> Self {
        Self {
            active_vehicles: Vec::new(),
            total_vehicles_passed: 0,
            max_velocity_recorded: 0.0,
            min_velocity_recorded: f32::MAX,
            max_time_in_intersection: 0.0,
            min_time_in_intersection: f32::MAX,
            close_calls: 0,
            is_running: true,
        }
    }
}
