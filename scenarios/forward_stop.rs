//! Move the rover forward and verify a native stop.

use phoxal::scenario::Simulation;
use std::time::Duration;
phoxal::api!();

fn main() -> phoxal::Result<()> {
    let mut simulation = Simulation::new("simulation/scene.xml")?;
    let mut plan = simulation.plan();
    plan.advance(Duration::from_millis(500))?;
    plan.send(api::motion::manual(api::motion::MotionIntent {
        linear_x_mps: 0.5,
        angular_z_radps: 0.0,
    }))?;
    plan.advance(Duration::from_millis(20))?;
    let arm = plan.send(api::motion::arm(api::motion::ArmRequest {
        mode: api::motion::ControlMode::Manual,
    }))?;
    plan.advance(Duration::from_secs(2))?;
    plan.expect_displacement_at_least("robot-rover", 0.5)?;
    plan.send(api::motion::withdraw_manual())?;
    let disarm = plan.send(api::motion::disarm(phoxal::contracts::Empty {}))?;
    plan.advance(Duration::from_millis(400))?;
    plan.expect_speed_at_most("robot-rover", 0.03)?;
    let observed = simulation.run(plan)?;
    assert!(
        matches!(
            observed.reply(arm)?,
            api::motion::ApplyEmergencyResponse::Accepted
        ),
        "Motion refused arm"
    );
    assert!(
        matches!(
            observed.reply(disarm)?,
            api::motion::ApplyEmergencyResponse::Accepted
        ),
        "Motion refused disarm"
    );
    Ok(())
}
