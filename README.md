# Phoxal robot-rover

Public sandbox robot project for trying Phoxal with a small rover. It also
provides a non-application-specific check for framework changes.
Linux and macOS are supported.
Windows and other operating systems are unsupported and unqualified.

The project follows the authored robot project layout and tracks the evolving
pre-1.0 framework.
The native robot model is `model.xml`, selected from `robot.yaml`; the simulation
environment is owned separately by `simulation/scene.xml`.

This repository is the authoritative source for the current public rover example.
See <https://phoxal.com> for the project vision and public introduction.

## Service graph

The authored graph connects all four wheel encoders to Kinematics, World, Safety, and Motion.
Navigation also receives the converted World revision, proving the robot-owned map connection without changing the movement policy.
Each drive service has explicit left and right wheel membership, a 110 mm wheel radius, a 520 mm contact-line separation, and per-wheel direction and gearing.
The brain starts without manual or autonomous intent, and Motion starts disarmed.
The framework service graph provides no implicit mission or automatic arm request.

The scene uses a 10 ms native quantum and an explicitly synthetic WGS84 origin.
The chassis starts with the four wheel surfaces on the ground; native contact determines the settled pose.
These authored parameters are a sandbox model, not hardware calibration or proof of physical driver support.

## Authored brain

The brain runtime is authored through the framework's runtime authoring API.
`src/runtime.rs` declares one `#[phoxal::endpoints]` contract whose two leased setpoint projections are typed by the selected Motion service's generated intent payload, and the inherent `#[phoxal::runtime]` implementation publishes `None` for both missions, so no manual or autonomous intent exists at startup.
The crate attaches its generated API once through `phoxal::api!()` and launches `runtime::Brain` through `phoxal::runtime::run::<runtime::Brain>()`.
Ordinary `From` implementations in `src/conversions.rs` map the selected services' independently typed input expectations.
The API generator attaches their bounded inputs and stamped outputs to the brain's endpoint contract; conversions execute within the brain's normal accepted invocation, preserving the producer's original capture stamp.

## Desktop simulation

Install the public applications from crates.io:

```sh
cargo install cargo-phoxal
cargo install phoxal-simulator
```

Native operations need an externally installed MuJoCo 3.12.0 shared library.
The simulator installs, starts, and shows help without MuJoCo; it checks the user-managed library only when starting native simulation.
See the [simulator prerequisite and discovery instructions](https://github.com/phoxal/simulator#readme).
If the library is outside supported system locations, set `PHOXAL_MUJOCO_LIBRARY` to its actual file.

From this repository, run one command:

```sh
cargo phoxal simulation project
```

The simulator invokes the public tool's source-preparation/bundle command and opens the desktop with simulation running.
The window shows the actual scene, simulation time, and step counter.
**Pause / Run** suspends and resumes the same execution; **Step** advances one boundary while paused.
**Stop** ends the execution and shuts down its supervisor and participants.
**Restart**, available after shutdown, starts a fresh execution from the prepared scene.
Closing the window stops and joins the current execution.
The default bound is 10,000 steps; use `--steps <count>` to change it or `--paused` to open paused.
The brain starts disarmed, so visible simulation advancement alone does not command rover motion.

## Movement qualification

```sh
cargo phoxal test forward_turn_stop -- --nocapture
```

The headless scenario arms Motion, checks replies and status observations, verifies actual displacement, and confirms a final stop and Disarmed state.
The four-wheel qualification and conversion tests retain capture provenance and failure/reset checks.
Use `PHOXAL_SIMULATOR=/absolute/path/phoxal-simulator` to qualify an explicitly selected source binary.

## License

MIT. See [LICENSE](LICENSE).
