//! Native movement check through the rover's selected Motion service.

use phoxal::scenario::{CapturePolicy, Simulation};

phoxal::api!();

use api::__contracts::phoxal::motion::v1::{
    ArmRequest, ControlMode, MotionIntent, apply_emergency_response::Decision,
};
use api::motion;
use api::{kinematics, safety};

#[phoxal::scenario]
fn forward_turn_stop(sim: &mut Simulation) -> phoxal::Result<()> {
    let mut plan = sim.plan();
    let body = plan.record_body("robot-rover")?;
    let status = plan.record(motion::status(), CapturePolicy::best_effort_history(1_024)?)?;
    let safety_status =
        plan.record(safety::status(), CapturePolicy::best_effort_history(1_024)?)?;
    let odometry = plan.record(
        kinematics::odometry(),
        CapturePolicy::best_effort_history(1_024)?,
    )?;
    plan.wait_steps(50)?;
    plan.send(manual(0.5, 0.0))?;
    plan.wait_steps(1)?;
    let arm = plan.send(motion::arm(ArmRequest {
        mode: ControlMode::Manual as i32,
    }))?;
    plan.wait_steps(1)?;
    for _ in 0..50 {
        plan.send(manual(0.5, 0.0))?;
        plan.wait_steps(3)?;
    }
    for _ in 0..35 {
        plan.send(manual(0.0, 1.5))?;
        plan.wait_steps(3)?;
    }
    plan.send(manual(0.0, 0.0))?;
    plan.wait_steps(20)?;
    plan.send(motion::withdraw_manual())?;
    let disarm = plan.send(motion::disarm(phoxal::contract::Empty {}))?;
    plan.wait_steps(20)?;

    let observed = sim.run(plan)?;
    let arm = observed.reply(arm)?;
    let statuses = observed.history(&status)?;
    let safety_history = observed.history(&safety_status)?;
    let odometry_history = observed.history(&odometry)?;
    assert!(
        matches!(arm.decision.as_ref(), Some(Decision::Accepted(_))),
        "Motion refused Arm: {arm:?}; Motion status: {:?}; Safety status: {:?}; odometry: {:?}",
        statuses.last().map(|value| value.value()),
        safety_history.last().map(|value| value.value()),
        odometry_history.last().map(|value| value.value()),
    );
    let disarm = observed.reply(disarm)?;
    assert!(
        matches!(disarm.decision.as_ref(), Some(Decision::Accepted(_))),
        "Motion refused Disarm: {disarm:?}"
    );
    let final_status = statuses
        .last()
        .ok_or_else(|| phoxal::anyhow!("no Motion status"))?
        .value();
    assert_eq!(final_status.mode, ControlMode::Disarmed as i32);
    assert!(final_status.stopped, "Motion status did not report a stop");
    let body = observed.body_history(&body)?;
    let first = body
        .first()
        .ok_or_else(|| phoxal::anyhow!("body history is empty"))?
        .value();
    let last = body
        .last()
        .ok_or_else(|| phoxal::anyhow!("body history is empty"))?
        .value();
    let displacement = ((last.position_m[0] - first.position_m[0]).powi(2)
        + (last.position_m[1] - first.position_m[1]).powi(2))
    .sqrt();
    assert!(
        displacement >= 0.5,
        "native displacement {displacement:.3} m is too small"
    );
    let speed = last
        .linear_velocity_mps
        .iter()
        .map(|value| value.powi(2))
        .sum::<f64>()
        .sqrt();
    assert!(speed < 0.03, "rover did not stop: {speed:.3} m/s");
    Ok(())
}

fn manual(
    linear_x_mps: f64,
    angular_z_radps: f64,
) -> impl phoxal::scenario::SendOperation<Response = phoxal::contract::Empty> {
    motion::manual(MotionIntent {
        linear_x_mps,
        angular_z_radps,
    })
}
