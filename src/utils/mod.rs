pub mod runtime;
pub mod version;
pub mod styled;
pub mod telemetry;

#[track_caller]
pub fn num_cpus() -> std::num::NonZeroUsize {
    match std::thread::available_parallelism() {
        Ok(parallelism) => parallelism,
        Err(err) => {
            log::warn!("failed to fetch the available parallelism (fallback to 1): {err:?}");
            std::num::NonZeroUsize::new(1).unwrap()
        }
    }
}