#!/usr/bin/env python3
"""R2 acceptance script: rover-first scripted forward/turn/stop demo.

Runs the framework pipeline and validates the contract surface for the
scenario fixture.  Per project convention this script is not committed
to git; it lives in scripts/ as a throwaway test rig for the duration
of the R0-R4 work.

Pass criteria (each is a hard assertion):
    1. `cargo phoxal check` exits 0
    2. `cargo phoxal build` produces target/phoxal/robot-rover/bundle
    3. `cargo phoxal simulation run --headless --steps 300` reports:
       - outcome == "success"
       - completed_steps == requested_steps == 300
       - provider_contract_verified == True
       - cleanup complete (supervisor_exited, no error)
       - the simulator's native_bindings enumerate all 4 wheel motors
         with producer == "motion"
       - the simulator's native_bindings enumerate the 5 components
       - robot_bundle_identity is stable across reruns (determinism)
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CARGO_PHOXAL = Path("/Users/jbernavaprah/RustroverProjects/phoxal/framework/target/release/cargo-phoxal")
SIMULATOR = "/Users/jbernavaprah/RustroverProjects/phoxal/simulator/target/Phoxal Simulator.app/Contents/MacOS/phoxal-simulator-mujoco"
SCENE = ROOT / "target/phoxal/robot-rover/bundle/source/simulation/scene.xml"
BUNDLE = ROOT / "target/phoxal/robot-rover/bundle"
IDENTITY_LOG = ROOT / ".phoxal/control-identity.log"
STEPS = 300


def run(cmd: list[str]) -> subprocess.CompletedProcess:
    print(f"$ {' '.join(cmd)}", flush=True)
    return subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True)


def assert_step(label: str, condition: bool, detail: str = "") -> None:
    status = "OK " if condition else "FAIL"
    print(f"  [{status}] {label}{(' — ' + detail) if detail else ''}", flush=True)
    if not condition:
        raise SystemExit(1)


def main() -> int:
    assert CARGO_PHOXAL.exists(), f"cargo-phoxal not at {CARGO_PHOXAL}"
    assert Path(SIMULATOR).exists(), f"simulator not at {SIMULATOR}"
    assert SCENE.exists(), f"scene not at {SCENE} (run cargo phoxal build first)"

    print("\n=== R2 acceptance: rover-first scripted driving demo ===\n")

    # 1. check
    print("\n[1/4] cargo phoxal check")
    res = run([str(CARGO_PHOXAL), "phoxal", "check", "--locked"])
    assert_step("check exit 0", res.returncode == 0, res.stderr.splitlines()[-1] if res.stderr else "")

    # 2. build
    print("\n[2/4] cargo phoxal build")
    res = run([str(CARGO_PHOXAL), "phoxal", "build", "--locked"])
    assert_step("build exit 0", res.returncode == 0, res.stderr.splitlines()[-1] if res.stderr else "")
    assert_step("bundle dir exists", (BUNDLE / "manifest.json").exists())

    # 3. simulation run
    print(f"\n[3/4] cargo phoxal simulation run --headless --steps {STEPS}")
    res = run([
            str(CARGO_PHOXAL),
            "phoxal",
            "simulation",
            "run",
            "--headless",
            "--steps",
            str(STEPS),
            "--simulator",
            SIMULATOR,
            "--locked",
            str(SCENE),
        ])
    assert_step("run exit 0", res.returncode == 0, res.stderr.splitlines()[-1] if res.stderr else "")

    # 4. parse the simulator's stdout envelope (cargo-phoxal emits two:
    # the inner simulator envelope with native_bindings/outcome, and an
    # outer envelope with cleanup/supervisor_ready. We capture both.)
    print("\n[4/4] parsing simulator envelope")
    envelope: dict | None = None
    outer: dict | None = None
    for line in res.stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("schema") != "phoxal/simulation-run/v0":
            continue
        if "native_bindings" in obj and envelope is None:
            envelope = obj
        elif "cleanup" in obj and outer is None:
            outer = obj
    assert_step("envelope parsed", envelope is not None)
    assert_step("outer envelope parsed", outer is not None)

    assert_step(
        "outcome == success",
        envelope["outcome"] == "success",
        f"got {envelope['outcome']!r}",
    )
    assert_step(
        f"completed_steps == {STEPS}",
        envelope["completed_steps"] == STEPS,
        f"got {envelope['completed_steps']}",
    )
    assert_step(
        f"requested_steps == {STEPS}",
        envelope["requested_steps"] == STEPS,
        f"got {envelope['requested_steps']}",
    )
    assert_step(
        "provider_contract_verified",
        envelope["provider_contract_verified"] is True,
    )

    bindings = envelope["native_bindings"]
    actuation_blocks = bindings.get("actuation", [])
    motors = [a for b in actuation_blocks for a in b.get("actuators", [])]
    motor_ids = {a["actuator_id"] for a in motors}
    expected_motors = {
        "front_left_drive.motor",
        "front_right_drive.motor",
        "rear_left_drive.motor",
        "rear_right_drive.motor",
    }
    assert_step(
        "4 wheel motors bound",
        motor_ids == expected_motors,
        f"got {sorted(motor_ids)}",
    )
    # `producer` lives on the parent actuation block, not on each actuator.
    motion_actuation = [
        b for b in actuation_blocks
        if b.get("producer") == "motion" and b.get("port") == "actuators"
    ]
    assert_step(
        "actuation block produced by motion service on `actuators` port",
        len(motion_actuation) == 1
        and sum(len(b.get("actuators", [])) for b in motion_actuation) == 4,
        f"got {[(b.get('producer'), b.get('port'), len(b.get('actuators', []))) for b in actuation_blocks]}",
    )

    obs_producers = {
        o["producer"] for o in bindings.get("observations", [])
    }
    expected_components = {
        "front_camera",
        "front_center_tof",
        "front_left_drive",
        "front_right_drive",
        "rear_left_drive",
        "rear_right_drive",
        "gnss",
        "imu",
    }
    assert_step(
        "5 component producers present",
        expected_components.issubset(obs_producers),
        f"missing {sorted(expected_components - obs_producers)}",
    )

    cleanup = (outer or envelope).get("cleanup", {}) or {}
    assert_step(
        "cleanup complete (supervisor_exited, no error)",
        cleanup.get("supervisor_exited") is True
        and cleanup.get("error") is None,
        f"got {cleanup}",
    )

    # 5. determinism: rerun and compare robot_bundle_identity
    print("\n[bonus] determinism rerun")
    res2 = run([
            str(CARGO_PHOXAL),
            "phoxal",
            "simulation",
            "run",
            "--headless",
            "--steps",
            str(STEPS),
            "--simulator",
            SIMULATOR,
            "--locked",
            str(SCENE),
        ])
    envelope2 = None
    outer2 = None
    for line in res2.stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("schema") != "phoxal/simulation-run/v0":
            continue
        if "native_bindings" in obj and envelope2 is None:
            envelope2 = obj
        elif "cleanup" in obj and outer2 is None:
            outer2 = obj
    identity1 = envelope["provenance"]["robot_bundle_identity"]
    identity2 = envelope2["provenance"]["robot_bundle_identity"]
    assert_step(
        "robot_bundle_identity stable across reruns",
        identity1 == identity2,
        f"{identity1} vs {identity2}",
    )

    # Log identity for future reference
    IDENTITY_LOG.parent.mkdir(parents=True, exist_ok=True)
    with IDENTITY_LOG.open("a") as f:
        f.write(f"{STEPS} steps\t{identity1}\n")

    print(f"\n  robot_bundle_identity = {identity1}")
    print("\n=== R2 acceptance: PASS ===\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())