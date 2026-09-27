use anyhow::{anyhow, Result};

/// Parse bitrate strings into kbps (kilobits per second).
/// Examples:
/// - "5M", "5m", "5mbps" -> 5000
/// - "2.5M" -> 2500
/// - "4500k", "4500kbps" -> 4500
/// - "4500" -> 4500
/// - "5000000" (raw bps) -> 5000
pub fn parse_bitrate_to_kbps(input: &str) -> Result<u32> {
    let s = input.trim().to_lowercase();
    if s.is_empty() {
        return Err(anyhow!("Empty bitrate string"));
    }

    if s.ends_with("mbps") {
        let v: f64 = s[..s.len() - 4].trim().parse()?;
        return Ok((v * 1000.0).round() as u32);
    }
    if s.ends_with('m') {
        let v: f64 = s[..s.len() - 1].trim().parse()?;
        return Ok((v * 1000.0).round() as u32);
    }
    if s.ends_with("kbps") {
        let v: f64 = s[..s.len() - 4].trim().parse()?;
        return Ok(v.round() as u32);
    }
    if s.ends_with('k') {
        let v: f64 = s[..s.len() - 1].trim().parse()?;
        return Ok(v.round() as u32);
    }

    let val: f64 = s.parse().map_err(|_| anyhow!("Invalid bitrate format: '{}'", s))?;
    if val >= 50_000.0 {
        // Raw bps (e.g. 5000000 -> 5000 kbps)
        Ok((val / 1000.0).round() as u32)
    } else {
        // Already in kbps (e.g. 4500)
        Ok(val.round() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bitrate() {
        assert_eq!(parse_bitrate_to_kbps("5M").unwrap(), 5000);
        assert_eq!(parse_bitrate_to_kbps("2.5m").unwrap(), 2500);
        assert_eq!(parse_bitrate_to_kbps("4500k").unwrap(), 4500);
        assert_eq!(parse_bitrate_to_kbps("4500kbps").unwrap(), 4500);
        assert_eq!(parse_bitrate_to_kbps("4500").unwrap(), 4500);
        assert_eq!(parse_bitrate_to_kbps("5000000").unwrap(), 5000);
    }
}
