# Phoxal robot-rover

Public sandbox robot project for trying Phoxal with a small rover. It also
provides a non-application-specific check for framework changes.

The project follows the authored robot project layout and tracks the evolving
pre-1.0 framework.
The native robot model is `model.xml`, selected from `robot.yaml`; the simulation
environment is owned separately by `simulation/scene.xml`.

This repository is the authoritative source for the current public rover example.
See <https://phoxal.com> for the project vision and public introduction.

## Service graph

The authored graph connects all four wheel encoders to Kinematics, then World, Navigation, Safety, and Motion.
Each drive service has explicit left and right wheel membership, a 110 mm wheel radius, a 520 mm contact-line separation, and per-wheel direction and gearing.
The brain starts without manual or autonomous intent, and Motion starts disarmed.
The framework service graph provides no implicit mission or automatic arm request.

The scene uses a 2 ms native quantum and an explicitly synthetic WGS84 origin.
The chassis starts with the four wheel surfaces on the ground; native contact determines the settled pose.
These authored parameters are a sandbox model, not hardware calibration or proof of physical driver support.

## License

MIT. See [LICENSE](LICENSE).

This review checkpoint pins framework Git revision `ca8926c4d22d3988f346c0b2a1ca20c0e7ce9ad5`.
Registry publication and hardware acceptance remain open.
