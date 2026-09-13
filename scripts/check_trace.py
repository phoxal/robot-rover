#!/usr/bin/env python3
"""Verify real receipt lineage and startup from an opt-in supervisor trace.

This proves input admission and output/application boundaries. It does not
claim that a disarmed run proves cancellation of a moving robot.
"""

import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path

from check_latency import analyze


def check(manifest, log):
    bundle = json.loads(manifest)
    analysis = analyze(bundle)
    records = []
    terminal = None
    for line in log.splitlines():
        if "phoxal::boundary: record=" in line:
            record = json.loads(line.split("record=", 1)[1])
            record["host_time"] = datetime.fromisoformat(line.split()[0].replace("Z", "+00:00"))
            records.append(record)
        elif line.startswith("{"):
            candidate = json.loads(line)
            if candidate.get("provider_contract_verified") and candidate.get("outcome") == "success":
                terminal = candidate
    assert terminal, "trace has no verified successful terminal record"
    identity = (terminal["execution_id"], terminal["timeline_id"])
    assert all((r["execution"], r["timeline"]) == identity for r in records), "mixed execution trace"
    invocations = {(r["runtime"], r["boundary"]): r for r in records if r["event"] == "invocation"}
    initialized = {r["runtime"]: r for r in records if r["event"] == "initialized"}
    expected_runtimes = {e["instance"] for e in bundle["executables"]}
    assert initialized.keys() == expected_runtimes
    assert max(r["host_time"] for r in initialized.values()) <= min(r["host_time"] for r in invocations.values())
    admissions = {r["boundary"]: r for r in records if r["event"] == "admission"}
    assert sorted(admissions) == list(range(terminal["completed_steps"] + 1))

    # Every configured bootstrap value exists before the first freeze,
    # including an unchanged producer started before its consumer.
    safety_zero = invocations[("safety", 0)]
    assert any(i["source"] == "motion" and i["port"] == "status" and i["sequence"] == 0 and i["items"] == 1 for i in safety_zero["inputs"])
    for (runtime, boundary), invocation in invocations.items():
        if runtime == "safety":
            assert any(i["input"] == "motion" and i["items"] == 1 for i in invocation["inputs"]), f"motion status missing at {boundary}"

    expected_wheels = {"front_left_drive", "rear_left_drive", "front_right_drive", "rear_right_drive"}
    for (runtime, boundary), invocation in invocations.items():
        if runtime == "kinematics":
            actual = {item["source"] for item in invocation["inputs"] if item["input"] == "encoders" and item["items"] == 1}
            assert actual == expected_wheels, f"incomplete wheel capture at {boundary}: {actual}"
    actuators = {actuator["actuator_id"] for binding in terminal["native_bindings"]["actuation"] for actuator in binding["actuators"]}
    assert actuators == {wheel + ".motor" for wheel in expected_wheels}

    proofs = []
    unexercised = []
    for path in analysis["paths"]:
        trace = path["worst_capture_trace"]
        captured = trace["capture_boundary"]
        source, port = path["source"].split(".")
        candidates = [o for o in admissions[captured]["observations"] if o["source"] == source and o["port"] == port and o["items"]]
        if not candidates:
            unexercised.append({"path": path["path"], "reason": "No source stimulus was published in this disarmed run."})
            continue
        sequence = candidates[0]["sequence"]
        for stage in trace["stages"]:
            runtime, field = stage["consumer"].split(".")
            invocation = invocations[(runtime, stage["boundary"])]
            assert any(i["input"] == field and i["source"] == source and i["port"] == port and i["sequence"] == sequence and i["items"] for i in invocation["inputs"]), f"missing input lineage: {path['path']} at {stage}"
            source, port = stage["output"].split(".")
            products = [o for o in invocation["products"] if o["port"] == port and o["items"]]
            assert len(products) == 1
            sequence = products[0]["sequence"]
        applied = admissions[trace["applied_boundary"]]
        assert any(a["membership"]["source"] == source and a["membership"]["port"] == port and a["membership"]["sequence"] == sequence for a in applied["actuation"])
        observed_ns = (applied["boundary"] - captured) * analysis["quantum_ns"]
        assert observed_ns == path["max_capture_to_application_ns"]
        proofs.append({"path": path["path"], "observed_capture_to_application_ns": observed_ns, "trace": trace})

    for boundary, admission in admissions.items():
        for actuation in admission["actuation"]:
            member = actuation["membership"]
            assert member["capture_boundary"] < boundary
            assert member["capture_time_ns"] == member["capture_boundary"] * analysis["quantum_ns"]
            assert actuation["valid_until_ns"] >= boundary * analysis["quantum_ns"]
    host_intervals = [(admissions[n]["host_time"] - admissions[n - 1]["host_time"]).total_seconds() for n in range(1, len(admissions))]
    return {"schema": "robot-rover/receipt-trace-proof/v0", "execution": identity[0], "timeline": identity[1],
            "manifest_sha256": hashlib.sha256(manifest).hexdigest(), "log_sha256": hashlib.sha256(log.encode()).hexdigest(),
            "completed_steps": terminal["completed_steps"], "initialized_runtimes": sorted(initialized),
            "maximum_traced_host_boundary_seconds": max(host_intervals), "paths": proofs,
            "unexercised": unexercised,
            "limitation": "Receipt lineage proves scheduling and admission; physical stopping and cancellation require an armed stimulus fixture."}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("log", type=Path)
    args = parser.parse_args()
    print(json.dumps(check(args.manifest.read_bytes(), args.log.read_text()), indent=2))
