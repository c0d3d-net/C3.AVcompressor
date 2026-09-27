use anyhow::{anyhow, Result};

/// Parse human time strings into seconds (as f64).
/// Supported formats:
/// - "01:23:45.67" (HH:MM:SS.mmm)
/// - "23:45" (MM:SS)
/// - "120" or "120s" (seconds)
/// - "2h15m", "1h30m20s", "45m10s", "500ms"
pub fn parse_time_to_seconds(input: &str) -> Result<f64> {
    let s = input.trim();
    if s.is_empty() {
        return Err(anyhow!("Empty time string"));
    }

    // Check if it's colon-separated: HH:MM:SS or MM:SS
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            2 => {
                let minutes: f64 = parts[0].parse().map_err(|_| anyhow!("Invalid minutes: {}", parts[0]))?;
                let seconds: f64 = parts[1].parse().map_err(|_| anyhow!("Invalid seconds: {}", parts[1]))?;
                return Ok(minutes * 60.0 + seconds);
            }
            3 => {
                let hours: f64 = parts[0].parse().map_err(|_| anyhow!("Invalid hours: {}", parts[0]))?;
                let minutes: f64 = parts[1].parse().map_err(|_| anyhow!("Invalid minutes: {}", parts[1]))?;
                let seconds: f64 = parts[2].parse().map_err(|_| anyhow!("Invalid seconds: {}", parts[2]))?;
                return Ok(hours * 3600.0 + minutes * 60.0 + seconds);
            }
            _ => return Err(anyhow!("Unsupported colon-separated time format: {}", s)),
        }
    }

    // Parse units by token scanning
    if s.contains('h') || s.contains('m') || s.contains('s') {
        let mut total = 0.0;
        let mut num_buf = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        let mut matched = false;

        while i < chars.len() {
            let c = chars[i];
            if c.is_ascii_digit() || c == '.' {
                num_buf.push(c);
                i += 1;
            } else if c == 'h' {
                if !num_buf.is_empty() {
                    let val: f64 = num_buf.parse()?;
                    total += val * 3600.0;
                    num_buf.clear();
                    matched = true;
                }
                i += 1;
            } else if c == 'm' {
                if i + 1 < chars.len() && chars[i + 1] == 's' {
                    // "ms"
                    if !num_buf.is_empty() {
                        let val: f64 = num_buf.parse()?;
                        total += val / 1000.0;
                        num_buf.clear();
                        matched = true;
                    }
                    i += 2;
                } else {
                    // "m" (minutes)
                    if !num_buf.is_empty() {
                        let val: f64 = num_buf.parse()?;
                        total += val * 60.0;
                        num_buf.clear();
                        matched = true;
                    }
                    i += 1;
                }
            } else if c == 's' {
                if !num_buf.is_empty() {
                    let val: f64 = num_buf.parse()?;
                    total += val;
                    num_buf.clear();
                    matched = true;
                }
                i += 1;
            } else if c.is_whitespace() {
                i += 1;
            } else {
                return Err(anyhow!("Unexpected character in time string: '{}'", c));
            }
        }

        if matched && num_buf.is_empty() {
            return Ok(total);
        }
    }

    // Try pure number
    let val: f64 = s.parse().map_err(|_| anyhow!("Cannot parse time value: '{}'", s))?;
    Ok(val)
}

/// Format seconds into HH:MM:SS.mmm string
pub fn format_seconds_to_time(seconds: f64) -> String {
    let total_millis = (seconds * 1000.0).round() as u64;
    let hours = total_millis / 3_600_000;
    let minutes = (total_millis % 3_600_000) / 60_000;
    let secs = (total_millis % 60_000) / 1000;
    let millis = total_millis % 1000;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time_to_seconds("10").unwrap(), 10.0);
        assert_eq!(parse_time_to_seconds("10s").unwrap(), 10.0);
        assert_eq!(parse_time_to_seconds("01:30").unwrap(), 90.0);
        assert_eq!(parse_time_to_seconds("01:00:00").unwrap(), 3600.0);
        assert_eq!(parse_time_to_seconds("00:01:30.500").unwrap(), 90.5);
        assert_eq!(parse_time_to_seconds("2h15m").unwrap(), 8100.0);
        assert_eq!(parse_time_to_seconds("1h30m20s").unwrap(), 5420.0);
        assert_eq!(parse_time_to_seconds("500ms").unwrap(), 0.5);
    }

    #[test]
    fn test_format_time() {
        assert_eq!(format_seconds_to_time(90.5), "00:01:30.500");
        assert_eq!(format_seconds_to_time(3661.123), "01:01:01.123");
    }
}
