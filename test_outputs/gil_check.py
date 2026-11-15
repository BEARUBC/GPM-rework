import time
import threading

def test_function():
    total = 0
    for i in range(200000):
        total += i

CALLS = 200
THREADS = 4

start = time.perf_counter()
for _ in range(CALLS):
    test_function()
one_thread = time.perf_counter() - start

def worker():
    for _ in range(CALLS // THREADS):
        test_function()

threads = [threading.Thread(target=worker) for _ in range(THREADS)]
start = time.perf_counter()
for t in threads:
    t.start()
for t in threads:
    t.join()     
many_threads = time.perf_counter() - start

speedup = one_thread / many_threads
print("1 thread:  %.0f ms" % (one_thread * 1000))
print("%d threads: %.0f ms" % (THREADS, many_threads * 1000))
print("Speedup: %.2fx" % speedup)
if speedup > 1.5:
    print("-> GIL is released. Multithreading could help.")
else:
    print("-> GIL is NOT released. Multithreading won't speed this up.")
