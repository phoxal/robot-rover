//! Intentionally-failing scenario. Demonstrates that an author-side
//! assertion rejection surfaces through the public command. The
//! plan() publishes one real `MotionIntent` setpoint at boundary 0
//! using the actual `phoxal-service-motion` contract types; the
//! `verify()` always returns `Err` so the case host must surface
//! the diagnostic through
//! `cargo phoxal simulation scenario run IntentionallyFailing`.

use std::time::Duration;

use phoxal::port::{PortKind, PortSignature};
use phoxal::scenario::{
    Action, Capture, Quantum, Scenario, ScenarioPlan, ScenarioRun, Step, Validity,
};
use prost::Message;

use motion::MotionIntent;

/// One linear transition at the canonical 2 ms quantum. The plan
/// declares a single authored setpoint at boundary 0 so the
/// lifecycle's record-and-seal path succeeds; the `verify()` call
/// is what fails. This scenario is the negative acceptance case:
/// the case host must surface a non-pass outcome through the
/// `cargo phoxal simulation scenario run` command.
const DURATION_MICROS: u64 = 2_000;
const QUANTUM_MICROS: u32 = 2_000;

fn manual_signature() -> PortSignature {
    PortSignature::new(
        "motion/manual",
        "phoxal.motion.v1.Motion",
        "Manual",
        PortKind::Setpoint,
        "phoxal.motion.v1.MotionIntent",
        "phoxal.motion.v1.MotionIntent",
    )
}

fn status_signature() -> PortSignature {
    PortSignature::new(
        "motion/status",
        "phoxal.motion.v1.Motion",
        "Status",
        PortKind::State,
        "google.protobuf.Empty",
        "phoxal.motion.v1.MotionStatus",
    )
}

#[derive(Default)]
pub struct IntentionallyFailing;

#[phoxal::scenario]
impl Scenario for IntentionallyFailing {
    fn plan(&self) -> phoxal::Result<ScenarioPlan> {
        let _quantum = Quantum::from_micros(QUANTUM_MICROS).expect("quantum");
        let payload = MotionIntent {
            owner_id: "scenarios/IntentionallyFailing".to_owned(),
            linear_x_mps: 0.0,
            angular_z_radps: 0.0,
        }
        .encode_to_vec();
        let steps = vec![Step::new(
            "s00000000",
            0,
            Action::setpoint(
                "motion",
                manual_signature(),
                payload,
                Validity::Permanent,
            )
            .expect("boundary 0 setpoint"),
        )];
        let captures = vec![
            Capture::state("motion/status", status_signature()).expect("motion status capture"),
        ];
        ScenarioPlan::with_steps(
            "simulation/scene.xml",
            Duration::from_micros(DURATION_MICROS),
            steps,
            captures,
        )
        .map_err(Into::into)
    }

    fn verify(&self, _run: &ScenarioRun) -> phoxal::Result<()> {
        Err(phoxal::anyhow!(
            "IntentionallyFailing: this scenario's verify() always returns Err \
             so the public command must surface it as a non-pass outcome"
        ))
    }
}