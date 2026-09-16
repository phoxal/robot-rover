//! Rover-first scripted driving demo runtime.
//!
//! See `main.rs` for the executable entrypoint; this crate root exists
//! so that the framework can address the package as a normal Cargo
//! dependency even though only its binary is shipped.

#![doc(hidden)]

pub use scenario_runtime::Scenario;
pub use scenario_runtime::Phase;

// Cargo does not see usage of `motion` inside macro attributes, so we
// re-export the types at the crate root to silence the unused-manifest
// warning without changing the public surface.
pub use motion::MotionIntent as _MotionIntentReexport;

mod scenario_runtime {
    use phoxal::runtime::{InitContext, Runtime, StepContext};

    /// Scripted driving scenario service.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Scenario;

    #[phoxal::runtime(period_ms = 20, timeout_ms = 100, init_timeout_ms = 1_000)]
    impl Runtime for Scenario {
        type Config = ();
        type State = Phase;
        type Inputs = Inputs;
        type Outputs = Outputs;

        fn init(&self, _ctx: &InitContext, _config: ()) -> phoxal::Result<Self::State> {
            Ok(Phase::default())
        }

        fn step(
            &self,
            ctx: &StepContext,
            _state: Self::State,
            _inputs: &Inputs,
        ) -> phoxal::Result<(Self::State, Outputs)> {
            let next = Phase {
                step: ctx.invocation_index(),
            };
            Ok((next, Outputs::default()))
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct Phase {
        /// Current invocation index; persisted so projection methods can
        /// read it without access to `StepContext`.
        pub step: u64,
    }

    #[phoxal::runtime::inputs]
    #[allow(dead_code, reason = "scenario is autonomous, no inputs")]
    pub struct Inputs {}

    #[phoxal::runtime::outputs]
    #[derive(Default)]
    pub struct Outputs {}

    #[phoxal::runtime::outputs]
    #[allow(dead_code, reason = "scenario emits a single setpoint per phase")]
    impl Scenario {
        /// Latest-wins manual intent. The framework publishes this on
        /// every step where it is `Some`; `None` lets the previous
        /// setpoint lapse.
        #[phoxal::runtime::outputs::setpoint(
            port = motion::ports::MANUAL,
            max_bytes = 256,
            valid_for_ms = 100
        )]
        fn manual(&self, state: &Phase) -> Option<motion::MotionIntent> {
            match state.step {
                0..=99 => Some(motion::MotionIntent {
                    owner_id: "scenario-forward".to_owned(),
                    linear_x_mps: 0.6,
                    angular_z_radps: 0.0,
                }),
                100..=199 => Some(motion::MotionIntent {
                    owner_id: "scenario-turn".to_owned(),
                    linear_x_mps: 0.0,
                    angular_z_radps: 1.5,
                }),
                _ => None,
            }
        }

        /// Autonomous slot is held idle throughout the demo.
        #[phoxal::runtime::outputs::setpoint(
            port = motion::ports::AUTONOMOUS,
            max_bytes = 256,
            valid_for_ms = 100
        )]
        fn autonomous(&self, _state: &Phase) -> Option<motion::MotionIntent> {
            None
        }
    }
}