# Phoxal robot-rover

A minimal rover with a mandatory empty brain, four wheel components, and Motion.
Linux and macOS are supported.
The root application attaches `phoxal::api!()` once and runs `runtime::Brain` through the canonical runtime entry.
The brain publishes nothing; Motion starts disarmed and supplies bounded commands to the wheels only after valid intent and arm requests.

`robot.yaml` owns component and service selections and their consumer-owned bindings.
`model.xml` owns the rover; `simulation/scene.xml` selects the physics environment.
Motion selects `drive.differential` with named wheel actuators and uses an authored example wheel radius of 0.11 m and track width of 0.52 m.
These explicit calibration values are not measurements derived from the composed model; automatic geometry is deferred.
The scene uses a 10 ms native quantum.
These are example parameters, not calibration for a physical robot.

Each wheel explicitly selects its driver and binds actuator to the corresponding named Motion output.
DDSM115 has no runtime configuration because its hardware backend is unavailable; hardware startup refuses honestly.
The simulator substitutes all four motors natively while retaining the same graph.
Use cargo phoxal config to inspect the common resolved document without building participants.
Official services and components are selected through local paths or full pinned Git revisions, not crates.io participant packages.
Configuration inspection validates authored shape; compiled check additionally validates the exact selected owner contracts.
For local development, retain selected layer files until preparing a different composition, because ordinary Cargo reads their exact prepared selection and checks input freshness.

## Simulation

Install the public tools with `cargo install cargo-phoxal --locked` and `cargo install phoxal-simulator --locked`.
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

## Gamepad layer

robot.gamepad.yaml selects the standalone gamepad service from the delivered services Git revision and binds Motion's manual input plus gamepad arm/disarm requirements.
Motion and gamepad use the same qualified public owner revision, while components and supervisor retain their independent full Git pins.
The common robot.yaml and forward_stop remain gamepad-free.
This layer uses pure-freeze Pause semantics: admitted authority and logical leases are preserved, and changed OS input becomes visible through ordinary sampling/admission after resume.
Native Linux aarch64 package build/tests are qualified against current local owners, and actual wrapped desktop controls and narrow layout have partial GUI acceptance.
Physical Stadia USB Manual admission, forward/nonzero movement and later commanded-zero/stationarity have partial observed acceptance.
Directed backward/turn/deadman-release, unplug while moving, reconnect-held/fresh reengagement, actual Linux device input and held manipulation/cleanup gestures remain open.
The observed zero/disarm transition is not yet attributed to a directed L1 release, and backward movement is human-reported only.
See the public [service owner documentation](https://github.com/phoxal/services#readme) for package ownership.
The [gamepad package safety and configuration notes](https://github.com/phoxal/services/tree/main/gamepad) belong to the public service owner.

```sh
cargo phoxal config -f robot.yaml -f robot.gamepad.yaml
cargo phoxal check -f robot.yaml -f robot.gamepad.yaml
cargo phoxal build -f robot.yaml -f robot.gamepad.yaml
```

For explicit private development, add .phoxal/local-owner.yaml and use the retained .phoxal/cargo-local wrapper only when selecting the separately retained local SDK patch.
Ordinary commands use the published SDK and public Git pins, with no automatically discovered local SDK patch.
Common and gamepad compositions are qualified through released cargo-phoxal 0.3.0, phoxal-simulator 0.2.1 and registry SDK 0.71.0.
Common native forward_stop retains its exact qualified displacement and stop-speed results through delivered owner acquisition.
After development checks, explicitly prepare/check the common files again so ordinary Cargo consumes the intended gamepad-free composition.

## Behavior checks and ordinary tests

```sh
cargo phoxal scenario scenarios/forward_stop.rs
cargo test
```

The scenario selects its scene explicitly, sends a shared SDK MotionSetpoint, advances simulated time, and checks native displacement and stop speed.
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
