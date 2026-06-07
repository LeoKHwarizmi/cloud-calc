// src/user.rs
use anyhow::Result;
use std::time::Instant;

pub struct UsageSample {
    pub cpu_seconds: f64,
    pub ram_bytes: i64,
}

pub fn measure_start() -> Instant {
    Instant::now()
}

pub fn measure_end(start: Instant) -> Result<UsageSample> {
    let elapsed = start.elapsed();
    let cpu_seconds = elapsed.as_secs_f64();

    let ram_bytes = get_rss_bytes().unwrap_or(0);

    Ok(UsageSample {
        cpu_seconds,
        ram_bytes,
    })
}

fn get_rss_bytes() -> Result<i64, procfs::ProcError> {
    #[cfg(target_os = "linux")]
    {
        use procfs::process::Process;
        let me = Process::myself()?;
        Ok((me.stat()?.rss * 4096) as i64)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(0)
    }
}
