//! Sandbox rover composition and mission policy.
//!
//! This executable hosts the authored brain runtime and the generated
//! conversion role together: `cargo phoxal prepare` persists the
//! discovered edges, the build helper emits the hosting glue into
//! `OUT_DIR` through `phoxal::conversions!()`, and the supervisor
//! launches this same binary once per instance id. The authored
//! conversion module reads generated bindings, so it compiles only once
//! the package's own prepared products exist.
phoxal::api!();
phoxal::conversions!();

mod config;
#[cfg(phoxal_self_prepared)]
mod conversions;
mod runtime;

fn main() -> phoxal::Result<()> {
    run_hosted_roles(runtime::Brain)
}
