export BPF_ARCH="arm64"
export BPF_CFLAGS="-target bpf -Wall -Wno-unused-value -Wno-pointer-sign -Wno-compare-distinct-pointer-types -fno-stack-protector -Werror -D__TARGET_ARCH_arm64 -O2"
export CFLAGS="-I/ebpf/retis/retis/src/collect/collector/ovs/bpf/include -I/ebpf/retis/retis/src/collect/collector/skb/bpf/include -I/ebpf/retis/retis/src/core/filters/meta/bpf/include -I/ebpf/retis/retis/src/core/filters/packets/bpf/include -I/ebpf/retis/retis/src/core/tracking/bpf/include -I/ebpf/retis/retis/src/core/events/bpf/include -I/ebpf/retis/retis/src/core/probe/user/bpf/include -I/ebpf/retis/retis/src/core/probe/bpf/include -I/ebpf/retis/retis/src/core/probe/kernel/bpf/include -I/ebpf/retis/retis/src/.out "
#make -f /ebpf/retis/ebpf.mk -C /ebpf/retis/retis/src/core/probe/kernel/bpf -n --dry-run -p
make -f /ebpf/retis/ebpf.mk -C /ebpf/retis/retis/src/core/probe/kernel/bpf



#