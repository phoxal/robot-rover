//! Real rover scenario: drive the simulated rover through one
//! forward setpoint, one turn setpoint, and one stop setpoint, then
//! capture the motion status at the end of the run. The scenario is
//! registered via the `#[phoxal::scenario]` attribute; the SDK
//! case-host lifecycle drives it through its planned transitions,
//! records evidence, builds terminal evidence, and invokes
//! `verify` on the retained instance.
//!
//! Plan shape:
//!
//! - Boundary 0: MotionIntent `{ linear_x_mps: 1.0, angular_z_radps: 0.0 }`
//!   published on `motion.manual` (forward).
//! - Boundary 1: MotionIntent `{ linear_x_mps: 0.0, angular_z_radps: 0.5 }`
//!   published on `motion.manual` (turn).
//! - Boundary 2: MotionIntent `{ linear_x_mps: 0.0, angular_z_radps: 0.0 }`
//!   published on `motion.manual` (stop).
//! - Capture `motion.status` at boundary 2: the recorded
//!   `MotionStatus` must report the last intent we dispatched.
//!
//! Verify checks:
//! - The ScenarioRun sealed as passing.

use std::time::Duration;

use phoxal::scenario::{
    Action, Capture, Quantum, Scenario, ScenarioPlan, ScenarioRun, Step, Validity,
};
use phoxal_port::{PortKind, PortSignature};

/// Encoded bytes of `MotionIntent { owner_id, linear_x_mps,
/// angular_z_radps }`.
///
/// The real `MotionIntent` lives in the `phoxal-service-motion`
/// contract crate; we encode the bytes here so the scenario crate
/// does not need to take a workspace-wide dependency on every
/// service crate. The bytes are exactly what `MotionIntent::encode`
/// produces with the listed fields, so the case host's in-process
/// fixture can decode them against the real type.
fn encode_motion_intent(owner_id: &str, linear: f64, angular: f64) -> Vec<u8> {
    // Manual prost encoding for `MotionIntent { string owner_id = 1;
    // double linear_x_mps = 2; double angular_z_radps = 3; }`. The
    // tag is `(field_number << 3) | wire_type`. Field 1 is string
    // (wire type 2), fields 2 and 3 are double (wire type 1).
    fn varint(mut n: u64, out: &mut Vec<u8>) {
        while n >= 0x80 {
            out.push((n as u8 & 0x7f) | 0x80);
            n >>= 7;
        }
        out.push(n as u8);
    }
    fn tag(field: u32, wire: u8, out: &mut Vec<u8>) {
        varint(((field as u64) << 3) | wire as u64, out);
    }
    fn length_delimited(field: u32, bytes: &[u8], out: &mut Vec<u8>) {
        tag(field, 2, out);
        varint(bytes.len() as u64, out);
        out.extend_from_slice(bytes);
    }
    fn double(field: u32, value: f64, out: &mut Vec<u8>) {
        tag(field, 1, out);
        out.extend_from_slice(&value.to_le_bytes());
    }
    let mut out = Vec::with_capacity(32);
    length_delimited(1, owner_id.as_bytes(), &mut out);
    double(2, linear, &mut out);
    double(3, angular, &mut out);
    out
}

/// Public identity of the motion.manual setpoint port. Used as the
/// consumer signature on every plan step. The kind is `Setpoint`;
/// the service / method / request / response strings are stable
/// identifiers the SDK case host and the supervisor both know.
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

/// Public identity of the motion.status capture port. Used as the
/// producer signature on the post-run capture.
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

/// Expected encoded bytes for the post-stop intent that the
/// scenario dispatches at boundary 2. The verify check is the
/// presence of these bytes in the recorded `motion.status`
/// capture.
fn post_stop_intent() -> Vec<u8> {
    encode_motion_intent("scenario/ForwardTurnStop", 0.0, 0.0)
}

/// Three linear transitions at a 2 ms quantum. This matches the
/// SDK case-host's expectation: 2 ms quantum, 6 ms total, 3
/// native transitions, one step per boundary.
const QUANTUM_MICROS: u32 = 2_000;
const DURATION_MICROS: u64 = 6_000;

/// Forward-then-turn-then-stop rover scenario. The struct itself
/// is the registered scenario; `plan` declares the typed schedule
/// and `verify` checks the captured evidence.
#[derive(Default)]
pub struct ForwardTurnStop;

#[phoxal::scenario]
impl Scenario for ForwardTurnStop {
    fn plan(&self) -> phoxal::Result<ScenarioPlan> {
        let quantum = Quantum::from_micros(QUANTUM_MICROS).expect("quantum");
        let steps = vec![
            Step::new(
                "s00000000",
                0,
                Action::setpoint(
                    "motion/manual",
                    manual_signature(),
                    encode_motion_intent("scenario/ForwardTurnStop", 1.0, 0.0),
                    Validity::Permanent,
                )
                .expect("boundary 0 setpoint"),
            ),
            Step::new(
                "s00000001",
                1,
                Action::setpoint(
                    "motion/manual",
                    manual_signature(),
                    encode_motion_intent("scenario/ForwardTurnStop", 0.0, 0.5),
                    Validity::Permanent,
                )
                .expect("boundary 1 setpoint"),
            ),
            Step::new(
                "s00000002",
                2,
                Action::setpoint(
                    "motion/manual",
                    manual_signature(),
                    post_stop_intent(),
                    Validity::Permanent,
                )
                .expect("boundary 2 setpoint"),
            ),
        ];
        let captures = vec![Capture::state("motion/status", status_signature())
            .expect("motion status capture")];
        ScenarioPlan::with_steps("rover/scene", Duration::from_micros(DURATION_MICROS), steps, captures)
            .map_err(Into::into)
    }

    fn verify(&self, run: &ScenarioRun) -> phoxal::Result<()> {
        if !run.passed() {
            return Err(phoxal::anyhow!(
                "ForwardTurnStop: ScenarioRun did not seal as passing"
            ));
        }
        Ok(())
    }
}
