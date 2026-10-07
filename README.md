# Phoxal robot-rover

A minimal rover with a mandatory empty brain, four wheel components, and Motion.
Linux and macOS are supported.
The root application attaches `phoxal::api!()` once and runs `runtime::Brain` through the canonical runtime entry.
The brain publishes nothing; Motion starts disarmed and supplies bounded commands to the wheels only after valid intent and arm requests.

`robot.yaml` owns robot.brain, robot.services, robot.components and their consumer-owned bindings.
Its named sources table shares concrete source selections; each reference is resolved after the selected files merge.
`model.xml` owns the rover; `simulation/scene.xml` selects the physics environment.
Motion selects `drive.differential` with named wheel actuators and uses an authored example wheel radius of 0.11 m and track width of 0.52 m.
These explicit calibration values are not measurements derived from the composed model; automatic geometry is deferred.
The physical free root is base_link, initially at world position [0, 0, 0.21] m, with chassis and wheel placement unchanged.
Recorded robot body pose refers to that physical origin, rather than a floor-projected footprint.
Recorded orientation is a body-to-world unit quaternion in [w, x, y, z] order; linear velocity is at the physical origin and both velocity vectors use world axes.
Native body IDs are model-scoped and changed when the rigid footprint wrapper was removed.
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

The nested robot section and named source references are supported by published SDK 0.72.0 and cargo-phoxal 0.4.0.
Install the released tools and inspect, check or build the common composition:

```sh
cargo install cargo-phoxal --version 0.4.0 --locked
cargo install phoxal-simulator --version 0.2.2 --locked
cargo phoxal config -f robot.yaml
cargo phoxal check -f robot.yaml
cargo phoxal build -f robot.yaml
cargo phoxal prepare -f robot.yaml
cargo check
```

Ignored .phoxal/local-owner.yaml, .phoxal/local-sdk.toml and .phoxal/cargo-local remain explicit private development inputs; ordinary Cargo does not discover their SDK patch automatically.
Keep any selected local layer while its preparation is active, and prepare the common document again before ordinary public-owner work.
Tool development may use an explicitly built source binary, but private owner/SDK overlays are not delivered-owner qualification.
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

Explicit files are ordered as robot.yaml then robot.gamepad.yaml; private owner overlays are optional development inputs and do not qualify delivered owners.
Named selections in resolved config and prepared API inputs are concrete paths or pinned Git selections, without reference metadata.
The published SDK 0.72.0 and cargo-phoxal 0.4.0 support this grouping without local patches.
The physical-root scenarios are qualified with released SDK 0.72.0, cargo-phoxal 0.4.0 and simulator 0.2.2; historical footprint-origin samples are not directly comparable.
After development checks, explicitly prepare/check the common files again so ordinary Cargo consumes the intended gamepad-free composition.

## Behavior checks and ordinary tests

```sh
cargo phoxal scenario scenarios/forward_stop.rs
cargo phoxal scenario scenarios/turn_stop.rs
cargo test
```

The scenario selects its scene explicitly, sends a shared SDK MotionSetpoint, advances simulated time, and checks native displacement and stop speed.
The framework renews the bounded intent until its explicit withdrawal.
Scenario targets are Cargo examples with `test = false`; ordinary Rust tests do not depend on a scenario host.
The common-composition turn_stop scenario additionally checks actual movement of all four wheels, accumulated native yaw, accepted arm/disarm, final Disarmed/stopped status and rotational stop.
Service behavior is tested independently in its owning package with typed SDK admission; retired six-service fixture wiring and conversions are not selected by this rover.

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
