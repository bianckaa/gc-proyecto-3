use gc_proyecto_3::platform::time::FixedTimestep;

const STEP: f32 = 1.0 / 60.0;

#[test]
fn accumulates_partial_frames_until_a_step_is_due() {
    let mut timestep = FixedTimestep::new(STEP, 5);
    assert_eq!(timestep.advance(STEP * 0.6), 0);
    assert_eq!(timestep.advance(STEP * 0.6), 1);
    assert!((timestep.alpha() - 0.2).abs() < 1e-3);
}

#[test]
fn runs_same_simulated_time_at_different_frame_rates() {
    let simulate = |frame_delta: f32, frames: u32| {
        let mut timestep = FixedTimestep::new(STEP, 5);
        (0..frames)
            .map(|_| timestep.advance(frame_delta))
            .sum::<u32>()
    };
    for (frame_delta, frames) in [(1.0 / 30.0, 30), (1.0 / 144.0, 144)] {
        let steps = simulate(frame_delta, frames) as i32;
        assert!((steps - 60).abs() <= 1, "pasos simulados: {steps}");
    }
}

#[test]
fn caps_steps_and_drops_backlog_to_avoid_spiral() {
    let mut timestep = FixedTimestep::new(STEP, 5);
    assert_eq!(timestep.advance(1.0), 5);
    assert!(timestep.alpha() < 1.0);
    assert_eq!(timestep.advance(0.0), 0);
}
