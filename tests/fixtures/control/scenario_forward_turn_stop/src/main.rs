//! Rover-first scripted driving demo binary.
//!
//! Phase map (per-quantum at the configured 20 ms period):
//!
//! | Phase   | Steps       | Manual                                  |
//! |---------|-------------|-----------------------------------------|
//! | forward | 0   ..=  99 | linear = 0.6 m/s, angular = 0           |
//! | turn    | 100 ..= 199 | linear = 0,    angular = 1.5 rad/s      |
//! | idle    | 200 ..=     | None  (setpoint expires, motion disarms) |
//!
//! The runtime implementation lives in the `scenario` lib target so the
//! framework can resolve the package as a normal Cargo dependency.

fn main() -> phoxal::Result<()> {
    phoxal::runtime::run(scenario::Scenario)
}