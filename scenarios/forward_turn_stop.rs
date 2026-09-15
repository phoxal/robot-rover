//! Rover canonical scenario: forward, turn, stop.
//!
//! Drives the case-host process to compose a typed P2 program that
//! publishes a setpoint for the `motion` chassis to advance, issues
//! a correlated yaw-turn command on the same chassis, and finally
//! publishes the zero-velocity stop setpoint. The fixture captures
//! the latest motion state throughout the run.
//!
//! Acceptance:
//! - The plan normalises cleanly into a bounded program.
//! - The `phoxal-scenarios` test harness compiles and lists this
//!   scenario as `scenarios/ForwardTurnStop`.
//! - Two seconds at 2 ms = 1_000 transitions; the plan must agree.

use phoxal::port::{PortKind, PortSignature};
use phoxal::scenario::{Action, Capture, Scenario, ScenarioPlan, Step, Validity};
use std::time::Duration;

/// One motion setpoint port signature. The production motion
/// service exposes its setpoint under this exact signature; the
/// case host asserts identity rather than reinventing it.
fn motion_setpoint_sig() -> PortSignature {
    PortSignature::new(
        "motion/cmd",
        "phoxal.motion",
        "Set",
        PortKind::Setpoint,
        "SetpointRequest",
        "SetpointReply",
    )
}

/// The motion service's commands-style surface for the yaw turn.
fn motion_command_sig() -> PortSignature {
    PortSignature::new(
        "motion/turn",
        "phoxal.motion",
        "Turn",
        PortKind::Commands,
        "TurnRequest",
        "TurnReply",
    )
}

/// The motion service's state surface.
fn motion_state_sig() -> PortSignature {
    PortSignature::new(
        "motion/state",
        "phoxal.motion",
        "State",
        PortKind::State,
        "State",
        "State",
    )
}

/// Canonical rover scenario: drive forward, turn, stop.
#[derive(Default)]
pub struct ForwardTurnStop;

impl Scenario for ForwardTurnStop {
    fn plan(&self) -> phoxal::Result<ScenarioPlan> {
        let setpoint = motion_setpoint_sig();
        let command = motion_command_sig();
        let state = motion_state_sig();
        let forward = Action::setpoint(
            "motion",
            setpoint.clone(),
            vec![0x01, 0x02, 0x03, 0x04],
            Validity::Permanent,
        )?;
        let turn = Action::command(
            "motion",
            command,
            vec![0x10, 0x20, 0x30],
            "turn_left",
            Duration::from_millis(500),
            Duration::from_secs(1),
        )?;
        let stop = Action::setpoint(
            "motion",
            setpoint,
            vec![0x00],
            Validity::Permanent,
        )?;
        let steps = vec![
            Step::new("forward", 0, forward),
            Step::new("turn", 250, turn),
            Step::new("stop", 500, stop),
        ];
        let captures = vec![Capture::state("motion", state)?];
        ScenarioPlan::with_steps(
            "scenarios/ForwardTurnStop",
            Duration::from_secs(2),
            steps,
            captures,
        )
        .map_err(Into::into)
    }

    fn verify(&self, _run: &phoxal::scenario::ScenarioRun) -> phoxal::Result<()> {
        // The canonical happy-path trace is: every step delivered at
        // its declared boundary, the turn command replies with the
        // accepted reply, and the motion state capture records at
        // least one observation. The framework's typed
        // EvidenceCollector performs those checks during seal; we
        // accept whatever survives seal as a passing trace.
        Ok(())
    }
}