# Phoxal robot-rover

A minimal simulated rover with an empty brain, gamepad input, Motion and four native wheel motors.
Linux and macOS are supported.
Physical DDSM115 hardware remains unavailable; do not use this workflow for real motor actuation.

## Quick start

Install the ordinary Rust robot toolchain and a compatible cargo-phoxal release, then install a complete application from [Phoxal Simulator releases](https://github.com/phoxal/simulator/releases).
The application includes its exact private MuJoCo engine and notices; no separate engine setup or simulator source installation is needed.
The commands below require cargo-phoxal with `-r, --robot` and `build --print-output-path`, plus an application supporting `run <scene> --build <directory>`.
The published cargo-phoxal 0.4.1 tag still has the earlier CLI, and the published simulator 0.2.4 release has no complete application assets.
Use subsequent compatible releases when available; a locally accepted development executable with the same version string does not establish published support.
Check `cargo phoxal --help` and `cargo phoxal build --help` before proceeding.

On macOS, copy the complete application from its DMG into Applications and select its executable:

```sh
export PHOXAL_SIMULATOR='/Applications/Phoxal Simulator.app/Contents/MacOS/phoxal-simulator'
```

On Linux, extract the complete archive for your architecture and select its bundled executable, replacing the placeholders with the downloaded asset's values:

```sh
mkdir -p "$HOME/.local/opt"
tar -xzf phoxal-simulator-<version>-<arch>-linux.tar.gz -C "$HOME/.local/opt"
export PHOXAL_SIMULATOR="$HOME/.local/opt/phoxal-simulator-<version>-<arch>-linux/bin/phoxal-simulator"
```

From this repository, build the common composition once, then query its complete publication path without another build:

```sh
cargo phoxal build -r robot.yaml
build_dir=$(cargo phoxal build -r robot.yaml --print-output-path)
"$PHOXAL_SIMULATOR" run simulation/scene.xml --build "$build_dir" --paused
```

`--print-output-path` is a pure path query, not preparation or compilation.
`cargo phoxal prepare` prepares contracts and generated APIs; it does not assemble the complete runnable directory.
The scene is explicitly `simulation/scene.xml`, and `--build` selects the complete compatible prepared directory.
Click Run when ready; the empty brain does not initiate movement.
The simulator starts in Realtime and its controls operate on the same authoritative native execution.

## Drive with a controller

Connect or pair through the operating system, with sticks centered and L1 released.
Gamepad input is included in the common `robot.yaml` composition; no scenario overlay is needed for joypad driving.
After Run, with sticks centered, freshly press and hold L1, then move the sticks and observe rover movement.
Left Y commands forward/back up to 0.5 m/s; inverted right X commands yaw up to 1.5 rad/s.
L1 maps to gilrs LeftTrigger/left_bumper on the observed Stadia USB controller.
Release L1 to withdraw intent; reconnect/reset requires observed release and a fresh neutral press.
Motion owns limits and authority, and uncertain calls require stop/reset rather than automatic rearming.
Linux needs access to the controller input devices and the gamepad owner's native prerequisites.

Pause freezes logical time, state, authority and logical leases.
Held connected control resumes normally; OS release/disconnect while paused is sampled at the next ordinary gamepad invocation and Motion admission.
Step advances one normal boundary without forcing an extra input poll.
The scenario composition removes onboard gamepad input so deterministic scenarios exclusively provide their own manual intent.

## Desktop controls

Run/Pause resumes/freezes.
Pause first, then use Step for one boundary or Reset to restore the confirmed original placement/execution.
Stop cleans up the supervisor and participants; Restart is available only after confirmed cleanup.
Realtime paces boundaries; Fast runs uncapped without changing physics or leases.
Sim/Wall/Speed show logical time, active wall time and recent achieved rate.

Navigate is the default tool: primary drag orbits, Shift-primary/secondary/middle drag pans, and scroll or pinch zooms.
Click selects a native body; Outline also selects the actual native hierarchy.
Frame selected frames the body; Default view restores the model camera.
For an attached child, Select robot body explicitly selects the nearest ancestor with its own free joint.
Select Manipulate before primary dragging to apply bounded physical force while running.
Paused manipulation translates only an eligible body's own free joint, preserves orientation and resets its velocity without advancing time.
Release, Escape, focus loss, overlays, tool changes and execution transitions cancel held gestures until release and a fresh press.
Scene/build inputs are read-only; session edits do not save robot files or native resources.

## Check behavior

Apply the scenario-only overlay after the common composition, in this exact order:

```sh
cargo phoxal -r robot.yaml -r scenarios/robot.yaml scenario scenarios/forward_stop.rs
cargo phoxal -r robot.yaml -r scenarios/robot.yaml scenario scenarios/turn_stop.rs
```

Forward_stop checks displacement and stopping.
Turn_stop checks all four wheels, yaw, arm/disarm and rotational stop.
These are native simulation evidence, not hardware or controller acceptance.
The scenario command prepares its declared Rust scenario; use the complete application selected by `PHOXAL_SIMULATOR`.
Do not use the scenario overlay for common joypad driving.

## Model and composition

robot.yaml owns nested participants, named pinned Git sources, gamepad intent and arm/disarm bindings.
model.xml owns the physical base_link root and simulation/scene.xml owns its environment, with a 10 ms native quantum.
Configured radius 0.11 m and track 0.52 m are example values, not derived geometry or physical calibration.
Services/components are Git/local-only; the Rust SDK and robot tool retain their registry distribution, while the simulator is a complete GitHub application.

## Troubleshooting and development

If the application's private engine is missing or damaged, reinstall the complete application rather than modify its package or disable operating-system protections.
Invalid scene/model errors need authored-resource repair, not another native download.
Use a graphical session for desktop or replace `--paused` with `--headless --duration 10s` for finite execution.
Unconfirmed cleanup disables restart; inspect the reported authority/process cleanup failure before reopening.
See the [simulator](https://github.com/phoxal/simulator#readme) and [gamepad](https://github.com/phoxal/services/tree/main/gamepad) owners for prerequisites and safety details.

After layered development, restore common preparation before ordinary Cargo:

```sh
cargo phoxal prepare -r robot.yaml
cargo check
cargo test
```

Private development overlays are optional and do not qualify published owners.

## License

MIT. See [LICENSE](LICENSE).
