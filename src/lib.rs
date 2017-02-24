#[macro_export]
macro_rules! dur_to_ns {
    ($d:expr) => {{
        const NANOS_PER_SEC: u64 = 1_000_000_000;
        let d = $d;
        d.as_secs() * NANOS_PER_SEC + d.subsec_nanos() as u64
    }}
}
