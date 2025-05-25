#[macro_export]
macro_rules! trace_build {
    ($label:expr, $expr:expr) => {{
        let _start = ::cpu_time::ProcessTime::now();
        let result = $expr;
        ::log::trace!("Building chunk '{}' took {:.2?}", $label, _start.elapsed());
        result
    }};
}

#[macro_export]
macro_rules! trace_parse {
    ($label:expr, $expr:expr) => {{
        let _start = ::cpu_time::ProcessTime::now();
        let result = $expr;
        ::log::trace!("Parsing chunk '{}' took {:.2?}", $label, _start.elapsed());
        result
    }};
}

