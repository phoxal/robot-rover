//! Qualify all four native wheels, accumulated yaw and rotational stop on the common rover.
use phoxal::contracts::robotics::MotionSetpoint;
use phoxal::scenario::{CapturePolicy, Simulation};
use std::time::Duration;
phoxal::api!();

fn yaw([w, x, y, z]: [f64; 4]) -> f64 {
    (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z))
}

fn main() -> phoxal::Result<()> {
    let mut simulation = Simulation::new("simulation/scene.xml")?;
    let mut plan = simulation.plan();
    let history = CapturePolicy::best_effort_history(1024)?;
    let body = plan.record_body("robot-rover")?;
    let status = plan.record(api::motion::status(), history)?;
    let wheels = [
        (
            "front_left",
            plan.record(api::front_left_drive::encoder(), history)?,
        ),
        (
            "front_right",
            plan.record(api::front_right_drive::encoder(), history)?,
        ),
        (
            "rear_left",
            plan.record(api::rear_left_drive::encoder(), history)?,
        ),
        (
            "rear_right",
            plan.record(api::rear_right_drive::encoder(), history)?,
        ),
    ];
    plan.advance(Duration::from_millis(500))?;
    plan.send(api::motion::manual(MotionSetpoint {
        linear_x_mps: 0.0,
        angular_z_radps: 1.5,
    }))?;
    plan.advance(Duration::from_millis(20))?;
    let arm = plan.send(api::motion::arm(api::motion::ArmRequest {
        mode: api::motion::ControlMode::Manual,
    }))?;
    plan.advance(Duration::from_millis(6000))?;
    plan.send(api::motion::manual(MotionSetpoint {
        linear_x_mps: 0.0,
        angular_z_radps: 0.0,
    }))?;
    plan.advance(Duration::from_millis(700))?;
    plan.send(api::motion::withdraw_manual())?;
    let disarm = plan.send(api::motion::disarm(phoxal::contracts::Empty {}))?;
    plan.advance(Duration::from_millis(400))?;
    let observed = simulation.run(plan)?;
    assert!(matches!(
        observed.reply(arm)?,
        api::motion::ApplyEmergencyResponse::Accepted
    ));
    assert!(matches!(
        observed.reply(disarm)?,
        api::motion::ApplyEmergencyResponse::Accepted
    ));
    for (name, capture) in wheels {
        assert!(
            observed.history(&capture)?.iter().any(|sample| sample
                .value()
                .velocity_radps
                .is_some_and(|velocity| velocity.abs() > 0.1)),
            "{name} native wheel did not move"
        );
    }
    let statuses = observed.history(&status)?;
    let final_status = statuses
        .last()
        .ok_or_else(|| phoxal::anyhow!("missing Motion status"))?
        .value();
    assert_eq!(final_status.mode, api::motion::ControlMode::Disarmed);
    assert!(final_status.stopped);
    let bodies = observed.body_history(&body)?;
    let accumulated_yaw: f64 = bodies
        .windows(2)
        .map(|pair| {
            let delta =
                yaw(pair[1].value().orientation_wxyz) - yaw(pair[0].value().orientation_wxyz);
            (delta + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
        })
        .sum();
    assert!(
        accumulated_yaw.abs() >= 1.0,
        "native yaw {accumulated_yaw} rad below 1 rad"
    );
    let last = bodies
        .last()
        .ok_or_else(|| phoxal::anyhow!("missing native body"))?
        .value();
    let speed = last
        .linear_velocity_mps
        .iter()
        .map(|velocity| velocity.powi(2))
        .sum::<f64>()
        .sqrt();
    assert!(speed < 0.03, "native residual translation {speed} m/s");
    assert!(
        last.angular_velocity_radps[2].abs() < 0.05,
        "native residual yaw speed {} rad/s",
        last.angular_velocity_radps[2]
    );
    println!(
        "Common rover: four native wheels moved, accumulated yaw {accumulated_yaw:.12} rad, final linear speed {speed:.12} m/s, final yaw speed {:.12} rad/s, Disarmed/stopped",
        last.angular_velocity_radps[2]
    );
    Ok(())
}
