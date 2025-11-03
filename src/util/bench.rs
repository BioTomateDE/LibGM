use colored::Colorize;
use cpu_time::ProcessTime;
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

pub trait DurFmt {
    fn ms(&self) -> String;
}
impl DurFmt for Duration {
    fn ms(&self) -> String {
        format!("{:.2} ms", self.as_secs_f64() * 1000.0)
    }
}

pub struct Stopwatch {
    cpu_time: ProcessTime,
    real_time: Instant,
}
impl Stopwatch {
    pub fn start() -> Self {
        Self { cpu_time: ProcessTime::now(), real_time: Instant::now() }
    }
    pub fn elapsed_real(&self) -> Duration {
        self.real_time.elapsed()
    }
    pub fn elapsed_cpu(&self) -> Duration {
        self.cpu_time.elapsed()
    }
}
impl Display for Stopwatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let real: Duration = self.elapsed_real();
        let cpu: Duration = self.elapsed_cpu();
        write!(
            f,
            "{} {} {}{}",
            real.ms().bright_magenta(),
            "(cpu:".dimmed(),
            cpu.ms().magenta(),
            ")".dimmed()
        )
    }
}
