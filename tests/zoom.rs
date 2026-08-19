use weekly_status_board::zoom::{clamp_zoom, step_zoom, MIN_ZOOM, MAX_ZOOM};

#[test]
fn clamp_and_step() {
    assert_eq!(clamp_zoom(0.0), MIN_ZOOM);
    assert_eq!(clamp_zoom(9.0), MAX_ZOOM);
    assert!((step_zoom(1.0, 1) - 1.1).abs() < 1e-6);
    assert!((step_zoom(1.0, -1) - 0.9).abs() < 1e-6);
    assert_eq!(step_zoom(0.25, -1), MIN_ZOOM);
}
