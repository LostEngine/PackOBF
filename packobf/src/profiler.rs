#[cfg(feature = "profiling")]
use dashmap::DashMap;
#[cfg(feature = "profiling")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "profiling")]
use std::sync::LazyLock;
#[cfg(feature = "profiling")]
use std::time::{Duration, Instant};
#[cfg(feature = "profiling")]
use arc_swap::ArcSwap;

#[cfg(feature = "profiling")]
pub static PROFILER: LazyLock<ArcSwap<Profiler>> = LazyLock::new(|| ArcSwap::from_pointee(Profiler::new()));

#[cfg(feature = "profiling")]
pub struct Stat {
    pub total_ns: AtomicU64,
    pub calls: AtomicU64,
}

#[cfg(feature = "profiling")]
pub struct Profiler {
    stats: DashMap<&'static str, Stat>,
}

#[cfg(feature = "profiling")]
impl Profiler {
    pub fn new() -> Self {
        Self {
            stats: DashMap::new(),
        }
    }

    pub fn record(&self, name: &'static str, duration: Duration) {
        let nanos = duration.as_nanos() as u64;

        let entry = self.stats.entry(name).or_insert_with(|| Stat {
            total_ns: AtomicU64::new(0),
            calls: AtomicU64::new(0),
        });

        entry.total_ns.fetch_add(nanos, Ordering::Relaxed);
        entry.calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn print(&self) {
        let mut entries = self
            .stats
            .iter()
            .map(|e| {
                let total = e.total_ns.load(Ordering::Relaxed);
                let calls = e.calls.load(Ordering::Relaxed);

                (*e.key(), total, calls, total as f64 / calls.max(1) as f64)
            })
            .collect::<Vec<_>>();

        entries.sort_by_key(|e| std::cmp::Reverse(e.1));

        println!("==== PROFILING ====");

        for (name, total, calls, avg) in entries {
            println!(
                "{:<40} total={:>10.2}ms calls={:>8.2} avg={:>10.2}µs",
                name,
                total as f64 / 1_000_000.0,
                calls,
                avg / 1_000.0
            );
        }
    }
}

#[cfg(feature = "profiling")]
pub struct ScopeTimer {
    name: &'static str,
    start: Instant,
}

#[cfg(feature = "profiling")]
impl ScopeTimer {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

#[cfg(feature = "profiling")]
impl Drop for ScopeTimer {
    fn drop(&mut self) {
        PROFILER.load().record(self.name, self.start.elapsed());
    }
}

#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        #[cfg(feature = "profiling")]
        let _profiler_scope = $crate::profiler::profiler::ScopeTimer::new($name);
    };
}
