//! Sandbox rover composition and mission policy.
phoxal::api!();

mod config;
mod conversions;
mod runtime;

fn main() -> phoxal::Result<()> {
    phoxal::runtime::run(runtime::Brain)
}
