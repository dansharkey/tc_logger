use crate::serial;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub const CHANNEL_NAMES: [&str; 4] = ["T1", "T2", "T3", "T4"];

#[derive(Clone)]
pub struct Sample {
    pub elapsed: f64,
    pub temps: [f32; 4],
}

#[derive(Clone)]
pub struct Marker {
    pub elapsed: f64,
    pub label: String,
}

/// Per-channel summary, computed once per frame rather than inline in the UI.
#[derive(Clone, Default)]
pub struct ChannelStats {
    pub latest: Option<f32>,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

impl ChannelStats {
    pub fn compute(samples: &[Sample], ch: usize) -> Self {
        let values: Vec<f32> = samples
            .iter()
            .map(|s| s.temps[ch])
            .filter(|v| !v.is_nan())
            .collect();

        Self {
            latest: values.last().copied(),
            min: values.iter().copied().reduce(f32::min),
            max: values.iter().copied().reduce(f32::max),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum State {
    Idle,
    Running,
    Paused,
}

struct Shared {
    pub samples: Vec<Sample>,
    pub markers: Vec<Marker>,
    pub state: State,
    pub error: Option<String>,
}

/// Public handle to the capture system.
pub struct Capture {
    shared: Arc<Mutex<Shared>>,
    serial_thread: Option<thread::JoinHandle<()>>,
    capture_start: Option<Instant>,
    elapsed_at_pause: f64,
}

impl Capture {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared {
                samples: Vec::new(),
                markers: Vec::new(),
                state: State::Idle,
                error: None,
            })),
            serial_thread: None,
            capture_start: None,
            elapsed_at_pause: 0.0,
        }
    }

    pub fn state(&self) -> State {
        self.shared.lock().unwrap().state.clone()
    }

    pub fn error(&self) -> Option<String> {
        self.shared.lock().unwrap().error.clone()
    }

    pub fn elapsed(&self) -> f64 {
        self.elapsed_at_pause
            + self.capture_start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0)
    }

    /// Clone current buffer for rendering — avoids holding the lock across UI.
    pub fn snapshot(&self) -> (Vec<Sample>, Vec<Marker>) {
        let s = self.shared.lock().unwrap();
        (s.samples.clone(), s.markers.clone())
    }

    pub fn start(&mut self, port_name: &str, interval_ms: u64) {
        let shared = Arc::clone(&self.shared);
        {
            let mut s = shared.lock().unwrap();
            s.samples.clear();
            s.markers.clear();
            s.error = None;
            s.state = State::Running;
        }
        self.capture_start = Some(Instant::now());
        self.elapsed_at_pause = 0.0;

        let port_name = port_name.to_string();
        let interval = Duration::from_millis(interval_ms);

        self.serial_thread = Some(thread::spawn(move || {
            let mut port = match serial::open(&port_name) {
                Ok(p) => p,
                Err(e) => {
                    let mut s = shared.lock().unwrap();
                    s.error = Some(format!("Failed to open port: {e}"));
                    s.state = State::Idle;
                    return;
                }
            };

            let start = Instant::now();
            loop {
                match shared.lock().unwrap().state.clone() {
                    State::Idle => break,
                    State::Paused => { thread::sleep(Duration::from_millis(50)); continue; }
                    State::Running => {}
                }

                if let Some(temps) = serial::read_sample(&mut port) {
                    shared.lock().unwrap().samples.push(Sample {
                        elapsed: start.elapsed().as_secs_f64(),
                        temps,
                    });
                }

                thread::sleep(interval);
            }
        }));
    }

    pub fn pause(&mut self) {
        self.shared.lock().unwrap().state = State::Paused;
        self.elapsed_at_pause += self.capture_start.take()
            .map(|s| s.elapsed().as_secs_f64())
            .unwrap_or(0.0);
    }

    pub fn resume(&mut self) {
        self.shared.lock().unwrap().state = State::Running;
        self.capture_start = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.shared.lock().unwrap().state = State::Idle;
        if let Some(h) = self.serial_thread.take() { let _ = h.join(); }
        self.capture_start = None;
    }

    pub fn add_marker(&mut self, label: String) {
        let elapsed = self.elapsed();
        self.shared.lock().unwrap().markers.push(Marker { elapsed, label });
    }
}
