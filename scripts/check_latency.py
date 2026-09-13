#!/usr/bin/env python3
"""Check reference-robot scheduling ceilings against an actual compiled bundle.

This computes logical scheduling bounds. It does not measure host processing,
prove that business logic reacts to a stimulus, or establish stopping distance.
"""

import argparse
import hashlib
import json
import math
from pathlib import Path


def analyze(bundle):
    quantum = bundle["simulation"]["quantum_ns"]
    runtimes = {e["instance"]: e["artifact"]["runtime"] for e in bundle["executables"]}
    connections = bundle["document"]["connections"]
    providers = {
        f'{p["service_instance"]}.{p["port"]}': p
        for p in bundle["simulation"]["providers"]
    }
    periods = {}
    outputs = {}
    for instance, runtime in runtimes.items():
        period_ns = runtime["period_ms"] * 1_000_000
        assert period_ns % quantum == 0, f"nonintegral period: {instance}"
        periods[instance] = period_ns // quantum
        for output in runtime["service_outputs"] + runtime["transient_outputs"]:
            if output["port"]:
                outputs[f'{instance}.{output["port"]}'] = output

    # These are robot acceptance paths, not an additional editable graph.
    # Every edge is checked against the compiler's retained authored graph.
    paths = [
        ("encoder_world_stop", "front_left_drive.encoder", [
            ("kinematics.encoders", "kinematics.odometry"),
            ("world.pose", "world.belief"),
            ("safety.world", "safety.constraints"),
            ("motion.safety", "motion.actuators"),
        ], 62, 82),
        ("encoder_motion", "front_left_drive.encoder", [
            ("kinematics.encoders", "kinematics.odometry"),
            ("motion.measurements", "motion.actuators"),
        ], 22, 42),
        ("range_stop", "front_center_tof.range", [
            ("safety.ranges", "safety.constraints"),
            ("motion.safety", "motion.actuators"),
        ], 32, 82),
        ("manual_cancel", "brain.manual", [
            ("motion.manual", "motion.actuators"),
        ], 22, 42),
    ]
    results = []
    for name, source, stages, capture_limit_ms, event_limit_ms in paths:
        previous = source
        stage_periods = []
        for consumer, output in stages:
            sources = connections[consumer]
            if isinstance(sources, str):
                sources = [sources]
            assert previous in sources, f"missing graph edge: {previous} -> {consumer}"
            instance = consumer.split(".")[0]
            assert output.split(".")[0] == instance
            declaration = outputs[output]
            assert declaration.get("every_steps") in (None, 1), f"publication divisor: {output}"
            stage_periods.append(periods[instance])
            previous = output
        assert any(
            f'{b["service_instance"]}.{b["port"]}' == previous
            for b in bundle["simulation"]["actuation_bindings"]
        ), f"path does not end in applied actuation: {name}"
        if source in providers:
            ticks = providers[source]["rate_microhertz"] * quantum
            denominator = 1_000_000_000_000_000
            assert 0 < ticks <= denominator
            cycle = denominator // math.gcd(ticks, denominator)
            due = lambda boundary: boundary == 0 or boundary * ticks // denominator > (boundary - 1) * ticks // denominator
            source_delay = 0
        else:
            cycle = periods[source.split(".")[0]]
            due = lambda boundary: boundary % cycle == 0
            source_delay = 1
        horizon = math.lcm(cycle, *stage_periods)
        assert horizon <= 1_000_000, "cadence cycle exceeds the acceptance fixture bound"
        worst_capture = worst_event = 0
        worst_trace = None
        previous_capture = 0
        for captured in range(horizon + 1):
            if not due(captured):
                continue
            eligible = captured + source_delay
            trace = []
            for (consumer, output), period in zip(stages, stage_periods):
                invoked = ((eligible + period - 1) // period) * period
                trace.append({"consumer": consumer, "output": output, "boundary": invoked})
                eligible = invoked + 1
            # The final output is selected at N and applied during N -> N+1.
            applied = trace[-1]["boundary"] + 1
            age = (applied - captured) * quantum
            event_age = (applied - previous_capture) * quantum if captured else 0
            if age >= worst_capture:
                worst_capture = age
                worst_trace = {"capture_boundary": captured, "stages": trace, "applied_boundary": applied}
            worst_event = max(worst_event, event_age)
            previous_capture = captured
        assert worst_capture <= capture_limit_ms * 1_000_000, f"capture age budget exceeded: {name}"
        assert worst_event <= event_limit_ms * 1_000_000, f"stimulus response budget exceeded: {name}"
        command_ttl = outputs[previous]["valid_for_ms"]
        assert command_ttl is not None and quantum <= command_ttl * 1_000_000
        results.append({"path": name, "source": source, "max_capture_to_application_ns": worst_capture,
                        "max_event_to_application_ns": worst_event,
                        "command_valid_for_ms": command_ttl, "worst_capture_trace": worst_trace})
    return {"schema": "robot-rover/latency-analysis/v0", "quantum_ns": quantum,
            "assumption": "Each stage publishes the relevant change at its first due invocation; host latency is excluded.",
            "paths": results}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    raw = args.manifest.read_bytes()
    result = analyze(json.loads(raw))
    result["manifest_sha256"] = hashlib.sha256(raw).hexdigest()
    print(json.dumps(result, indent=2))
