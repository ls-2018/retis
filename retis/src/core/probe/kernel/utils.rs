#![allow(unused_imports)]
use anyhow::{bail, Result};

use crate::core::probe::kernel::utils::CliProbeType::{Kprobe, Kretprobe, RawTracepoint};
use crate::core::{
    kernel::symbol::{matching_events_to_symbols, matching_functions_to_symbols, Symbol},
    probe::Probe,
};

/// Probe type for probes given through cli arguments.
pub(crate) enum CliProbeType {
    Kprobe,
    Kretprobe,
    RawTracepoint,
}

impl CliProbeType {
    pub(crate) fn to_str(&self) -> &'static str {
        use CliProbeType::*;
        match self {
            Kprobe => "kprobe",
            Kretprobe => "kretprobe",
            RawTracepoint => "raw_tracepoint",
        }
    }
}

/// 解析作为命令行参数给出的探测器，并返回其类型以及带有类型标识符的探测器（如果有）。
pub(crate) fn parse_cli_probe(input: &str) -> Result<(CliProbeType, &str)> {
    use CliProbeType::*;
    // tp:skb:kfree_skb   consume_skb
    Ok(match input.split_once(':') {
        Some((type_str, target)) => match type_str {
            "kprobe" | "k" => (Kprobe, target),
            "kretprobe" | "kr" => (Kretprobe, target),
            "raw_tracepoint" | "tp" => (RawTracepoint, target),
            // If a single ':' was found in the probe name but we didn't match
            // any known type, defaults to trying using it as a raw tracepoint.
            _ if input.chars().filter(|c| *c == ':').count() == 1 => (RawTracepoint, input),
            x => bail!("Invalid TYPE {}. See the help.", x),
        },
        // If no ':' was found, defaults to kprobe.
        None => (Kprobe, input),
    })
}

/// 解析用户自定义的探测器（通过命令行参数）并将其转换为我们的探测器表示形式（`Probe`）。
pub(crate) fn probe_from_cli<F>(probe: &str, filter: F) -> Result<Vec<Probe>>
where
    F: Fn(&Symbol) -> bool,
{
    use CliProbeType::*;

    let (r#type, target) = parse_cli_probe(probe)?;

    // Convert the target to a list of matching ones for probe types
    // supporting it.
    let mut symbols = match r#type {
        Kprobe | Kretprobe => matching_functions_to_symbols(target)?,
        RawTracepoint => matching_events_to_symbols(target)?,
    };

    let mut probes = Vec::new();
    for symbol in symbols.drain(..) {
        // Check if the symbol matches the filter.
        if !filter(&symbol) {
            // 用户指定的函数，是不是包含要追踪的参数
            continue;
        }

        probes.push(match r#type {
            Kprobe => Probe::kprobe(symbol)?,
            Kretprobe => Probe::kretprobe(symbol)?,
            RawTracepoint => Probe::raw_tracepoint(symbol)?,
        })
    }

    Ok(probes)
}
