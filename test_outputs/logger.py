import os
import csv
import time
import psutil 

LOG_DIR = "time_logs"

run_name = time.strftime("%Y-%m-%d_%H-%M-%S")
timing_file = os.path.join(LOG_DIR, run_name + " timing.csv")
resource_file = os.path.join(LOG_DIR, run_name + " resources.csv")
emg_file = os.path.join(LOG_DIR, run_name + " emg.csv")

SAVE_EVERY = 1000
timing_rows = []
emg_rows = []

iteration = 0              
start_of_run = 0          
process = psutil.Process()  

def init_logger():
    global start_of_run
    os.makedirs(LOG_DIR, exist_ok=True)

    with open(timing_file, "w", newline="") as f:
        csv.writer(f).writerow(["time_s", "iteration", "name", "duration_ms"])

    with open(resource_file, "w", newline="") as f:
        csv.writer(f).writerow(["time_s", "iteration", "cpu_percent", "memory_mb"])

    with open(emg_file, "w", newline="") as f:
        csv.writer(f).writerow(["time_s", "iteration", "channel", "value"])

    process.cpu_percent()   
    start_of_run = time.perf_counter()


def next_iteration():
    global iteration
    iteration +=1 


def log_time(name, start, end):
    time_s = start - start_of_run         
    duration_ms = (end - start) * 1000    
    timing_rows.append([time_s, iteration, name, duration_ms])

    if len(timing_rows) >= SAVE_EVERY:
        save_log()


def log_emg(channel, samples):
    time_s = time.perf_counter() - start_of_run
    if isinstance(samples, (int, float)):
        samples = [samples]              
    for value in samples:
        emg_rows.append([time_s, iteration, channel, float(value)])

    if len(emg_rows) >= SAVE_EVERY:
        save_log()


def log_resources():
    time_s = time.perf_counter() - start_of_run
    cpu = process.cpu_percent()                      
    memory_mb = process.memory_info().rss / 1024 / 1024
    with open(resource_file, "a", newline="") as f:
        csv.writer(f).writerow([time_s, iteration, cpu, memory_mb])


def save_log():
    if timing_rows:
        with open(timing_file, "a", newline="") as f:
            csv.writer(f).writerows(timing_rows)
        timing_rows.clear()

    if emg_rows:
        with open(emg_file, "a", newline="") as f:
            csv.writer(f).writerows(emg_rows)
        emg_rows.clear()
