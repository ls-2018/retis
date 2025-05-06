use anyhow::Result;

use crate::{bindings::common_uapi::retis_probe_config, core::kernel::Symbol};

/// Find the offset of drop reason enums. We do not distinguish them as they're
/// part of the same (virtual) enum.
fn drop_reason_offset(symbol: &Symbol) -> Result<Option<i8>> {
    Ok(
        #[allow(clippy::manual_map)]
        if let Some(offset) = symbol.parameter_offset("enum skb_drop_reason")? {
            Some(offset as i8)
        } else if let Some(offset) = symbol.parameter_offset("enum mac80211_drop_reason")? {
            Some(offset as i8)
        } else if let Some(offset) = symbol.parameter_offset("enum ovs_drop_reason")? {
            Some(offset as i8)
        } else {
            None
        },
    )
}

/// Inspect a target using BTF and fill its description.
pub(super) fn inspect_symbol(symbol: &Symbol) -> Result<retis_probe_config> {
    let mut cfg = retis_probe_config::default();

    // Look for known parameter types.
    if let Some(offset) = symbol.parameter_offset("struct sk_buff *")? {
        cfg.offsets.sk_buff = offset as i8;
    }
    if let Some(offset) = drop_reason_offset(symbol)? {
        cfg.offsets.skb_drop_reason = offset;
    }
    if let Some(offset) = symbol.parameter_offset("struct net_device *")? {
        cfg.offsets.net_device = offset as i8;
    }
    if let Some(offset) = symbol.parameter_offset("struct net *")? {
        cfg.offsets.net = offset as i8;
    }
    if let Some(offset) = symbol.parameter_offset("struct nft_pktinfo *")? {
        cfg.offsets.nft_pktinfo = offset as i8;
    }
    if let Some(offset) = symbol.parameter_offset("struct nft_traceinfo *")? {
        cfg.offsets.nft_traceinfo = offset as i8;
    }

    Ok(cfg)
}
