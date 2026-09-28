//! The rover brain: no mission selected at startup. Both mission intents
//! are leased setpoint outputs bound to the selected Motion service's
//! generated port identities, so the brain publishes exactly the leased
//! calls Motion admits.

use crate::config::Config;
use phoxal::runtime::{InitContext, Runtime, StepContext};

#[phoxal::runtime::inputs]
pub(crate) struct Inputs {}

#[phoxal::runtime::outputs]
#[derive(Default)]
pub(crate) struct Outputs {}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Brain;

#[phoxal::runtime(period_ms = 20, timeout_ms = 100, init_timeout_ms = 1_000)]
impl Runtime for Brain {
    type Config = Config;
    type State = ();
    type Inputs = Inputs;
    type Outputs = Outputs;

    fn init(&self, _ctx: &InitContext, _config: Config) -> phoxal::Result<()> {
        Ok(())
    }

    fn step(
        &self,
        _ctx: &StepContext,
        state: (),
        _inputs: &Inputs,
    ) -> phoxal::Result<((), Outputs)> {
        Ok((state, Outputs::default()))
    }
}

#[phoxal::runtime::outputs]
impl Brain {
    /// No operator mission is selected at startup.
    #[phoxal::runtime::outputs::setpoint(
        port = crate::api::motion::MANUAL.setpoint_port(),
        max_bytes = 256,
        valid_for_ms = 100
    )]
    fn manual(&self, _state: &()) -> Option<crate::api::motion::MotionIntent> {
        None
    }

    /// No autonomous mission is selected at startup.
    #[phoxal::runtime::outputs::setpoint(
        port = crate::api::motion::AUTONOMOUS.setpoint_port(),
        max_bytes = 256,
        valid_for_ms = 100
    )]
    fn autonomous(&self, _state: &()) -> Option<crate::api::motion::MotionIntent> {
        None
    }
}
