#!/usr/bin/env python3
"""Chart exactly 30 completed render frames from the browser QA report."""
import collections
import csv
import json
from pathlib import Path
import statistics
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt


def main():
    report_path = Path(sys.argv[1])
    report = json.loads(report_path.read_text())
    assert report["passed"] and "webgpu" in report["rendererModule"]
    trace = json.loads(Path(report["waterfall"]).read_text())
    assert trace["droppedEvents"] == 0
    events = [e for e in trace["traceEvents"] if e.get("ph") == "X" and not e.get("inProgress")]
    start = report["walkTraceStartFrame"]
    renders = sorted((e for e in events if e["name"] == "render.frame" and e["args"]["frame"] > start), key=lambda e: e["ts"])
    assert len(renders) >= 31, "Need 30 full frames plus the next interval boundary"
    assert [e["args"]["frame"] for e in renders[:31]] == list(range(start + 1, start + 32))
    mains = sorted((e for e in events if e["name"] == "frame.main"), key=lambda e: e["ts"])
    paired = [max((e for e in mains if e["ts"] + e["dur"] <= frame["ts"]), key=lambda e: e["ts"]) for frame in renders[:31]]
    assert len({e["ts"] for e in paired}) == 31, "Main/render pairing must be one-to-one"
    stage_names = ["prepare_assets", "prepare_meshes", "prepare_views", "queue", "sort", "prepare_gpu", "submit"]
    cpu = {"Main app": [], "Extraction / gap": []}
    cpu.update({name: [] for name in stage_names})
    nested = collections.defaultdict(list)
    intervals = []
    submit_spans = []
    for i, (main_frame, frame) in enumerate(zip(paired[:30], renders[:30])):
        origin = main_frame["ts"]
        cpu["Main app"].append((0., main_frame["dur"] / 1000.))
        cpu["Extraction / gap"].append((main_frame["dur"] / 1000., (frame["ts"] - origin - main_frame["dur"]) / 1000.))
        intervals.append((paired[i + 1]["ts"] - origin) / 1000.)
        for name in stage_names:
            stages = [e for e in events if e["name"] == "render." + name and e["ts"] >= frame["ts"] and e["ts"] + e["dur"] <= frame["ts"] + frame["dur"] + .01]
            assert len(stages) == 1, (frame["args"]["frame"], name, len(stages))
            e = stages[0]
            if name == "submit": submit_spans.append(e)
            cpu[name].append(((e["ts"] - origin) / 1000., e["dur"] / 1000.))
        for name in ["frame.sync_3d", "frame.modeled_actors", "region.grove_stream", "region.grove_interest"]:
            nested[name].append(sum(e["dur"] for e in events if e["name"] == name and e["ts"] >= main_frame["ts"] and e["ts"] + e["dur"] <= main_frame["ts"] + main_frame["dur"] + .01) / 1000.)
    cpu_rows = [{"stage": name, "startMs": statistics.mean(a for a, _ in samples),
                 "meanMs": statistics.mean(b for _, b in samples), "samples": 30} for name, samples in cpu.items()]
    gpu = report.get("gpuTiming", {})
    gpu_rows = []
    gpu_totals = []
    unwritten = collections.Counter()
    clock_offset_range = None
    if gpu.get("supported"):
        assert gpu["frameCount"] == 30, "GPU capture must contain exactly 30 render submissions"
        # Submission CPU times must lie inside their corresponding Rust submit
        # spans under one fixed clock-origin offset. This verifies the ordinal
        # pairing instead of assuming every submission belongs to that frame.
        offsets = [(g["submitCpuMs"] * 1000. - s["ts"] - s["dur"],
                    g["submitCpuMs"] * 1000. - s["ts"])
                   for g, s in zip(gpu["frames"], submit_spans)]
        clock_offset_range = [max(a for a, _ in offsets), min(b for _, b in offsets)]
        assert clock_offset_range[0] <= clock_offset_range[1], "GPU/main frame pairing is inconsistent"
        grouped = collections.defaultdict(dict)
        for index, frame in enumerate(gpu["frames"]):
            for pass_record in frame["passes"]:
                if int(pass_record["startNs"]) == int(pass_record["endNs"]) == 0:
                    unwritten[pass_record["label"]] += 1
            passes = [p for p in frame["passes"] if int(p["startNs"]) > 0 and int(p["endNs"]) > 0]
            assert passes
            origin = min(int(p["startNs"]) for p in passes)
            end = max(int(p["endNs"]) for p in passes)
            assert 0 < origin <= end
            gpu_totals.append((end - origin) / 1e6)
            occurrence = collections.Counter()
            for p in passes:
                label = p["label"]
                occurrence[label] += 1
                name = f'{label} #{occurrence[label]}'
                a, b = int(p["startNs"]), int(p["endNs"])
                assert origin <= a <= b <= end
                grouped[name][index] = ((a - origin) / 1e6, (b - a) / 1e6)
        for name, samples in grouped.items():
            gpu_rows.append({"stage": name,
                "startMs": statistics.mean(a for a, _ in samples.values()),
                "meanMs": sum(b for _, b in samples.values()) / 30., "samples": len(samples)})
    result = {"frames": 30, "renderFrameIds": [e["args"]["frame"] for e in renders[:30]],
        "source": str(report_path), "module": report["rendererModule"],
        "meanEngineFrameIntervalMs": statistics.mean(intervals), "cpu": cpu_rows,
        "nestedMainMs": {name: statistics.mean(values) for name, values in nested.items()},
        "gpu": gpu_rows, "unwrittenTimestampPasses": dict(unwritten), "gpuCpuClockOffsetRangeUs": clock_offset_range, "meanGpuSpanMs": statistics.mean(gpu_totals) if gpu_totals else None,
        "meaning": "CPU schedule spans and GPU timestamp-query passes. Separate clocks; not displayed FPS. Nested main spans are not additive. GPU capture adds query/resolve work."}
    out = report_path.with_suffix("")
    Path(str(out) + "-30-frame-summary.json").write_text(json.dumps(result, indent=2) + "\n")
    with Path(str(out) + "-30-frame-stages.csv").open("w") as f:
        writer = csv.DictWriter(f, fieldnames=["timeline", "stage", "startMs", "meanMs", "samples"])
        writer.writeheader()
        for timeline, rows in [("CPU", cpu_rows), ("GPU", gpu_rows)]:
            for row in rows:
                writer.writerow({"timeline": timeline, **row})
    fig, axes = plt.subplots(2 if gpu_rows else 1, 1, figsize=(13, 13 if gpu_rows else 5), squeeze=False)
    for ax, (title, rows) in zip(axes[:, 0], [("CPU application + render schedule", cpu_rows), ("GPU passes (hardware timestamps)", gpu_rows)]):
        for i, row in enumerate(rows):
            color = "#e47732" if row["meanMs"] == max(r["meanMs"] for r in rows) else "#3788b9"
            ax.barh(i, row["meanMs"], left=row["startMs"], color=color, height=.65)
            ax.text(row["startMs"] + row["meanMs"] + .07, i, f'{row["meanMs"]:.2f} ms', va="center", fontsize=9)
        ax.set_yticks(range(len(rows)), [r["stage"].replace("_", " ") for r in rows])
        ax.invert_yaxis()
        ax.set_title(title, loc="left", fontweight="bold")
        ax.set_xlabel("Mean offset from this timeline's frame start (ms)")
        ax.grid(axis="x", alpha=.2)
        ax.set_axisbelow(True)
        ax.set_xlim(0, max(r["startMs"] + r["meanMs"] for r in rows) * 1.22 + .5)
    fig.suptitle(f'WebGPU {report.get("scenario", "walking")} waterfall — average of 30 consecutive frames\nEngine frame interval: {statistics.mean(intervals):.2f} ms', fontsize=15)
    fig.text(.02, .012, "CPU and GPU clocks are separate. Bars show averages, not one real frame. Query/resolve instrumentation is included; no GPU readback during the sample.", fontsize=8)
    fig.tight_layout(rect=[0, .04, 1, .92])
    fig.savefig(str(out) + "-30-frame-waterfall.png", dpi=180)
    fig.savefig(str(out) + "-30-frame-waterfall.svg")
    plt.close(fig)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
