use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

//misura e accumula tempo di cpu
pub fn update_cpu_time(total_cpu_time: Arc<Mutex<Duration>>, start: Instant) {
    let elapsed = start.elapsed();
    let mut total = total_cpu_time.lock().unwrap();
    *total += elapsed;
}
