use anyhow::{bail, Result};
use nix::time::{clock_gettime, ClockId};

use crate::events::TimeSpec;

/// Returns the monotonic timestamp in nanoseconds.
pub(crate) fn monotonic_timestamp() -> Result<u64> {
    let monotonic = clock_gettime(ClockId::CLOCK_MONOTONIC)?;

    let ts = monotonic.tv_sec() * 1000000000 + monotonic.tv_nsec();
    if ts < 0 {
        bail!("Monotonic timestamp is negative: {ts}");
    }

    Ok(ts as u64)
}

/// Computes and returns the offset of CLOCK_MONOTONIC to the wall-clock time.
pub(crate) fn monotonic_clock_offset() -> Result<TimeSpec> {
    let realtime = clock_gettime(ClockId::CLOCK_REALTIME)?; // 系统当前的挂钟时间，受系统时间更改影响
    let monotonic = clock_gettime(ClockId::CLOCK_MONOTONIC)?; // 表示从某个固定时间点（通常是系统启动）到现在所经过的时间，不会受系统时间的更改影响
    let offset = realtime - monotonic;
    Ok(TimeSpec::new(offset.tv_sec(), offset.tv_nsec()))
}
