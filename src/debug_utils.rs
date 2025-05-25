#[macro_export]
macro_rules! trace_build {
    ($label:expr, $expr:expr) => {{
        let _start = ::cpu_time::ProcessTime::now();
        let result = $expr;
        ::log::trace!("Building chunk '{}' took {}", $label, _start.elapsed().ms());
        result
    }};
}

#[macro_export]
macro_rules! trace_parse {
    ($label:expr, $expr:expr) => {{
        let _start = ::cpu_time::ProcessTime::now();
        let result = $expr;
        ::log::trace!("Parsing chunk '{}' took {}", $label, _start.elapsed().ms());
        result
    }};
}



pub trait DurationExt {
    fn ms(&self) -> String;
}

impl DurationExt for std::time::Duration {
    fn ms(&self) -> String {
        format!("{:.2} ms", self.as_secs_f64() * 1000.0)
    }
}
