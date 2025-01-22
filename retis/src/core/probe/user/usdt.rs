use std::{collections::HashSet, mem::MaybeUninit, os::fd::RawFd};

use anyhow::{anyhow, bail, Result};
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};

use crate::core::{
    filters::Filter,
    probe::{builder::*, Hook, Probe, ProbeType},
};

mod usdt_bpf {
    include!("bpf/.out/usdt.skel.rs");
}
use usdt_bpf::UsdtSkelBuilder;

#[derive(Default)]
pub(crate) struct UsdtBuilder {
    links: Vec<libbpf_rs::Link>,
    map_fds: Vec<(String, RawFd)>,
    hooks: HashSet<Hook>,
}

impl ProbeBuilder for UsdtBuilder {
    fn new() -> UsdtBuilder {
        UsdtBuilder::default()
    }

    fn init(
        &mut self,
        map_fds: Vec<(String, RawFd)>,
        hooks: HashSet<Hook>,
        _filters: Vec<Filter>,
    ) -> Result<()> {
        self.map_fds = map_fds;
        if hooks.len() > 1 {
            bail!("USDT Probes only support a single hook");
        }
        self.hooks = hooks;
        Ok(())
    }

    fn attach(&mut self, probe: &Probe) -> Result<()> {
        let probe = match probe.r#type() {
            ProbeType::Usdt(usdt) => usdt,
            _ => bail!("Wrong probe type"),
        };

        let mut open_object = MaybeUninit::uninit();
        let mut skel = UsdtSkelBuilder::default().open(&mut open_object)?;
        skel.maps.rodata_data.log_level = log::max_level() as u8;

        reuse_map_fds(skel.open_object_mut(), &self.map_fds)?;

        let skel = skel.load()?;
        let prog = skel
            .object()
            .progs_mut()
            .find(|p| p.name() == "probe_usdt")
            .ok_or_else(|| anyhow!("Couldn't get program"))?;

        self.links
            .push(prog.attach_usdt(probe.pid, &probe.path, &probe.provider, &probe.name)?);

        Ok(())
    }

    fn detach(&mut self) -> Result<()> {
        self.links.drain(..);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::{probe::user::UsdtProbe, user::proc::Process};

    use ::probe::probe as define_usdt;

    #[test]
    #[cfg_attr(not(feature = "test_cap_bpf"), ignore)]
    fn init_and_attach_usdt() {
        define_usdt!(test_builder, usdt, 1);

        let mut builder = UsdtBuilder::new();

        let p = Process::from_pid(std::process::id() as i32).unwrap();

        // It's for now, the probes below won't do much.
        assert!(builder.init(Vec::new(), Vec::new(), Vec::new()).is_ok());
        assert!(builder
            .attach(&Probe::usdt(UsdtProbe::new(&p, "test_builder::usdt").unwrap()).unwrap())
            .is_ok());
    }
}
