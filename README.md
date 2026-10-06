# Phoxal robot-rover

A minimal rover with a mandatory empty brain, four wheel components, and Motion.
Linux and macOS are supported.
The root application attaches `phoxal::api!()` once and runs `runtime::Brain` through the canonical runtime entry.
The brain publishes nothing; Motion starts disarmed and supplies bounded commands to the wheels only after valid intent and arm requests.

`robot.yaml` owns component and service selections and their connections.
`model.xml` owns the rover; `simulation/scene.xml` selects the physics environment.
Motion selects `drive.differential` with named wheel actuators and uses an authored example wheel radius of 0.11 m and track width of 0.52 m.
These explicit calibration values are not measurements derived from the composed model; automatic geometry is deferred.
The scene uses a 10 ms native quantum.
These are example parameters, not calibration for a physical robot.

## Simulation

Install the public tools with `cargo install cargo-phoxal` and `cargo install phoxal-simulator`.
Native simulation requires a user-managed MuJoCo 3.12.0 library.
See the [simulator prerequisite and discovery instructions](https://github.com/phoxal/simulator#readme).
Set `PHOXAL_MUJOCO_LIBRARY` when the library is outside supported discovery locations.

From this project, launch the desktop with an explicit scene:

```sh
cargo phoxal simulation simulation/scene.xml
```

Use `--paused` for paused startup, `--release` for release compilation, or `--headless --duration 10s` for a finite headless run.
Pause/Run controls one execution; Step advances one paused boundary; Reset returns that execution to its initial state.
Stop shuts down the supervisor and participants; Restart starts a fresh execution from the prepared scene.
Closing the window shuts down the active execution.
An empty brain does not initiate movement merely because simulated time advances.

## Behavior checks and ordinary tests

```sh
cargo phoxal scenario scenarios/forward_stop.rs
cargo test
```

The scenario selects its scene explicitly, sends a typed Motion intent, advances simulated time, and checks native displacement and stop speed.
The framework renews the bounded intent until its explicit withdrawal.
Scenario targets are Cargo examples with `test = false`; ordinary Rust tests do not depend on a scenario host.
Deeper full-stack and conversion qualification belongs to its service and framework owners.

## Deployable build

```sh
cargo phoxal build
```

This assembles the release build under Cargo's target directory and writes `bundle/robot-rover.zip`.
The archive contains the runnable composition and its resources with relative paths and executable permissions.
Use `--output <ZIP_FILE>` to select another archive destination.
Simulator ZIP ingestion is outside the current command surface.

## License

MIT. See [LICENSE](LICENSE).
