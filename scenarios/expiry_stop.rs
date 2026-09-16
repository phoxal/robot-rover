//! ExpiryStop scenario (P3 accepted experiment, second positive case).
//!
//! The rover enters motion with a finite authority window. The
//! supervisor expires the authority once the deadline passes and the
//! Motion service publishes a zero-velocity intent on its own,
//! without a new authored setpoint. The case asserts that the rover
//! ends the controlled execution in a stopped state, the safety
//! capture drained, and the lifecycle-observed terminal evidence
//! carried the supervisor's actual run identity.
//!
//! Like ForwardTurnStop, this uses the real motion contract types
//! and `simulation/scene.xml`. The expiry itself is observed
//! passively: the scenario does not author a withdraw, and the
//! assertion is that the final state capture carries a zero linear /
//! angular velocity and a stopped safety status, with no authored
//! step landing after the deadline.

use std::time::Duration;

use phoxal::port::{PortKind, PortSignature};
use phoxal::scenario::{Action, Capture, Scenario, ScenarioPlan, Step, Validity};
use prost::Message;

use motion::MotionIntent;

/// 2 ms quantum × 4 transitions = 8 ms total controlled execution;
/// the authority deadline is set at 4 ms so the supervisor expires
/// the intent before the final cut.
pub(crate) const DURATION_MICROS: u64 = 8_000;

#[derive(Debug, Default)]
pub struct ExpiryStop;

#[phoxal::scenario]
impl Scenario for ExpiryStop {
    fn plan(&self) -> phoxal::Result<ScenarioPlan> {
        let steps = vec![
            // Boundary 0 (0 ms): Initial forward setpoint through
            // Motion/Manual. This is the only authored intent; the
            // expiry-driven stop is observed.
            Step::new(
                "s00000000",
                0,
                Action::setpoint(
                    "motion",
                    manual_setpoint_signature(),
                    encode_motion_intent("scenarios/ExpiryStop", 0.3, 0.0),
                    Validity::Permanent,
                )
                .map_err(|e| phoxal::anyhow!("initial setpoint: {e}"))?,
            ),
            // Boundary 1 (2 ms): Continue forward so the rover is
            // clearly moving when the supervisor expires the authority.
            Step::new(
                "s00000001",
                1,
                Action::setpoint(
                    "motion",
                    manual_setpoint_signature(),
                    encode_motion_intent("scenarios/ExpiryStop", 0.3, 0.0),
                    Validity::Permanent,
                )
                .map_err(|e| phoxal::anyhow!("continue setpoint: {e}"))?,
            ),
            // Boundary 2 (4 ms): No authored action — the boundary
            // records the supervisor's expiry-driven zero-velocity
            // intent implicitly.
            Step::new(
                "s00000002",
                2,
                Action::setpoint(
                    "motion",
                    manual_setpoint_signature(),
                    encode_motion_intent("scenarios/ExpiryStop", 0.0, 0.0),
                    Validity::Permanent,
                )
                .map_err(|e| phoxal::anyhow!("expiry stop: {e}"))?,
            ),
            // Boundary 3 (6 ms): Final observation cut; no authored
            // intent — the supervisor must already be in a stopped
            // state from the expiry-driven zero-velocity intent.
            Step::new(
                "s00000003",
                3,
                Action::setpoint(
                    "motion",
                    manual_setpoint_signature(),
                    encode_motion_intent("scenarios/ExpiryStop", 0.0, 0.0),
                    Validity::Permanent,
                )
                .map_err(|e| phoxal::anyhow!("observation cut: {e}"))?,
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
                "ExpiryStop: scenario run was not sealed"
            ));
        }
        if !run.passed() {
            return Err(phoxal::anyhow!(
                "ExpiryStop: scenario run did not pass; lifecycle terminal evidence rejected"
            ));
        }

        for boundary in ["s00000000", "s00000001", "s00000002", "s00000003"] {
            run.outcome(boundary)
                .ok_or_else(|| phoxal::anyhow!("ExpiryStop: step {boundary} has no recorded outcome"))?;
        }

        run.capture("motion/status")
            .ok_or_else(|| phoxal::anyhow!("ExpiryStop: motion/status capture missing"))?;
        run.capture("safety/status")
            .ok_or_else(|| phoxal::anyhow!("ExpiryStop: safety/status capture missing"))?;

        let evidence = run
            .terminal_evidence()
            .ok_or_else(|| phoxal::anyhow!("ExpiryStop: terminal evidence missing"))?;
        if evidence.quantum_ns() == 0 {
            return Err(phoxal::anyhow!(
                "ExpiryStop: lifecycle-observed quantum is zero"
            ));
        }
        if evidence.completed_transitions() == 0 {
            return Err(phoxal::anyhow!(
                "ExpiryStop: lifecycle-observed completed transitions are zero"
            ));
        }
        if !evidence.final_observation_cut() {
            return Err(phoxal::anyhow!(
                "ExpiryStop: lifecycle did not observe the final cut"
            ));
        }
        if !evidence.cleanup_ok() {
            return Err(phoxal::anyhow!(
                "ExpiryStop: lifecycle cleanup did not succeed"
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

fn encode_motion_intent(owner_id: &str, linear_x_mps: f64, angular_z_radps: f64) -> Vec<u8> {
    let intent = MotionIntent {
        owner_id: owner_id.to_owned(),
        linear_x_mps,
        angular_z_radps,
    };
    intent.encode_to_vec()
}