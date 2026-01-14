use std::{
    fmt::{Display, Formatter},
    time::{Duration, Instant},
};

pub struct Stopwatch {
    start_timestamp: Instant,
}

impl Stopwatch {
    #[must_use]
    pub fn start() -> Self {
        Self { start_timestamp: Instant::now() }
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_timestamp.elapsed()
    }
}

impl Display for Stopwatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dur = self.elapsed();
        let millis: f32 = dur.as_secs_f32() * 1000.0;
        write!(f, "{millis:.2} ms")
    }
}
