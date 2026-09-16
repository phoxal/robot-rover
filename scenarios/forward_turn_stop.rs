//! ForwardTurnStop scenario (P3 accepted experiment).
//!
//! Drives the rover through the canonical arm → forward → turn → stop
//! → disarm sequence over a finite controlled execution. The scenario
//! uses:
//!   - `simulation/scene.xml` (the rover's authored scene).
//!   - Real `phoxal-service-motion` types (`MotionIntent`,
//!     `ApplyEmergencyRequest`, `MotionStatus`) — no manual protobuf
//!     encoder and no handwritten descriptor strings.
//!   - `Motion/Manual` setpoint port for forward / turn / stop
//!     intents.
//!   - `Motion/Status` state capture for the controlled motion
//!     observation.
//!   - `Motion/Emergency` commands port for explicit arm / disarm.
//!   - `Safety/Status` state capture for the safety observation.
//!
//! Validity note: the SDK's [`phoxal::scenario::Validity`] only
//! models `Permanent` today; renewal-intent-on-expiry is a future
//! SDK expansion (see followup-5d11cfc1.md §3). The intent steps
//! below use `Permanent` validity with a comment at each step. The
//! supervisor still admits the bundle, the simulator still drives
//! the controlled execution, and the case host still seals the run
//! against the lifecycle-observed terminal evidence.
//!
//! Rover quantum is 2 ms (5 transitions of 2 ms each = 10 ms) — long
//! enough to drive the full arm → forward → turn → stop → disarm
//! sequence through the controlled execution.

use std::time::Duration;

use phoxal::port::{PortKind, PortSignature};
use phoxal::scenario::{
    Action, Capture, Scenario, ScenarioPlan, Step, Validity,
};
use prost::Message;

use motion::{
    ApplyEmergencyRequest, Arm, ControlMode, Disarm, MotionIntent,
};

/// 2 ms quantum × 6 transitions = 12 ms total controlled execution.
/// Six transitions: arm, forward, forward, turn, stop, disarm.
pub(crate) const DURATION_MICROS: u64 = 12_000;

/// Forward-turn-stop scenario that drives the rover's canonical
/// arm → forward → turn → stop → disarm sequence against
/// `simulation/scene.xml`. The rover owns the arm/disarm behavior;
/// the scenarios are observed-only beyond the boundary the user
/// authors.
#[derive(Debug, Default)]
pub struct ForwardTurnStop;

#[phoxal::scenario]
impl Scenario for ForwardTurnStop {
    fn plan(&self) -> phoxal::Result<ScenarioPlan> {
        let steps = vec![
            // Boundary 0 (0 ms): Arm the rover through Motion/Emergency
            // with ControlMode::Manual and the rover's owner identity.
            Step::new(
                "s00000000",
                0,
                Action::command(
                    "motion",
                    emergency_signature(),
                    encode_arm_request("scenarios/ForwardTurnStop"),
                    "arm-rover",
                    Duration::from_micros(DURATION_MICROS),
                    Duration::from_secs(1),
                )
                .map_err(|e| phoxal::anyhow!("arm action: {e}"))?,
            ),
            // Boundary 1 (2 ms): Forward setpoint through Motion/Manual
            // with linear_x_mps = 0.5. The intent is permanent for the
            // simulator's duration; finite validity + renewal is a
            // future SDK expansion.
            Step::new(
                "s00000001",
                1,
                Action::setpoint(
                    "motion",
                    manual_setpoint_signature(),
                    encode_motion_intent("scenarios/ForwardTurnStop", 0.5, 0.0),
                    Validity::Permanent,
                )
                .map_err(|e| phoxal::anyhow!("forward setpoint: {e}"))?,
            ),
            // Boundary 2 (4 ms): Continue forward with the same intent.
            Step::new(
                "s00000002",
                2,
                Action::setpoint(
                    "motion",
                    manual_setpoint_signature(),
                    encode_motion_intent("scenarios/ForwardTurnStop", 0.5, 0.0),
                    Validity::Permanent,
                )
                .map_err(|e| phoxal::anyhow!("forward continue: {e}"))?,
            ),
            // Boundary 3 (6 ms): Turn setpoint with linear = 0,
            // angular_z = 0.5 rad/s.
            Step::new(
                "s00000003",
                3,
                Action::setpoint(
                    "motion",
                    manual_setpoint_signature(),
                    encode_motion_intent("scenarios/ForwardTurnStop", 0.0, 0.5),
                    Validity::Permanent,
                )
                .map_err(|e| phoxal::anyhow!("turn setpoint: {e}"))?,
            ),
            // Boundary 4 (8 ms): Stop setpoint with linear = 0 and
            // angular = 0. The Motion contract uses zero-velocity
            // intents to stop, not a withdraw; the rover's Motion
            // service is responsible for the controlled deceleration.
            Step::new(
                "s00000004",
                4,
                Action::setpoint(
                    "motion",
                    manual_setpoint_signature(),
                    encode_motion_intent("scenarios/ForwardTurnStop", 0.0, 0.0),
                    Validity::Permanent,
                )
                .map_err(|e| phoxal::anyhow!("stop setpoint: {e}"))?,
            ),
            // Boundary 5 (10 ms): Disarm through Motion/Emergency.
            Step::new(
                "s00000005",
                5,
                Action::command(
                    "motion",
                    emergency_signature(),
                    encode_disarm_request(),
                    "disarm-rover",
                    Duration::from_micros(DURATION_MICROS),
                    Duration::from_secs(1),
                )
                .map_err(|e| phoxal::anyhow!("disarm action: {e}"))?,
            ),
        ];

        let captures = vec![
            Capture::state("motion/status", motion_status_signature())
                .map_err(|e| phoxal::anyhow!("motion status capture: {e}"))?,
            Capture::state("safety/status", safety_status_signature())
                .map_err(|e| phoxal::anyhow!("safety status capture: {e}"))?,
        ];

        ScenarioPlan::with_steps(
            "simulation/scene.xml",
            Duration::from_micros(DURATION_MICROS),
            steps,
            captures,
        )
        .map_err(|e| phoxal::anyhow!("plan validate: {e}"))
    }

    fn verify(&self, run: &phoxal::scenario::ScenarioRun) -> phoxal::Result<()> {
        if !run.is_sealed() {
            return Err(phoxal::anyhow!(
                "ForwardTurnStop: scenario run was not sealed; the case host must seal before verify"
            ));
        }
        if !run.passed() {
            return Err(phoxal::anyhow!(
                "ForwardTurnStop: scenario run did not pass; lifecycle terminal evidence rejected"
            ));
        }

        // Every step boundary must have a recorded outcome.
        for boundary in [
            "s00000000",
            "s00000001",
            "s00000002",
            "s00000003",
            "s00000004",
            "s00000005",
        ] {
            run.outcome(boundary)
                .ok_or_else(|| phoxal::anyhow!("ForwardTurnStop: step {boundary} has no recorded outcome"))?;
        }

        // Motion and safety status captures must be observed.
        run.capture("motion/status")
            .ok_or_else(|| phoxal::anyhow!("ForwardTurnStop: motion/status capture missing"))?;
        run.capture("safety/status")
            .ok_or_else(|| phoxal::anyhow!("ForwardTurnStop: safety/status capture missing"))?;

        // The terminal evidence must carry the lifecycle-observed
        // supervisor-run identity (the bundle's run_id, not a
        // locally fabricated string).
        let evidence = run
            .terminal_evidence()
            .ok_or_else(|| phoxal::anyhow!("ForwardTurnStop: terminal evidence missing"))?;
        if evidence.quantum_ns() == 0 {
            return Err(phoxal::anyhow!(
                "ForwardTurnStop: lifecycle-observed quantum is zero; the case host \
                 did not record the supervisor's terminal facts"
            ));
        }
        if evidence.completed_transitions() == 0 {
            return Err(phoxal::anyhow!(
                "ForwardTurnStop: lifecycle-observed completed transitions are zero; \
                 the simulator did not complete the controlled execution"
            ));
        }
        if !evidence.final_observation_cut() {
            return Err(phoxal::anyhow!(
                "ForwardTurnStop: lifecycle did not observe the final motion / safety cut"
            ));
        }
        if !evidence.final_capture_drain() {
            return Err(phoxal::anyhow!(
                "ForwardTurnStop: lifecycle did not drain the capture buffer"
            ));
        }
        if !evidence.cleanup_ok() {
            return Err(phoxal::anyhow!(
                "ForwardTurnStop: lifecycle cleanup did not succeed"
            ));
        }

        Ok(())
    }
}

fn manual_setpoint_signature() -> PortSignature {
    PortSignature::new(
        "motion/manual",
        "phoxal.motion.v1.Motion",
        "Manual",
        PortKind::Setpoint,
        "phoxal.motion.v1.MotionIntent",
        "phoxal.motion.v1.MotionIntent",
    )
}

fn motion_status_signature() -> PortSignature {
    PortSignature::new(
        "motion/status",
        "phoxal.motion.v1.Motion",
        "Status",
        PortKind::State,
        "google.protobuf.Empty",
        "phoxal.motion.v1.MotionStatus",
    )
}

fn safety_status_signature() -> PortSignature {
    PortSignature::new(
        "safety/status",
        "phoxal.safety.v1.Safety",
        "Status",
        PortKind::State,
        "google.protobuf.Empty",
        "phoxal.safety.v1.SafetyStatus",
    )
}

fn emergency_signature() -> PortSignature {
    PortSignature::new(
        "motion/emergency",
        "phoxal.motion.v1.Motion",
        "Emergency",
        PortKind::Commands,
        "phoxal.motion.v1.ApplyEmergencyRequest",
        "phoxal.motion.v1.ApplyEmergencyResponse",
    )
}

fn encode_motion_intent(owner_id: &str, linear_x_mps: f64, angular_z_radps: f64) -> Vec<u8> {
    let intent = MotionIntent {
        owner_id: owner_id.to_owned(),
        linear_x_mps,
        angular_z_radps,
    };
    intent.encode_to_vec()
}

fn encode_arm_request(owner_id: &str) -> Vec<u8> {
    let request = ApplyEmergencyRequest {
        command: Some(motion::apply_emergency_request::Command::Arm(Arm {
            mode: ControlMode::Manual as i32,
            owner_id: owner_id.to_owned(),
        })),
    };
    request.encode_to_vec()
}

fn encode_disarm_request() -> Vec<u8> {
    let request = ApplyEmergencyRequest {
        command: Some(motion::apply_emergency_request::Command::Disarm(
            Disarm {},
        )),
    };
    request.encode_to_vec()
}