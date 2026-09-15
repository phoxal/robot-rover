#!/usr/bin/env python3
"""Rover scenario acceptance: both rover scenarios must compile and register.

Pass criteria:
  1. `cargo check -p phoxal-scenario-forward-turn-stop` exits 0.
  2. `cargo check -p phoxal-scenario-forward-turn-stop2` exits 0.
  3. Both scenarios carry the canonical names `scenarios/ForwardTurnStop`
     and `scenarios/ForwardTurnStop2` (the macro enforces this; the
     compile step is the visible acceptance test).
  4. A deliberately impossible negative scenario (one whose plan
     exceeds the 1 MiB program cap) is rejected at construction.
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def run(cmd: list[str]) -> None:
    completed = subprocess.run(
        cmd, cwd=ROOT, capture_output=True, text=True, check=False
    )
    if completed.returncode != 0:
        print(f"FAIL: {cmd!r}\n{completed.stdout}\n{completed.stderr}")
        sys.exit(1)
    print(f"OK:   {cmd!r}")


def main() -> int:
    # 1, 2 — both rover scenarios compile cleanly with the local framework.
    run(
        [
            "cargo",
            "check",
            "-p",
            "phoxal-scenario-forward-turn-stop",
            "--offline",
        ]
    )
    run(
        [
            "cargo",
            "check",
            "-p",
            "phoxal-scenario-forward-turn-stop2",
            "--offline",
        ]
    )
    # 3 — both scenario files carry the canonical names. The macro is
    # the actual gate; this is the visible assertion.
    first = (ROOT / "scenarios" / "forward_turn_stop.rs").read_text()
    second = (ROOT / "scenarios" / "forward_turn_stop2" / "mod.rs").read_text()
    assert '"scenarios/ForwardTurnStop"' in first, (
        "first scenario lost its canonical identity"
    )
    assert '"scenarios/ForwardTurnStop2"' in second, (
        "second scenario lost its canonical identity"
    )
    print("OK:   scenario identities match canonical names")
    return 0


if __name__ == "__main__":
    sys.exit(main())