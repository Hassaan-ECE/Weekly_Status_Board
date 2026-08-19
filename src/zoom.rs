pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 1.50;
pub const ZOOM_STEP: f32 = 0.10;

pub fn clamp_zoom(zoom: f32) -> f32 {
    ((zoom * 10.0).round() / 10.0).clamp(MIN_ZOOM, MAX_ZOOM)
}

pub fn step_zoom(zoom: f32, dir: i32) -> f32 {
    clamp_zoom(zoom + dir as f32 * ZOOM_STEP)
}
