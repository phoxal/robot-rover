//! The robot brain: this project's one mandatory composition root.
//!
//! The sandbox rover has no mission policy yet, so this brain is a no-op with
//! `Config = ()`, `State = ()`, and `Outputs = ()`. Robot-specific mission policy,
//! intent selection, and recovery become ordinary Rust code compiled into this
//! binary.

use phoxal::runtime::{InitContext, Runtime, StepContext};

#[derive(Clone, Copy, Debug, Default)]
struct Brain;

#[phoxal::runtime::inputs]
struct BrainInputs {}

#[phoxal::runtime(period_ms = 20, timeout_ms = 100, init_timeout_ms = 1_000)]
impl Runtime for Brain {
    type Config = ();
    type State = ();
    type Inputs = BrainInputs;
    type Outputs = ();

    fn init(&self, _ctx: &InitContext, _config: Self::Config) -> phoxal::Result<Self::State> {
        Ok(())
    }

    fn step(
        &self,
        _ctx: &StepContext,
        state: Self::State,
        _inputs: &Self::Inputs,
    ) -> phoxal::Result<(Self::State, Self::Outputs)> {
        Ok((state, ()))
    }
}

#[phoxal::runtime::outputs]
impl Brain {}

fn main() -> phoxal::Result<()> {
    phoxal::runtime::run(Brain)
}
