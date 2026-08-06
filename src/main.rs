//! The robot brain: this project's one mandatory composition root.
//!
//! The sandbox rover has no mission policy yet, so this brain is a no-op with
//! `Config = ()`, `State = ()`, and `Api = ()`. Robot-specific mission policy,
//! intent selection, and recovery become ordinary Rust code compiled into this
//! binary.

use phoxal::prelude::*;

#[phoxal::brain]
struct Brain;

impl Participant for Brain {
    async fn setup(
        &self,
        _ctx: &mut SetupContext<Self>,
        _config: Self::Config,
    ) -> Result<(Self::State, Self::Api)> {
        Ok(((), ()))
    }
}

fn main() -> phoxal::Result<()> {
    phoxal::run::<Brain>()
}
