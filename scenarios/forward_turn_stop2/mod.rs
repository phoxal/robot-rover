//! Rover canonical scenario variant: forward, turn (right), stop.
//!
//! Same shape as `forward_turn_stop`, exercising a different command
//! label so the harness suite can confirm the canonical program
//! schema is portable across scenarios. The fixture captures the
//! latest motion state plus the safety service's published state
//! throughout the run.

use phoxal::port::{PortKind, PortSignature};
use phoxal::scenario::{Action, Capture, Scenario, ScenarioPlan, Step, Validity};
use std::time::Duration;

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

fn safety_state_sig() -> PortSignature {
    PortSignature::new(
        "safety/state",
        "phoxal.safety",
        "State",
        PortKind::State,
        "SafetyState",
        "SafetyState",
    )
}

/// Canonical rover scenario variant: drive forward, turn right, stop.
#[derive(Default)]
pub struct ForwardTurnStop2;

impl Scenario for ForwardTurnStop2 {
    fn plan(&self) -> phoxal::Result<ScenarioPlan> {
        let setpoint = motion_setpoint_sig();
        let command = motion_command_sig();
        let motion_state = motion_state_sig();
        let safety_state = safety_state_sig();
        let forward = Action::setpoint(
            "motion",
            setpoint.clone(),
            vec![0x01, 0x02, 0x03, 0x04],
            Validity::Permanent,
        )?;
        let turn = Action::command(
            "motion",
            command,
            vec![0x40, 0x50, 0x60],
            "turn_right",
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
            Step::new("turn_right", 300, turn),
            Step::new("stop", 600, stop),
        ];
        let captures = vec![
            Capture::state("motion", motion_state)?,
            Capture::state("safety", safety_state)?,
        ];
        ScenarioPlan::with_steps(
            "scenarios/ForwardTurnStop2",
            Duration::from_secs(2),
            steps,
            captures,
        )
        .map_err(Into::into)
    }

    fn verify(&self, _run: &phoxal::scenario::ScenarioRun) -> phoxal::Result<()> {
        Ok(())
    }
}