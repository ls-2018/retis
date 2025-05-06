//! # Skb Tracking Module
//!
//! Reports tracking data.

// Re-export skb_tracking
#[allow(clippy::module_inception)]
pub(crate) mod skb_tracking;
pub(crate) use skb_tracking::*;

mod tracking_hook {
    include!("bpf/.out/tracking_hook.rs");
}
