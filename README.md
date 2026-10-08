# Phoxal robot-rover

A minimal simulated rover with an empty brain, gamepad input, Motion and four native wheel motors.
Linux and macOS are supported.
Physical DDSM115 hardware remains unavailable; do not use this workflow for real motor actuation.

## Quick start

Install the released tools, explicitly set up MuJoCo, then run from this repository:

```sh
cargo install cargo-phoxal --version 0.4.1 --locked
cargo install phoxal-simulator --version 0.2.4 --locked
phoxal-simulator setup
cargo phoxal check
cargo phoxal simulation simulation/scene.xml
```

The empty brain does not initiate movement.
The simulator starts in Realtime; its controls operate on the same authoritative native execution.

## Drive with a controller

Connect or pair through the operating system, with sticks centered and L1 released.
Gamepad input is part of the common composition:

```sh
cargo phoxal config
cargo phoxal check
cargo phoxal simulation simulation/scene.xml
```

With sticks centered, freshly press and hold L1, then move the sticks and observe rover movement.
Left Y commands forward/back up to 0.5 m/s; inverted right X commands yaw up to 1.5 rad/s.
L1 maps to gilrs LeftTrigger/left_bumper on the observed Stadia USB controller.
Release withdraws intent; reconnect/reset requires observed release and a fresh neutral press.
Motion owns limits and authority, and uncertain calls require stop/reset rather than automatic rearming.
Linux needs access to the controller input devices and the gamepad owner's native prerequisites.

Pause freezes logical time, state, authority and logical leases.
Held connected control resumes normally; OS release/disconnect while paused is sampled at the next ordinary gamepad invocation and Motion admission.
Step advances one normal boundary without forcing an extra input poll.
The scenario composition removes onboard gamepad input so deterministic scenarios exclusively provide their own manual intent.

## Desktop controls

Run/Pause resumes/freezes.
Pause first, then use Step for one boundary or Reset to restore the original scene/execution.
Stop cleans up the supervisor and participants; Restart is available only after confirmed cleanup.
Realtime paces boundaries; Fast runs uncapped without changing physics or leases.
Sim/Wall/Speed show logical time, active wall time and recent achieved rate.

Click selects a native body; right drag orbits, Shift-right/middle drag pans, scroll zooms and Shift-scroll pans.
Focus selected frames the body; Default view resets the camera.
Select base_link or explicitly Select movable ancestor before manipulating the rover root.
Primary drag applies physical force while running; paused eligible free-body translation updates pose and resets that body's velocity without advancing time.
Release/Escape/focus loss and liveness timeout clear simulator-owned drag force.

## Check behavior

```sh
cargo phoxal -f robot.yaml -f scenarios/robot.yaml scenario scenarios/forward_stop.rs
cargo phoxal -f robot.yaml -f scenarios/robot.yaml scenario scenarios/turn_stop.rs
cargo phoxal prepare
cargo check
cargo test
```

Forward_stop checks displacement and stopping.
Turn_stop checks all four wheels, yaw, arm/disarm and rotational stop.
These are native simulation evidence, not hardware or controller acceptance.
Earlier retained-build GUI/controller observations are separate from published-owner native checks.
Actual current-build GUI/held gestures, directed controller back/turn/release/unplug/reconnect and Linux device input remain deferred.

## Model and composition

robot.yaml owns nested participants, named pinned Git sources, gamepad intent and arm/disarm bindings.
model.xml owns the physical base_link root and simulation/scene.xml owns its environment, with a 10 ms native quantum.
Configured radius 0.11 m and track 0.52 m are example values, not derived geometry or physical calibration.
Services/components are Git/local-only; SDK/tool/application releases retain their normal registry distribution.

## Troubleshooting and development

Run the simulator's displayed contextual setup command for missing/incompatible runtime; a strict PHOXAL_MUJOCO_LIBRARY override must be repaired or removed.
Invalid scene/model errors need authored-resource repair, not another native download.
Use a graphical session for desktop or `--headless --duration 10s` for finite execution.
Unconfirmed cleanup disables restart; confirm authority/process cleanup and close/reopen.
See the [simulator](https://github.com/phoxal/simulator#readme) and [gamepad](https://github.com/phoxal/services/tree/main/gamepad) owners for prerequisites and safety details.

After layered development, restore common preparation before ordinary Cargo:

```sh
cargo phoxal prepare -f robot.yaml
cargo check
```

Private development overlays are optional and do not qualify published owners.

## License

MIT. See [LICENSE](LICENSE).
