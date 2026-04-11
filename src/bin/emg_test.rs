use gpm::hardware::{emg::Emg, Resource};
use std::time::{Duration, Instant};

const BUFFER_SIZE: usize = 64;
const INNER_THRESHOLD: f32 = 450.0;
const OUTER_THRESHOLD: f32 = 450.0;
const LOOP_MS: u64 = 100;

fn main() {
    let mut emg = Emg::init();
    emg.configure(BUFFER_SIZE);
    emg.calibrate(INNER_THRESHOLD, OUTER_THRESHOLD);

    println!("EMG Test");
    println!(
        "  buffer_size={BUFFER_SIZE}, inner_threshold={INNER_THRESHOLD}, outer_threshold={OUTER_THRESHOLD}"
    );
    println!("  Press Ctrl+C to stop.\n");
    println!(
        "{:<12} {:<10} {:<10} {:<8} {:<8} {:<8} {:<8} {}",
        "time_ms", "ch0_avg", "ch1_avg", "ch0_min", "ch0_max", "ch1_min", "ch1_max", "gesture"
    );
    println!("{}", "-".repeat(76));

    let start = Instant::now();

    loop {
        let loop_start = Instant::now();

        match emg.read_buffer() {
            Ok(samples) => {
                let ch0: Vec<u16> = samples.iter().step_by(2).copied().collect();
                let ch1: Vec<u16> = samples.iter().skip(1).step_by(2).copied().collect();

                let ch0_avg = ch0.iter().sum::<u16>() as f32 / ch0.len() as f32;
                let ch1_avg = ch1.iter().sum::<u16>() as f32 / ch1.len() as f32;

                let ch0_min = ch0.iter().min().copied().unwrap_or(0);
                let ch0_max = ch0.iter().max().copied().unwrap_or(0);
                let ch1_min = ch1.iter().min().copied().unwrap_or(0);
                let ch1_max = ch1.iter().max().copied().unwrap_or(0);

                let gesture = emg.process_data(&[ch0_avg, ch1_avg]).unwrap_or(-1);
                let label = match gesture {
                    1 => "OPEN",
                    0 => "CLOSE",
                    _ => "HOLD",
                };

                println!(
                    "{:<12} {:<10.1} {:<10.1} {:<8} {:<8} {:<8} {:<8} {}",
                    start.elapsed().as_millis(),
                    ch0_avg,
                    ch1_avg,
                    ch0_min,
                    ch0_max,
                    ch1_min,
                    ch1_max,
                    label,
                );
            }
            Err(e) => eprintln!("read error: {e}"),
        }

        let elapsed = loop_start.elapsed();
        if elapsed < Duration::from_millis(LOOP_MS) {
            std::thread::sleep(Duration::from_millis(LOOP_MS) - elapsed);
        }
    }
}
