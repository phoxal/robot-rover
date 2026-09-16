//! Intentionally-failing scenario. Demonstrates that an author-side
//! assertion rejection surfaces through the public command. The
//! plan() publishes one setpoint; verify() always returns Err so
//! the case host must surface the diagnostic through
//! `cargo phoxal simulation scenario run IntentionallyFailing`.

use std::time::Duration;

use phoxal::scenario::{
    Action, Capture, Quantum, Scenario, ScenarioPlan, ScenarioRun, Step, Validity,
};
use phoxal_port::{PortKind, PortSignature};

/// One linear transition at the canonical 2 ms quantum. The plan
/// declares a single setpoint at boundary 0 so the lifecycle's
/// record-and-seal path succeeds; the verify call is what fails.
const DURATION_MICROS: u64 = 2_000;
const QUANTUM_MICROS: u32 = 2_000;

fn manual_signature() -> PortSignature {
    PortSignature::new(
        "manual",
        "phoxal.motion",
        "Manual",
        PortKind::Setpoint,
        "MotionIntent",
        "MotionStatus",
    )
}

fn status_signature() -> PortSignature {
    PortSignature::new(
        "status",
        "phoxal.motion",
        "Status",
        PortKind::State,
        "MotionStatus",
        "MotionStatus",
    )
}

#[derive(Default)]
pub struct IntentionallyFailing;

#[phoxal::scenario]
impl Scenario for IntentionallyFailing {
    fn plan(&self) -> phoxal::Result<ScenarioPlan> {
        let quantum = Quantum::from_micros(QUANTUM_MICROS).expect("quantum");
        let steps = vec![Step::new(
            "s00000000",
            0,
            Action::setpoint(
                "motion/manual",
                manual_signature(),
                vec![0u8; 24],
                Validity::Permanent,
            )
            .expect("boundary 0 setpoint"),
        )];
        let captures = vec![Capture::state("motion/status", status_signature())
            .expect("motion status capture")];
        ScenarioPlan::with_steps("rover/scene", Duration::from_micros(DURATION_MICROS), steps, captures)
            .map_err(Into::into)
    }

    fn verify(&self, _run: &ScenarioRun) -> phoxal::Result<()> {
        Err(phoxal::anyhow!(
            "IntentionallyFailing: this scenario's verify() always returns Err \
             so the public command must surface it as a non-pass outcome"
        ))
    }
}
