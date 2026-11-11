import csv
import glob
import statistics

LOG_DIR = "time_logs"

files = sorted(glob.glob(LOG_DIR + "/*_timing.csv"))
run_name = files[-1].replace("_timing.csv", "")
print("Analyzing:", run_name)


def read_csv(path):
    with open(path) as f:
        return list(csv.DictReader(f))


def p99(values):
    values = sorted(values)
    return values[min(int(len(values) * 0.99), len(values) - 1)]


timing_rows = read_csv(run_name + "_timing.csv")
resource_rows = read_csv(run_name + "_resources.csv")
emg_rows = read_csv(run_name + "_emg.csv")
report = []

durations = {}
for row in timing_rows:
    durations.setdefault(row["name"], []).append(float(row["duration_ms"]))

report.append("TIMING (ms)")
report.append(f"{'name':<18}{'calls':>8}{'avg':>10}{'p99':>10}{'max':>10}{'std':>10}")
for name, values in durations.items():
    std = statistics.stdev(values) if len(values) > 1 else 0
    report.append(f"{name:<18}{len(values):>8}{statistics.mean(values):>10.4f}"
                  f"{p99(values):>10.4f}{max(values):>10.4f}{std:>10.4f}")
report.append("")

if "loop_total" in durations:
    loops = durations["loop_total"]
    n = max(1, len(loops) // 10)
    report.append("LOOP")
    report.append(f"Loops: {len(loops)}")
    report.append(f"Rate: {1000 / statistics.mean(loops):.1f} Hz")
    report.append(f"Avg loop time start: {statistics.mean(loops[:n]):.3f} ms")
    report.append(f"Avg loop time end: {statistics.mean(loops[-n:]):.3f} ms")
    report.append("")

cpu = [float(row["cpu_percent"]) for row in resource_rows]
memory = [float(row["memory_mb"]) for row in resource_rows]
if cpu:
    report.append("CPU / MEMORY")
    report.append(f"CPU avg: {statistics.mean(cpu):.1f}%  max: {max(cpu):.1f}%")
    report.append(f"Memory start: {memory[0]:.1f} MB  end: {memory[-1]:.1f} MB  max: {max(memory):.1f} MB")
    report.append("")

channels = {}
for row in emg_rows:
    channels.setdefault(row["channel"], []).append(float(row["value"]))

if channels:
    report.append("EMG")
    seconds = float(emg_rows[-1]["time_s"]) - float(emg_rows[0]["time_s"])

for ch, values in channels.items():
    jumps = [abs(values[i] - values[i - 1]) for i in range(1, len(values))]
    nonzero = [j for j in jumps if j > 0]
    report.append(f"Channel {ch}:")
    report.append(f"  samples: {len(values)}")
    if seconds > 0:
        report.append(f"  sample rate: {len(values) / seconds:.0f} Hz")
    report.append(f"  range: {min(values):g} to {max(values):g}")
    report.append(f"  unique values: {len(set(values))}")
    if nonzero:
        report.append(f"  resolution (smallest change): {min(nonzero):g}")
        report.append(f"  avg change: {statistics.mean(jumps):.2f}  max change: {max(jumps):g}")
    if jumps:
        report.append(f"  repeated samples: {100 * (len(jumps) - len(nonzero)) / len(jumps):.1f}%")
report.append("")

summary = "\n".join(report)
print(summary)
with open(run_name + "_summary.txt", "w") as f:
    f.write(summary)

print("Saved", run_name + "_summary.txt")