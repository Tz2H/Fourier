use fourier::{
    animation::{MAX_TRACE_POINTS, Simulation},
    math::{Epicycle, chain},
};

fn assert_point(actual: [f64; 2], expected: [f64; 2]) {
    for axis in 0..2 {
        assert!(
            (actual[axis] - expected[axis]).abs() < 1e-8,
            "actual {actual:?}, expected {expected:?}"
        );
    }
}

#[test]
fn second_vector_starts_at_first_tip() {
    let epicycles = [
        Epicycle {
            radius: 10.0,
            frequency: 1.0,
            phase: 0.0,
        },
        Epicycle {
            radius: 4.0,
            frequency: -2.0,
            phase: 0.0,
        },
    ];
    let points = chain(&epicycles, 0.25);
    assert_point(points[0], [0.0, 0.0]);
    assert_point(points[1], [0.0, 10.0]);
    // 第一根向上，第二根向左；合成末端是 (-4, 10)。
    assert_point(points[2], [-4.0, 10.0]);
}

#[test]
fn opposite_frequencies_cancel_vertical_motion() {
    let epicycles = [
        Epicycle {
            radius: 3.0,
            frequency: 1.0,
            phase: 0.0,
        },
        Epicycle {
            radius: 3.0,
            frequency: -1.0,
            phase: 0.0,
        },
    ];
    for step in 0..=100 {
        let time = step as f64 / 100.0;
        assert_point(
            chain(&epicycles, time)[2],
            [6.0 * (std::f64::consts::TAU * time).cos(), 0.0],
        );
    }
}

#[test]
fn zero_second_radius_recovers_single_circle() {
    let epicycles = [
        Epicycle {
            radius: 10.0,
            frequency: 1.0,
            phase: 0.0,
        },
        Epicycle {
            radius: 0.0,
            frequency: -0.7,
            phase: 0.0,
        },
    ];
    assert_point(chain(&epicycles, 0.25)[2], [0.0, 10.0]);
}

#[test]
fn default_pattern_closes_after_five_seconds() {
    let mut simulation = Simulation::default();
    let start = simulation.points()[2];
    for _ in 0..500 {
        simulation.advance(0.01);
    }
    assert_point(simulation.points()[2], start);
    assert_point(*simulation.trace().back().unwrap(), start);
}

#[test]
fn pause_parameter_changes_and_reset_preserve_consistent_state() {
    let mut simulation = Simulation::default();
    simulation.advance(0.02);
    simulation.toggle_pause();
    let previous_points = simulation.points();
    let previous_trace = simulation.trace().clone();
    simulation.advance(0.05);
    assert_eq!(simulation.points(), previous_points);
    assert_eq!(simulation.trace(), &previous_trace);
    let mut epicycles = simulation.epicycles().to_vec();
    epicycles[1].radius = 30.0;
    simulation.set_epicycles(epicycles);
    assert!(simulation.paused());
    assert_eq!(simulation.seconds(), 0.0);
    assert_eq!(simulation.trace().len(), 1);
    assert_point(simulation.points()[2], [150.0, 0.0]);
    simulation.toggle_pause();
    simulation.advance(0.02);
    assert!(simulation.seconds() > 0.0);
    simulation.reset();
    assert!(!simulation.paused());
    assert_eq!(simulation.trace().len(), 1);
    assert_point(*simulation.trace().front().unwrap(), [150.0, 0.0]);
}

#[test]
fn stationary_vectors_do_not_accumulate_duplicate_trace_points() {
    let mut simulation = Simulation::new([
        Epicycle {
            radius: 10.0,
            frequency: 0.0,
            phase: 0.0,
        },
        Epicycle {
            radius: 0.0,
            frequency: 1.0,
            phase: 0.0,
        },
    ]);
    simulation.advance(0.05);
    assert!(!simulation.is_moving());
    assert_eq!(simulation.trace().len(), 1);
    assert_point(simulation.points()[2], [10.0, 0.0]);
}

#[test]
fn long_frames_are_capped_and_invalid_deltas_ignored() {
    let mut simulation = Simulation::default();
    for delta in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
        simulation.advance(delta);
    }
    assert_eq!(simulation.seconds(), 0.0);
    simulation.advance(10.0);
    assert!((simulation.seconds() - 0.05).abs() < 1e-12);
    // 最快向量每秒 0.6 圈，0.05 秒转 10.8 度，至少细分成 11 段。
    assert!(simulation.trace().len() >= 12);
}

#[test]
fn trace_remains_bounded_and_tracks_the_current_tip() {
    let mut simulation = Simulation::default();
    for _ in 0..1000 {
        simulation.advance(0.05);
    }
    assert_eq!(simulation.trace().len(), MAX_TRACE_POINTS);
    assert_point(*simulation.trace().back().unwrap(), simulation.points()[2]);
}
