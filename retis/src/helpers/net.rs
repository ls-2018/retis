use anyhow::{anyhow, Result};


/// Parses an IPv4 address into a String.
pub(crate) fn parse_ipv4_addr(raw: u32) -> Result<String> {
    let u8_to_utf8 = |addr: &mut String, mut input: u32| -> Result<()> {
        let mut push = false;

        for ord in [100, 10, 1] {
            let current = input / ord;
            input %= ord;

            // Do not push leading 0s but always push the last number in case
            // all we got was 0s.
            if push || current != 0 || ord == 1 {
                push = true;
                addr.push(
                    char::from_digit(current, 10).ok_or_else(|| anyhow!("invalid IPv4 digit"))?,
                );
            }
        }

        Ok(())
    };

    let mut addr = String::with_capacity(15);
    u8_to_utf8(&mut addr, raw >> 24)?;
    addr.push('.');
    u8_to_utf8(&mut addr, (raw >> 16) & 0xff)?;
    addr.push('.');
    u8_to_utf8(&mut addr, (raw >> 8) & 0xff)?;
    addr.push('.');
    u8_to_utf8(&mut addr, raw & 0xff)?;

    Ok(addr)
}


/// Parses an Ethernet address into a String.
pub(crate) fn parse_eth_addr(raw: &[u8; 6]) -> Result<String> {
    let mut addr = String::with_capacity(17);

    for (i, group) in raw.iter().enumerate() {
        addr.push(
            char::from_digit((group >> 4).into(), 16).ok_or_else(|| anyhow!("invalid eth byte"))?,
        );
        addr.push(
            char::from_digit((group & 0xf).into(), 16)
                .ok_or_else(|| anyhow!("invalid eth byte"))?,
        );
        if i < 5 {
            addr.push(':');
        }
    }

    Ok(addr)
}